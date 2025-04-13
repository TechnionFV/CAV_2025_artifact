// ************************************************************************************************
// use
// ************************************************************************************************

use std::collections::BinaryHeap;

use fxhash::{FxHashMap, FxHashSet};

use crate::models::{
    circuit::{
        cut_enumeration::{Cut, CutSetItem},
        node_types::{CircuitAnd, CircuitGenericGate, CircuitNodeType},
        CircuitSimplifier,
    },
    signal_tracker::SignalTransformation,
    Circuit, Signal, TruthTable, UniqueSortedHashMap, UniqueSortedVec, Wire,
};

// ************************************************************************************************
// simplifier
// ************************************************************************************************

pub struct CircuitTechnologyMapper {
    k: usize,
    l: usize,
}

// ************************************************************************************************
// impl
// ************************************************************************************************

impl CircuitTechnologyMapper {
    fn insert_if_needed(
        signals: impl Iterator<Item = Signal>,
        signals_we_care_about: &FxHashSet<Signal>,
        max_heap: &mut BinaryHeap<Signal>,
    ) {
        for signal in signals.filter(|s| !signals_we_care_about.contains(s))
        // ignore signals we already saw
        {
            max_heap.push(signal.to_owned());
        }
    }
    // ********************************************************************************************
    // common
    // ********************************************************************************************

    fn get_signals_to_keep_from_chosen_cuts(
        circuit: &mut Circuit,
        cut_for_each_signal: &UniqueSortedHashMap<Signal, Cut>,
    ) -> FxHashSet<Signal> {
        let mut signals_we_care_about =
            FxHashSet::with_capacity_and_hasher(cut_for_each_signal.len(), Default::default());
        let mut max_heap = BinaryHeap::with_capacity(cut_for_each_signal.len());

        // add all important signals to the heap
        for s in circuit.important_signals.iter().copied() {
            max_heap.push(s);
        }

        while !max_heap.is_empty() {
            let current_signal = max_heap.pop().unwrap();

            while Some(&current_signal) == max_heap.peek() {
                max_heap.pop().unwrap();
            }

            signals_we_care_about.insert(current_signal);
            // println!("current signal: {:?}", current_signal);

            // add all cut signals to the heap
            let cut = cut_for_each_signal.get(&current_signal).unwrap();

            if cut.peek() == &[current_signal] {
                // unit cut, this may be an and gate with too many inputs
                // (used merge and gates prior)
                let node = circuit.nodes.get(&current_signal).unwrap();
                match &node.node_type {
                    CircuitNodeType::And(a) => {
                        Self::insert_if_needed(
                            a.inputs.iter().map(|w| w.signal()),
                            &signals_we_care_about,
                            &mut max_heap,
                        );
                    }
                    // this might also be signal or input
                    CircuitNodeType::ConstantZero
                    | CircuitNodeType::Input
                    | CircuitNodeType::Latch { .. } => {}
                    _ => unreachable!(),
                };
            } else {
                Self::insert_if_needed(cut.iter().copied(), &signals_we_care_about, &mut max_heap);
            };
        }

        signals_we_care_about
    }

    // fn _fix_negated_signals(
    //     circuit: &mut Circuit,
    //     negated_signals: &UniqueSortedHashMap<Signal, ()>,
    // ) {
    //     let signals: Vec<Signal> = circuit.nodes.iter_sorted().collect();
    //     let negate_wire = |w: &mut Wire| {
    //         if negated_signals.contains_key(&w.signal()) {
    //             *w = !*w;
    //         }
    //     };
    //     for signal in signals.into_iter() {
    //         let node = circuit.nodes.get_mut(&signal).unwrap();
    //         match &mut node.node_type {
    //             CircuitNodeType::And(a) => {
    //                 a.inputs.perform_operation_on_each_value(negate_wire);
    //             }
    //             CircuitNodeType::GenericGate(a) => {
    //                 a.inputs.perform_operation_on_each_value(negate_wire);
    //             }
    //             CircuitNodeType::Latch(a) => negate_wire(&mut a.input),
    //             _ => {}
    //         }
    //     }

