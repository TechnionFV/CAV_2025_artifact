// ************************************************************************************************
// use
// ************************************************************************************************

use crate::models::{
    circuit::{node_types::CircuitNodeType, CircuitError},
    UniqueSortedHashMap,
};

use super::{Circuit, CircuitBuilder, Signal, TruthTable, UniqueSortedVec, Wire};

// ************************************************************************************************
// impl
// ************************************************************************************************

type M = UniqueSortedHashMap<Signal, Signal>;

// ************************************************************************************************
// impl
// ************************************************************************************************

impl CircuitBuilder {
    // ********************************************************************************************
    // helper functions
    // ********************************************************************************************

    /// Gets mapping from signals give to new signals.
    /// This is done to preserve:
    /// 1. Signal 0 is always ground
    /// 2. Inputs are always first
    /// 3. Latches are always second
    /// 4. Gates are always third
    fn get_signal_mapping(
        inputs: &UniqueSortedVec<Signal>,
        latches: &UniqueSortedVec<Signal>,
        gates: &UniqueSortedVec<Signal>,
    ) -> M {
        let max_signal = {
            let a = inputs.max();
            let b = latches.max();
            let c = gates.max();
            *[a, b, c]
                .into_iter()
                .flatten()
                .max()
                .unwrap_or(&Signal::GROUND)
        };
        let mut signal_mapping = M::new(max_signal);
        signal_mapping.insert(Signal::GROUND, Signal::GROUND);
        let mut max_signal = Signal::GROUND;
        for s in inputs.iter() {
            max_signal = Signal::new(max_signal.number() + 1);
            signal_mapping.insert(*s, max_signal);
        }
        for s in latches.iter() {
            max_signal = Signal::new(max_signal.number() + 1);
            signal_mapping.insert(*s, max_signal);
        }
        for s in gates.iter() {
            max_signal = Signal::new(max_signal.number() + 1);
            signal_mapping.insert(*s, max_signal);
        }

        signal_mapping
    }

    fn convert_wire_according_to_mapping(wire: Wire, mapping: &M) -> Option<Wire> {
        let signal = wire.signal();
        let new_signal = Self::convert_signal_according_to_mapping(signal, mapping);
        new_signal.map(|x| {
            if wire.is_negated() {
                x.wire(true)
            } else {
                x.wire(false)
            }
        })
    }

    fn convert_wires_according_to_mapping(wires: &[Wire], mapping: &M) -> Option<Vec<Wire>> {
        let mut new_wires = Vec::with_capacity(wires.len());
        for wire in wires.iter() {
            let wire = Self::convert_wire_according_to_mapping(*wire, mapping);
            match wire {
                Some(wire) => new_wires.push(wire),
                None => return None,
            }
        }
        Some(new_wires)
    }

    fn convert_signal_according_to_mapping(signal: Signal, mapping: &M) -> Option<Signal> {
        mapping.get(&signal).copied()
    }

    fn convert_signals_according_to_mapping(
        signals: &[Signal],
        mapping: &M,
    ) -> Option<Vec<Signal>> {
        let mut new_signals = Vec::with_capacity(signals.len());
        for signal in signals.iter() {
            let s = Self::convert_signal_according_to_mapping(*signal, mapping);
            match s {
                Some(s) => new_signals.push(s),
                None => return None,
            }
        }
        Some(new_signals)
    }

    fn unwrap_or_error<T, E>(result: Option<T>, e: E) -> Result<T, E> {
        match result {
            Some(r) => Ok(r),
            None => Err(e),
        }
    }

    // ********************************************************************************************
    // API
    // ********************************************************************************************

    pub fn build(self) -> Result<(Circuit, M), CircuitError> {
        let mut inputs = Vec::new();
        let mut latches = Vec::new();
        let mut gates = Vec::new();

        let mut latch_details = Vec::new();
        let mut and_details = Vec::new();
        let mut generic_details = Vec::new();

        for (signal, node) in self.signals {
            match node {
                CircuitNodeType::Input => inputs.push(signal),
                CircuitNodeType::Latch(l) => {
                    latches.push(signal);
                    latch_details.push((signal, l));
                }
                CircuitNodeType::And(a) => {
                    gates.push(signal);
                    and_details.push((signal, a))
                }
                CircuitNodeType::GenericGate(g) => {
                    gates.push(signal);
                    generic_details.push((signal, g))
                }
                CircuitNodeType::ConstantZero => {}
            }
        }

        let inputs = UniqueSortedVec::from_sequence(inputs);
        let latches = UniqueSortedVec::from_sequence(latches);
        let gates = UniqueSortedVec::from_sequence(gates);
        let mapping = Self::get_signal_mapping(&inputs, &latches, &gates);

        let inputs =
            Self::convert_signals_according_to_mapping(&inputs.unpack(), &mapping).unwrap();

        let outputs = Self::convert_wires_according_to_mapping(&self.outputs, &mapping);
        let outputs = Self::unwrap_or_error(outputs, CircuitError::OutputWireDoesNotExist)?;

        let bad = Self::convert_wires_according_to_mapping(&self.bad, &mapping);
        let bad = Self::unwrap_or_error(bad, CircuitError::BadWireDoesNotExist)?;

        let invariant_constraints =
            Self::convert_wires_according_to_mapping(&self.invariant_constraints, &mapping);
        let invariant_constraints = Self::unwrap_or_error(
            invariant_constraints,
            CircuitError::ConstraintWireDoesNotExist,
        )?;

        for (s, l) in latch_details.iter_mut() {
            let input = Self::convert_wire_according_to_mapping(l.input, &mapping);
            l.input = Self::unwrap_or_error(input, CircuitError::InputToLatchDoesNotExist)?;
            *s = Self::convert_signal_according_to_mapping(*s, &mapping).unwrap();
        }
        for (s, a) in and_details.iter_mut() {
            let inputs = Self::convert_wires_according_to_mapping(a.inputs.peek(), &mapping);
            let inputs = Self::unwrap_or_error(inputs, CircuitError::InputToAndGateDoesNotExist)?;
            a.inputs = UniqueSortedVec::from_sequence(inputs);
            *s = Self::convert_signal_according_to_mapping(*s, &mapping).unwrap();
        }
        for (s, g) in generic_details.iter_mut() {
            // let inputs = Self::convert_wires_according_to_mapping(g.inputs.peek(), &mapping);
            // let inputs =
            // Self::unwrap_or_error(inputs, CircuitError::InputToGenericGateDoesNotExist)?;
            // g.inputs = UniqueSortedVec::from_sequence(inputs);
            g.truth_table = TruthTable::new_truth_table_with_signals_renamed(&g.truth_table, |x| {
                Self::convert_signal_according_to_mapping(*x, &mapping).unwrap()
            });
            *s = Self::convert_signal_according_to_mapping(*s, &mapping).unwrap();
        }

        Circuit::new(
            UniqueSortedVec::from_sequence(inputs),
            latch_details,
            and_details,
            generic_details,
            UniqueSortedVec::from_sequence(outputs),
            UniqueSortedVec::from_sequence(bad),
            UniqueSortedVec::from_sequence(invariant_constraints),
        )
        .map(|c| (c, mapping))
    }
}
