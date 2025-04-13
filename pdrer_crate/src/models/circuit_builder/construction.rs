// ************************************************************************************************
// use
// ************************************************************************************************

use super::{Circuit, CircuitBuilder, Signal};
use crate::models::circuit::node_types::CircuitNodeType;

// ************************************************************************************************
// impl
// ************************************************************************************************

impl CircuitBuilder {
    // ********************************************************************************************
    // API
    // ********************************************************************************************

    pub fn new() -> Self {
        Self {
            signals: Default::default(),
            max_signal: Signal::GROUND,
            outputs: Vec::new(),
            invariant_constraints: Vec::new(),
            bad: Vec::new(),
        }
    }

    pub fn from_circuit(circuit: &Circuit) -> Self {
        debug_assert!(circuit.check().is_ok());
        let mut builder = Self::new();
        for signal in circuit.iter_sorted() {
            match &circuit.get_node(&signal).unwrap().node_type {
                CircuitNodeType::Input => {
                    builder.add_input(signal);
                }
                CircuitNodeType::Latch(l) => {
                    builder.add_latch(signal, l.input, l.initial);
                }
                CircuitNodeType::And(a) => {
                    builder.add_and_gate(signal, a.inputs.to_owned()).unwrap();
                }
                CircuitNodeType::GenericGate(g) => {
                    builder
                        .add_generic_gate(signal, g.truth_table.clone())
                        .unwrap();
                }
                CircuitNodeType::ConstantZero => {
                    builder.add_ground();
                }
            }
        }
        for output in circuit.get_output_wires().iter() {
            builder.mark_as_output(*output);
        }
        for bad in circuit.get_bad_wires().iter() {
            builder.mark_as_bad(*bad);
        }
        for constraint in circuit.get_invariant_constraint_wires().iter() {
            builder.mark_as_invariant_constraint(*constraint);
        }
        debug_assert!({
            let _ = builder.clone().build().unwrap();
            true
        });

        builder
    }
}

// ************************************************************************************************
// Default
// ************************************************************************************************

impl Default for CircuitBuilder {
    fn default() -> Self {
        Self::new()
    }
}