    //     circuit.outputs.perform_operation_on_each_value(negate_wire);
    //     circuit.bad.perform_operation_on_each_value(negate_wire);
    //     circuit
    //         .constraints
    //         .perform_operation_on_each_value(negate_wire);
    // }

    fn make_changes(
        circuit: &mut Circuit,
        verbose: bool,
        set_of_signals_to_keep: &FxHashSet<Signal>,
        cut_and_truth_table_for_each_signal: &UniqueSortedHashMap<Signal, TruthTable>,
    ) -> UniqueSortedVec<Signal> {
        let signals: Vec<Signal> = circuit.nodes.iter_sorted().collect();
        let iterations = signals.len();
        let one_percent_iterations = iterations / 100;
        let ten_percent_iterations = iterations / 10;
        let initial_number_of_nodes = circuit.nodes.len();
        let mut should_add_ground = false;
        let mut removed_signals = UniqueSortedVec::new();
        // let mut negated_signals: UniqueSortedHashMap<Signal, ()> =
        // UniqueSortedHashMap::new_like(&self.nodes);
        for (i, signal) in signals.into_iter().enumerate() {
            // print progress
            if verbose && (ten_percent_iterations > 0) && i % ten_percent_iterations == 0 {
                println!(
                    "TECHNOLOGY MAPPING, current number of nodes: {}, progress = {}%",
                    circuit.nodes.len(),
                    if one_percent_iterations > 0 {
                        i / one_percent_iterations
                    } else {
                        0
                    }
                );
            }

            if set_of_signals_to_keep.contains(&signal) {
                // change node type to generic
                let node = circuit.nodes.get_mut(&signal).unwrap();
                match &mut node.node_type {
                    CircuitNodeType::And { .. } => {
                        let truth_table = cut_and_truth_table_for_each_signal.get(&signal).unwrap();
                        if truth_table.get_signals().peek() == &[signal] {
                            // unit cut was chosen, this can happen if an and gate has too many inputs
                            // (used merge and gates prior)
                            // do nothing
                        } else {
                            debug_assert!(
                                !truth_table.get_signals().contains(&signal),
                                "Loop in technology mapping."
                            );
                            if truth_table.is_all_zeros() {
                                should_add_ground = true;
                                node.node_type = CircuitNodeType::And(CircuitAnd {
                                    inputs: UniqueSortedVec::from_ordered_set(vec![
                                        Signal::GROUND.wire(false)
                                    ]),
                                })
                            } else if truth_table.is_all_ones() {
                                should_add_ground = true;
                                node.node_type = CircuitNodeType::And(CircuitAnd {
                                    inputs: UniqueSortedVec::from_ordered_set(vec![
                                        Signal::GROUND.wire(true)
                                    ]),
                                })
                            } else {
                                let truth_table = truth_table.to_owned();
                                // let normal_size = truth_table.calculate_area();
                                // truth_table.negate();
                                // let negated_size = truth_table.calculate_area();

                                // if negated_size < normal_size {
                                //     negated_signals.insert(signal, ());
                                // } else {
                                //     // negate again to get back to normal
                                //     truth_table.negate();
                                // }

                                node.node_type = CircuitNodeType::GenericGate(CircuitGenericGate {
                                    truth_table,
                                });
                            }
                        }
                    }
                    CircuitNodeType::GenericGate { .. } => {
                        panic!("Running technology mapping twice is not supported.")
                    }
                    _ => {}
                }
            } else {
                // we are allowed to delete inputs and latches in technology mapping
                // debug_assert!(matches!(
                //     self.nodes.get(&signal).unwrap().node_type,
                //     CircuitNodeType::And { .. }
                // ));
                circuit.nodes.remove(&signal);

                // remove from inputs or latches or and gates
                circuit.gates.remove(&signal);
                circuit.latches.remove(&signal);
                circuit.inputs.remove(&signal);
                removed_signals.push(signal);
                // debug_assert!(r);
            }
        }

        // self.fix_negated_signals(&negated_signals);

        if should_add_ground {
            circuit.add_ground_if_possible()
        }

        // fix levels
        // circuit.fix_levels_and_users();

        circuit.greatest_signal = circuit.nodes.max_key().unwrap();

        circuit.important_signals = circuit.recalculate_important_signals();

        debug_assert!(circuit.check().is_ok());

        if verbose {
            println!(
                "DONE! number of nodes before: {}, number of nodes after = {} ({}% reduction)",
                initial_number_of_nodes,
                circuit.nodes.len(),
                100.0
                    * (((initial_number_of_nodes - circuit.nodes.len()) as f32)
                        / (initial_number_of_nodes as f32)),
            );
        }

        removed_signals
    }

