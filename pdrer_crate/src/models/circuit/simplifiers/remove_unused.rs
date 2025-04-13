// ************************************************************************************************
// use
// ************************************************************************************************

use fxhash::FxHashSet;

use crate::models::{
    circuit::{node_types::CircuitNodeType, CircuitSimplifier},
    signal_tracker::SignalTransformation,
    Circuit, Signal, UniqueSortedVec,
};

// ************************************************************************************************
// simplifier
// ************************************************************************************************

pub struct CircuitUnusedSignalRemover {}

// ************************************************************************************************
// impl
// ************************************************************************************************

impl CircuitUnusedSignalRemover {
    // ********************************************************************************************
    // helper functions
    // ********************************************************************************************

    fn get_signals_to_keep(circuit: &mut Circuit) -> UniqueSortedVec<Signal> {
        let mut wires_to_track = [
            circuit.bad.peek().to_vec(),
            circuit.outputs.peek().to_vec(),
            circuit.constraints.peek().to_vec(),
        ]
        .concat();
        wires_to_track.sort_unstable();
        wires_to_track.dedup();
        let mut signals_to_keep = UniqueSortedVec::new();

        while !wires_to_track.is_empty() {
            let new_cone = circuit.get_cone_of_influence(wires_to_track.iter().map(|w| w.signal()));
            let state_variables_in_new_cone = new_cone.intersect(&circuit.latches);
            let latch_signals_not_yet_seen = state_variables_in_new_cone.subtract(&signals_to_keep);
            wires_to_track = latch_signals_not_yet_seen
                .iter()
                .map(|s| match &circuit.get_node(s).unwrap().node_type {
                    CircuitNodeType::Latch(l) => l.input,
                    _ => unreachable!(),
                })
                .collect();
            wires_to_track.sort_unstable();
            wires_to_track.dedup();
            wires_to_track.retain(|w| !signals_to_keep.contains(&w.signal()));
            signals_to_keep = signals_to_keep.merge(&new_cone);
        }

        signals_to_keep
    }

    fn remove_signals_not_in_cone_of_important_signals(
        circuit: &mut Circuit,
    ) -> UniqueSortedVec<Signal> {
        let signals_to_keep = Self::get_signals_to_keep(circuit);
        let all_signals = UniqueSortedVec::from_ordered_set(circuit.nodes.iter_sorted().collect());
        let signals_to_remove = all_signals.subtract(&signals_to_keep);
        for signal in signals_to_remove.iter() {
            circuit.nodes.remove(signal);
            circuit.inputs.remove(signal);
            circuit.latches.remove(signal);
            circuit.gates.remove(signal);
            debug_assert!(!circuit.outputs.contains(&signal.wire(false)));
            debug_assert!(!circuit.outputs.contains(&signal.wire(true)));
            debug_assert!(!circuit.bad.contains(&signal.wire(false)));
            debug_assert!(!circuit.bad.contains(&signal.wire(true)));
            debug_assert!(!circuit.constraints.contains(&signal.wire(false)));
            debug_assert!(!circuit.constraints.contains(&signal.wire(true)));
        }
        let new_important_signals = circuit.recalculate_important_signals();
        circuit.important_signals = new_important_signals;
        signals_to_remove
    }

    fn check_results(circuit: &Circuit) -> bool {
        let mut used_signals = FxHashSet::default();

        for n in circuit.nodes.iter_items() {
            match &n.node_type {
                CircuitNodeType::ConstantZero => {}
                CircuitNodeType::Input => {}
                CircuitNodeType::Latch(l) => {
                    used_signals.insert(l.input.signal());
                }
                CircuitNodeType::And(a) => {
                    for i in a.inputs.iter() {
                        used_signals.insert(i.signal());
                    }
                }
                CircuitNodeType::GenericGate(g) => {
                    for s in g.truth_table.get_signals().iter() {
                        used_signals.insert(*s);
                    }
                }
            }
        }

        for w in [&circuit.outputs, &circuit.constraints, &circuit.bad] {
            for s in w.iter() {
                used_signals.insert(s.signal());
            }
        }

        // make sure all remaining nodes are used
        for s in circuit.nodes.iter_sorted() {
            if !used_signals.contains(&s) {
                panic!("Signal {} is not used but not removed.", s);
            }
        }

        true
    }

    // ********************************************************************************************
    // API
    // ********************************************************************************************

    pub fn new() -> Self {
        Self {}
    }

    pub fn remove_unused_signals(circuit: &mut Circuit) -> SignalTransformation {
        let r = Self::remove_signals_not_in_cone_of_important_signals(circuit);
        // self.remove_latches_only_used_by_self_loops();
        // circuit.fix_users();
        circuit.greatest_signal = circuit.nodes.max_key().unwrap();
        circuit.nodes.resize_with(circuit.greatest_signal);
        debug_assert!(circuit.check().is_ok());
        debug_assert!(Self::check_results(circuit));
        SignalTransformation::SignalsRemovedBecauseTheyAreNotUsed(r)
    }
}

impl CircuitSimplifier for CircuitUnusedSignalRemover {
    fn simplify(&mut self, circuit: &mut Circuit) -> SignalTransformation {
        Self::remove_unused_signals(circuit)
    }

    fn title(&self) -> String {
        "Remove unused signals".to_string()
    }
}
