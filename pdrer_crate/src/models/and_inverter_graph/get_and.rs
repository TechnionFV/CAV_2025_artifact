// ************************************************************************************************
// use
// ************************************************************************************************

use crate::models::{AndInverterGraph, Signal, Wire};

// ************************************************************************************************
// struct
// ************************************************************************************************

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub struct AndGate {
    // pub level: usize,
    pub in0: Wire,
    pub in1: Wire,
    pub out: Signal,
}

// ************************************************************************************************
// impl
// ************************************************************************************************

impl AndInverterGraph {
    // ********************************************************************************************
    // aig getting and gates
    // ********************************************************************************************

    pub fn get_all_and_gates(&self) -> Vec<AndGate> {
        let mut result = Vec::with_capacity(self.number_of_and_gates as usize);
        let from = 1 + self.number_of_inputs + self.number_of_latches;
        let to = from + self.number_of_and_gates;
        for signal in (from..to).map(Signal::new) {
            let node = &self.nodes.get(&signal).unwrap();
            // let and_out = signal.wire(false);
            let in0 = node.get_and_rhs0();
            let in1 = node.get_and_rhs1();

            let gate = AndGate {
                in0,
                in1,
                out: signal,
            };
            result.push(gate);
        }
        result
    }
}
