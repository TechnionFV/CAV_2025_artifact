// ************************************************************************************************
// use
// ************************************************************************************************

use crate::models::{signal::Signal, wire::Wire, Circuit, UniqueSortedVec};

use super::node_types::CircuitNodeType;

// ************************************************************************************************
// impl
// ************************************************************************************************

impl Circuit {
    pub fn get_input_signals(&self) -> &UniqueSortedVec<Signal> {
        &self.inputs
    }

    pub fn get_latch_signals(&self) -> &UniqueSortedVec<Signal> {
        &self.latches
    }

    pub fn get_gate_signals(&self) -> &UniqueSortedVec<Signal> {
        &self.gates
    }

    pub fn get_output_wires(&self) -> &UniqueSortedVec<Wire> {
        &self.outputs
    }

    pub fn get_bad_wires(&self) -> &UniqueSortedVec<Wire> {
        &self.bad
    }

    pub fn get_invariant_constraint_wires(&self) -> &UniqueSortedVec<Wire> {
        &self.constraints
    }

    pub fn get_highest_signal(&self) -> Signal {
        self.greatest_signal
    }

    pub(super) fn get_wires_that_feed_into_latches(&self) -> Vec<Wire> {
        self.get_latch_signals()
            .iter()
            .map(|s| self.get_node(s).unwrap())
            .map(|node| match &node.node_type {
                CircuitNodeType::Latch(l) => l.input,
                _ => panic!("This should never happen"),
            })
            .collect()
    }
}
