// ************************************************************************************************
// use
// ************************************************************************************************

use super::{CircuitBuilder, Signal, TernaryValue, TruthTable, UniqueSortedVec, Wire};
use crate::models::circuit::{
    node_types::{CircuitAnd, CircuitGenericGate, CircuitLatch, CircuitNodeType},
    CircuitError,
};
use std::cmp::max;

// ************************************************************************************************
// impl
// ************************************************************************************************

impl CircuitBuilder {
    // ********************************************************************************************
    // helper functions
    // ********************************************************************************************

    fn update_max_signal(&mut self, signal: Signal) {
        self.max_signal = max(self.max_signal, signal);
    }

    // ********************************************************************************************
    // API
    // ********************************************************************************************

    pub fn get_unused_signal(&mut self) -> Signal {
        let r = Signal::new(self.max_signal.number() + 1);
        self.update_max_signal(r);
        r
    }

    pub fn add_ground(&mut self) {
        let signal = Signal::GROUND;
        self.signals.insert(signal, CircuitNodeType::ConstantZero);
        self.update_max_signal(signal);
    }

    pub fn add_input(&mut self, signal: Signal) {
        self.signals.insert(signal, CircuitNodeType::Input);
        self.update_max_signal(signal);
    }

    pub fn add_latch(&mut self, signal: Signal, input: Wire, initial: TernaryValue) {
        self.signals.insert(
            signal,
            CircuitNodeType::Latch(CircuitLatch { input, initial }),
        );
        self.update_max_signal(signal);
    }

    pub fn add_and_gate(
        &mut self,
        signal: Signal,
        inputs: UniqueSortedVec<Wire>,
    ) -> Result<(), CircuitError> {
        if inputs.is_empty() {
            return Err(CircuitError::AndGateWithNoInputs);
        }
        if inputs.iter().map(|w| w.signal()).any(|s| s > signal) {
            return Err(CircuitError::AndGateWithInputGreaterOrEqualToIt);
        }
        if inputs
            .iter()
            .map(|w| w.signal())
            .any(|s| !self.signals.contains_key(&s))
        {
            return Err(CircuitError::InputToAndGateDoesNotExist);
        }
        self.signals
            .insert(signal, CircuitNodeType::And(CircuitAnd { inputs }));
        self.update_max_signal(signal);
        Ok(())
    }

    pub fn add_generic_gate(
        &mut self,
        signal: Signal,
        // inputs: UniqueSortedVec<Wire>,
        tt: TruthTable,
    ) -> Result<(), CircuitError> {
        if tt.get_signals().is_empty() {
            return Err(CircuitError::GenericGateWithNoInput);
        }
        if tt.get_signals().iter().any(|s| *s > signal) {
            return Err(CircuitError::GenericGateWithInputGreaterOrEqualToIt);
        }
        // if tt
        //     .get_signals()
        //     .iter()
        //     .zip(tt.get_signals().peek())
        //     .any(|(a, b)| a != b)
        // {
        //     // truth table contradicts inputs
        //     return Err(CircuitError::TruthTableSignalsDoNotMatchGateInputs);
        // }
        if tt
            .get_signals()
            .iter()
            .any(|s| !self.signals.contains_key(s))
        {
            return Err(CircuitError::InputToGenericGateDoesNotExist);
        }
        self.signals.insert(
            signal,
            CircuitNodeType::GenericGate(CircuitGenericGate { truth_table: tt }),
        );
        self.update_max_signal(signal);
        Ok(())
    }

    pub fn remove_signal(&mut self, signal: Signal) {
        self.signals.remove(&signal);
    }

    pub fn mark_as_output(&mut self, wire: Wire) {
        self.outputs.push(wire);
    }

    pub fn un_mark_as_output(&mut self, wire: Wire) {
        self.outputs.retain(|&x| x != wire);
    }

    pub fn mark_as_invariant_constraint(&mut self, wire: Wire) {
        self.invariant_constraints.push(wire);
    }

    pub fn un_mark_as_invariant_constraint(&mut self, wire: Wire) {
        self.invariant_constraints.retain(|&x| x != wire);
    }

    pub fn mark_as_bad(&mut self, wire: Wire) {
        self.bad.push(wire);
    }

    pub fn un_mark_as_bad(&mut self, wire: Wire) {
        self.bad.retain(|&x| x != wire);
    }
}
