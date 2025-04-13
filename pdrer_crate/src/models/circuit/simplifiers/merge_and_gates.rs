// ************************************************************************************************
// use
// ************************************************************************************************

use crate::models::{
    circuit::{
        node_types::{CircuitAnd, CircuitNodeType},
        CircuitSimplifier,
    },
    signal_tracker::SignalTransformation,
    Circuit, Signal, UniqueSortedHashMap, UniqueSortedVec, Wire,
};

// ************************************************************************************************
// simplifier
// ************************************************************************************************

pub struct CircuitAndGateMerger {
    verbose: bool,
}

// ************************************************************************************************
// impl
// ************************************************************************************************

impl CircuitAndGateMerger {
    // ********************************************************************************************
    // helper functions
    // ********************************************************************************************

    fn find_input_that_is_eligible_for_merge(
        circuit: &Circuit,
        signal: &Signal,
        users: &UniqueSortedHashMap<Signal, UniqueSortedVec<Signal>>,
    ) -> Option<Wire> {
        let parent_node = circuit.nodes.get(signal).unwrap();

        let inputs = match &parent_node.node_type {
            CircuitNodeType::And(a) => &a.inputs,
            _ => unreachable!("This should never happen"),
        };

        inputs
            .iter()
            .find(|input| {
                // checks if this input is negated or if a negation of this input is already in the inputs.
                if input.is_negated() || (inputs.contains(&!(**input))) {
                    // can't merge if there is a negation
                    false
                } else {
                    let input_node = circuit.nodes.get(&input.signal()).unwrap();
                    match input_node.node_type {
                        CircuitNodeType::And { .. } => {
                            let input_node_users = users.get(&input.signal()).unwrap();
                            assert!(input_node_users.contains(signal));
                            if input_node_users.len() <= 1 {
                                let is_used_anywhere =
                                    circuit.important_signals.contains(&input.signal());
                                !is_used_anywhere
                            } else {
                                false
                            }
                        }
                        _ => false,
                    }
                }
            })
            .copied()
    }

    fn merge_and_gate_with_one_of_its_fan_ins_if_possible(
        circuit: &mut Circuit,
        removed_signals: &mut Vec<Signal>,
        users: &mut UniqueSortedHashMap<Signal, UniqueSortedVec<Signal>>,
        signal: &Signal,
    ) -> bool {
        let parent_node = circuit.nodes.get(signal).unwrap().to_owned();

        // check if node is an and gate
        match &parent_node.node_type {
            CircuitNodeType::And(a) => &a.inputs,
            _ => {
                return false;
            }
        };

        // find input that is and gate that can be merged
        let some_input = Self::find_input_that_is_eligible_for_merge(circuit, signal, users);

        // was there an input that was found
        let son_signal = match some_input {
            Some(input) => input.signal(),
            None => {
                // no input that can be merged
                return false;
            }
        };

        Self::merge_son_into_parent(circuit, signal, &son_signal, users);
        removed_signals.push(son_signal);

        true
    }

    fn merge_son_into_parent(
        circuit: &mut Circuit,
        parent_signal: &Signal,
        son_signal: &Signal,
        users: &mut UniqueSortedHashMap<Signal, UniqueSortedVec<Signal>>,
    ) {
        // get son node
        let son_node_users = users.get(son_signal).unwrap().to_owned();
        let son_node = circuit.nodes.get(son_signal).unwrap().to_owned(); // O(1)

        // make sure that son is really using parent and only him
        debug_assert_eq!(son_node_users.len(), 1);
        debug_assert!(son_node_users.contains(parent_signal));

        // get parent node
        let mut parent_node = circuit.nodes.get(parent_signal).unwrap().to_owned(); // O(1)

        // remove both nodes
        circuit.nodes.remove(parent_signal); // O(1)
        circuit.nodes.remove(son_signal); // O(1)

        match (&son_node.node_type, &parent_node.node_type) {
            (CircuitNodeType::And(son), CircuitNodeType::And(parent)) => {
                let mut new_inputs = son.inputs.to_owned();
                let parent_inputs_filtered: Vec<Wire> = parent
                    .inputs
                    .iter()
                    .filter(|x| &x.signal() != son_signal)
                    .copied()
                    .collect();
                new_inputs.extend(parent_inputs_filtered.iter().copied());
                // assert_eq!(new_inputs.len(), son_inputs.len() + parent_inputs.len() - 1); // what if and gate takes son twice
                // assert_eq!(new_inputs.len(), son_inputs.len() + parent_inputs.len() - 1); // what if there are duplicates
                parent_node.node_type = CircuitNodeType::And(CircuitAnd { inputs: new_inputs });

                // for each input of son update users
                for input in son.inputs.iter() {
                    // let input_node = circuit.nodes.get_mut(&input.signal()).unwrap();
                    users.get_mut(&input.signal()).unwrap().remove(son_signal);
                    users
                        .get_mut(&input.signal())
                        .unwrap()
                        .insert(*parent_signal);
                    // input_node.users.remove(son_signal);
                    // input_node.users.insert(*parent_signal);
                    // input_node.internal_users.remove(son_signal);
                    // input_node.internal_users.insert(*parent_signal);
                }
            }
            _ => panic!("This should never happen"),
        }

        // add the changed input back
        circuit.nodes.insert(*parent_signal, parent_node); // O(1)

        // delete the and gate
        circuit.gates.remove(son_signal);
    }