    pub fn calculate_truth_table_for_cut(
        circuit: &Circuit,
        signal: &Signal,
        cut: &Cut,
    ) -> TruthTable {
        // create mapping from signal to truth table
        let mut signal_to_truth_table =
            FxHashMap::with_capacity_and_hasher(cut.len() << 1, Default::default());
        let signals = circuit.get_cone_of_signal_bounded_by_cut(signal, cut);

        // calculate truth table for each signal
        for signal in signals.iter() {
            let node = circuit.nodes.get(signal).unwrap();
            if cut.contains(signal) {
                match node.node_type {
                    CircuitNodeType::ConstantZero => {
                        signal_to_truth_table
                            .insert(signal.to_owned(), TruthTable::new_constant_0());
                    }
                    _ => {
                        signal_to_truth_table.insert(
                            signal.to_owned(),
                            TruthTable::new_identity_truth_table(signal),
                        );
                    }
                }
            } else {
                // not part of the cut
                let wire_to_tt = |w: &Wire| {
                    if w.is_negated() {
                        let mut t = signal_to_truth_table.get(&w.signal()).unwrap().to_owned();
                        t.negate();
                        t
                    } else {
                        signal_to_truth_table.get(&w.signal()).unwrap().to_owned()
                    }
                };
                match &node.node_type {
                    CircuitNodeType::And(a) => {
                        let truth_tables = a.inputs.iter().map(wire_to_tt);

                        let mut tt = None;
                        for table in truth_tables {
                            match tt {
                                None => tt = Some(table), // first iteration
                                Some(t) => {
                                    tt = Some(TruthTable::and(t, table));
                                }
                            }
                        }
                        tt.as_mut().unwrap().simplify();
                        signal_to_truth_table.insert(signal.to_owned(), tt.unwrap());
                    }
                    CircuitNodeType::GenericGate(_) => {
                        // let truth_tables = a.inputs.iter().map(wire_to_tt);
                        todo!("Generic gates are not supported yet.")
                    }
                    CircuitNodeType::ConstantZero => {
                        signal_to_truth_table
                            .insert(signal.to_owned(), TruthTable::new_constant_0());
                    }
                    _ => {
                        signal_to_truth_table.insert(
                            signal.to_owned(),
                            TruthTable::new_identity_truth_table(signal),
                        );
                    }
                }
            }

            // let tt = signal_to_truth_table.get(signal).unwrap();
            // println!(
            //     "Signal = {}, cut = {}, truth = {}",
            //     signal,
            //     tt.get_signals(),
            //     tt
            // );
        }

        signal_to_truth_table.get(signal).unwrap().to_owned()
    }

    /// WARNING: this function may change the cuts slightly, this is because some cuts
    /// contain signals that are in the cone of each other. The mapping function
    /// here simply takes the first cut that it sees.
    /// For example:
    /// ```text
    /// x  _
    ///     \
    ///      y _
    /// w  _/   \
    ///          z
    /// v  _ ___/
    /// ```
    /// If the cut we want to calculate the truth table of is {x, y, v} (for signal z) then
    /// the truth table of z will only contain {y, v}
    fn calculate_truth_table_for_each_signal(
        circuit: &Circuit,
        cut_for_each_signal: &UniqueSortedHashMap<Signal, CutSetItem>,
    ) -> UniqueSortedHashMap<Signal, TruthTable> {
        let mut result = UniqueSortedHashMap::new_like(cut_for_each_signal);
        for signal in cut_for_each_signal.iter_sorted() {
            let csi = cut_for_each_signal.get(&signal).unwrap();
            // take truth table if it exists
            let truth_table = match &csi.truth_table {
                Some(tt) => tt.to_owned(),
                None => Self::calculate_truth_table_for_cut(circuit, &signal, &csi.cut),
            };
            result.insert(signal.to_owned(), truth_table);
        }
        result
    }

