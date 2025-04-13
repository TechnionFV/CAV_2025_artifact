// ************************************************************************************************
// use
// ************************************************************************************************

use std::collections::BinaryHeap;

use crate::models::{
    circuit::{
        node_types::{CircuitNode, CircuitNodeType},
        Circuit,
    },
    Signal, UniqueSortedVec,
};

use super::cut_enumeration::Cut;

// ************************************************************************************************
// impl
// ************************************************************************************************

impl Circuit {
    // ********************************************************************************************
    // cone calculator
    // ********************************************************************************************

    /// Function that gets the cone of some wires.
    pub fn get_cone_of_influence_custom<I, F: FnMut(Signal)>(
        &self,
        desired_signals: I,
        mut call_back: F,
    ) where
        I: IntoIterator<Item = Signal>,
    {
        // let mut signals_that_were_already_passed: FxHashSet<Signal> = FxHashSet::new();
        // let mut cone_of_influence: FxHashMap<Signal, CircuitNode> = FxHashMap::new();
        let mut signals_to_visit = BinaryHeap::from_iter(desired_signals);
        // let mut current_wanted_wires = desired_wires.to_owned();
        while !signals_to_visit.is_empty() {
            let signal = signals_to_visit.pop().unwrap();

            // pop same signal in heap
            while signals_to_visit.peek().is_some() && signals_to_visit.peek().unwrap() == &signal {
                signals_to_visit.pop();
            }

            // get node
            let node = self.nodes.get(&signal).unwrap();

            // add it to the cone of influence
            // cone_of_influence.push(signal);
            call_back(signal);

            // see if it requires the addition of more nodes
            match &node.node_type {
                CircuitNodeType::ConstantZero => {}
                CircuitNodeType::Input => {}
                CircuitNodeType::Latch(_) => {}
                // mark inputs as wanted
                CircuitNodeType::And(a) => {
                    signals_to_visit.extend(a.inputs.iter().map(|i| i.signal()));
                }
                CircuitNodeType::GenericGate(g) => {
                    signals_to_visit.extend(g.truth_table.get_signals().iter());
                }
            };
        }
        // cone_of_influence.reverse();
        // UniqueSortedVec::from_ordered_set(cone_of_influence)
    }

    /// Function that gets the cone of some wires.
    pub fn get_cone_of_influence<I>(&self, desired_signals: I) -> UniqueSortedVec<Signal>
    where
        I: IntoIterator<Item = Signal>,
    {
        let mut cone_of_influence: Vec<Signal> = Vec::with_capacity(self.nodes.len());

        let call_back = |s: Signal| {
            cone_of_influence.push(s);
        };

        self.get_cone_of_influence_custom(desired_signals, call_back);
        cone_of_influence.reverse();
        cone_of_influence.shrink_to_fit();
        UniqueSortedVec::from_ordered_set(cone_of_influence)
    }

    pub fn get_cone_of_signal_bounded_by_cut(
        &self,
        signal: &Signal,
        cut: &Cut,
    ) -> UniqueSortedVec<Signal> {
        // get cone of influence until we reach the cut
        let mut signals = Vec::with_capacity(cut.len() << 1);
        signals.push(signal.to_owned());
        let mut i = 0;

        while i < signals.len() {
            let signal = signals[i];
            let node = self.nodes.get(&signal).unwrap();

            let mut handle_gate = |inputs: &[Signal]| {
                // stop if we reach the cut
                if !cut.contains(&signal) {
                    // add signals that we have not seen yet
                    for input in inputs.iter() {
                        if !signals.contains(input) {
                            signals.push(*input);
                        }
                    }
                }
            };

            match &node.node_type {
                CircuitNodeType::ConstantZero
                | CircuitNodeType::Input
                | CircuitNodeType::Latch { .. } => {
                    // debug_assert!(cut.contains(&signal))
                }
                CircuitNodeType::And(a) => {
                    handle_gate(&a.inputs.iter().map(|x| x.signal()).collect::<Vec<Signal>>())
                }
                CircuitNodeType::GenericGate(g) => handle_gate(g.truth_table.get_signals().peek()),
            }

            i += 1;
        }
        signals.reverse();
        UniqueSortedVec::from_sequence(signals)
    }

    // ********************************************************************************************
    // iterator
    // ********************************************************************************************

    pub fn iter_sorted(&self) -> impl Iterator<Item = Signal> + '_ {
        self.nodes.iter_sorted()
    }

    // ********************************************************************************************
    // node getters
    // ********************************************************************************************

    pub fn get_node(&self, signal: &Signal) -> Option<&CircuitNode> {
        self.nodes.get(signal)
    }

    /// Get the number of signals inside the circuit.
    pub fn get_number_of_nodes(&self) -> usize {
        self.nodes.len()
    }

    // this API can break the internal structure of a circuit
    // pub fn get_node_mut(&mut self, signal: &Signal) -> Option<&mut CircuitNode> {
    //     self.nodes.get_mut(signal)
    // }

    // ********************************************************************************************
    // gate getters
    // ********************************************************************************************
}