    fn check_result(circuit: &Circuit) -> bool {
        let users = circuit.get_users_per_signal();
        for n in circuit.nodes.iter_items() {
            match &n.node_type {
                CircuitNodeType::ConstantZero => {}
                CircuitNodeType::Input => {}
                CircuitNodeType::Latch(_) => {}
                CircuitNodeType::And(a) => {
                    for i in a.inputs.iter() {
                        assert!(circuit.nodes.contains_key(&i.signal()));
                        if i.is_negated()
                            || a.inputs.contains(&!(*i))
                            || users.get(&i.signal()).unwrap().len() > 1
                            || circuit.important_signals.contains(&i.signal())
                        {
                            continue;
                        }
                        let input_node = circuit.nodes.get(&i.signal()).unwrap();

                        assert!(!matches!(input_node.node_type, CircuitNodeType::And(_)));
                    }
                }
                CircuitNodeType::GenericGate(_) => {}
            }
        }
        true
    }

    // ********************************************************************************************
    // API
    // ********************************************************************************************

    pub fn new(verbose: bool) -> Self {
        Self { verbose }
    }

    pub fn merge_and_gates(&mut self, circuit: &mut Circuit) -> SignalTransformation {
        // get signals in order
        let signals: Vec<Signal> = circuit.nodes.iter_sorted().collect();
        let iterations = signals.len();
        let one_percent_iterations = iterations / 100;
        let ten_percent_iterations = iterations / 10;
        let initial_number_of_nodes = circuit.nodes.len();

        let mut removed_signals = Vec::new();
        let mut users: UniqueSortedHashMap<Signal, UniqueSortedVec<Signal>> =
            circuit.get_users_per_signal();

        // go over all nodes and call pattern matcher.
        for (i, signal) in signals.iter().copied().enumerate() {
            // print progress
            if self.verbose && (ten_percent_iterations > 0) && i % ten_percent_iterations == 0 {
                println!(
                    "MERGE AND GATES, current number of nodes: {}, progress = {}%",
                    circuit.nodes.len(),
                    if one_percent_iterations > 0 {
                        i / one_percent_iterations
                    } else {
                        0
                    }
                );
            }

            // check if signal has been deleted in a previous iteration
            let mut was_pattern_matched = true;

            // once pattern matches on node keep looping until it no longer does
            while was_pattern_matched {
                debug_assert!(circuit.nodes.contains_key(&signal));
                was_pattern_matched = Self::merge_and_gate_with_one_of_its_fan_ins_if_possible(
                    circuit,
                    &mut removed_signals,
                    &mut users,
                    &signal,
                );
            }
        }

        // fix levels
        // circuit.fix_levels();

        debug_assert!(circuit.check().is_ok());
        debug_assert!(Self::check_result(circuit));

        if self.verbose {
            println!(
                "DONE! number of nodes before: {}, number of nodes after = {} ({}% reduction)",
                initial_number_of_nodes,
                circuit.nodes.len(),
                100.0
                    * (((initial_number_of_nodes - circuit.nodes.len()) as f32)
                        / (initial_number_of_nodes as f32)),
            );
        }

        SignalTransformation::SignalsRemovedBecauseTheyAreNotUsed(UniqueSortedVec::from_sequence(
            removed_signals,
        ))
    }
}

impl CircuitSimplifier for CircuitAndGateMerger {
    fn simplify(&mut self, circuit: &mut Circuit) -> SignalTransformation {
        self.merge_and_gates(circuit)
    }

    fn title(&self) -> String {
        "Merge and gates".to_string()
    }
}