    // ********************************************************************************************
    // API
    // ********************************************************************************************

    // /// Returns the number of signals that will be kept after technology mapping
    // /// using the provided choices of cuts.
    // /// This prediction may be wrong if one of the cuts that is provided contains two signals
    // /// where one is in the cone of influence of the other
    // pub fn predict_tech_mapping_size(
    //     &self,
    //     cut_for_each_signal: &UniqueSortedHashMap<Signal, Cut>,
    // ) -> usize {
    //     let set_of_signals_to_keep = self.get_signals_to_keep_from_chosen_cuts(cut_for_each_signal);
    //     set_of_signals_to_keep.len()
    // }

    pub fn new(k: usize, l: usize) -> Self {
        Self { k, l }
    }

    /// Performs technology mapping using the provided choices of cuts.
    pub fn technology_map_using_provided_cuts(
        circuit: &mut Circuit,
        verbose: bool,
        cut_for_each_signal: &UniqueSortedHashMap<Signal, CutSetItem>,
    ) -> SignalTransformation {
        let cut_and_truth_table_for_each_signal: UniqueSortedHashMap<Signal, TruthTable> =
            Self::calculate_truth_table_for_each_signal(circuit, cut_for_each_signal);

        let mut cut_for_each_signal: UniqueSortedHashMap<Signal, Cut> =
            UniqueSortedHashMap::new_like(&cut_and_truth_table_for_each_signal);

        for signal in cut_and_truth_table_for_each_signal.iter_sorted() {
            let tt = cut_and_truth_table_for_each_signal.get(&signal).unwrap();
            cut_for_each_signal.insert(signal, tt.get_signals().to_owned());
        }
        // cut_and_truth_table_for_each_signal
        //     .into()
        //     .map(|(signal, (cut, _))| (signal.to_owned(), cut.to_owned()))
        //     .collect();

        let set_of_signals_to_keep =
            Self::get_signals_to_keep_from_chosen_cuts(circuit, &cut_for_each_signal);

        debug_assert!(circuit
            .get_output_wires()
            .iter()
            .chain(circuit.get_bad_wires().iter())
            .chain(circuit.get_invariant_constraint_wires().iter())
            .chain(circuit.get_wires_that_feed_into_latches().iter())
            .all(|x| set_of_signals_to_keep.contains(&x.signal())));
        debug_assert!(circuit
            .get_wires_that_feed_into_latches()
            .iter()
            .all(|x| set_of_signals_to_keep.contains(&x.signal())));

        let r = Self::make_changes(
            circuit,
            verbose,
            &set_of_signals_to_keep,
            &cut_and_truth_table_for_each_signal,
        );

        SignalTransformation::SignalsRemovedBecauseTheyAreNotUsed(r)
    }

    pub fn default_technology_mapping(&self, circuit: &mut Circuit) -> SignalTransformation {
        // enumerate cuts
        let refs = circuit.get_number_of_references();
        let cut_function = circuit.enumerate_k_feasible_cuts_leaving_only_l_best(
            self.k,
            self.l,
            |cut| circuit.cut_cost_function_used_by_abc(cut, self.k, &refs),
            true,
        );

        // choose cut per signal
        let cut_for_each_signal_2 =
            circuit.choose_cut_for_each_signal_using_area_flow(&cut_function);

        // perform technology mapping according to chosen cuts
        Self::technology_map_using_provided_cuts(circuit, false, &cut_for_each_signal_2)
    }
}

impl CircuitSimplifier for CircuitTechnologyMapper {
    fn simplify(&mut self, circuit: &mut Circuit) -> SignalTransformation {
        self.default_technology_mapping(circuit)
    }

    fn title(&self) -> String {
        "Technology mapping".to_string()
    }
}
