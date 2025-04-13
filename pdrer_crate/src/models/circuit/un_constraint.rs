//! Performs structural hashing
//!
//! // ************************************************************************************************
// use
// ************************************************************************************************

use super::Circuit;
use crate::models::{circuit_builder::CircuitBuilder, TernaryValue, UniqueSortedVec};

// ************************************************************************************************
// impl
// ************************************************************************************************

// ************************************************************************************************
// impl
// ************************************************************************************************

impl Circuit {
    // ********************************************************************************************
    // helper functions
    // ********************************************************************************************

    // ********************************************************************************************
    // API
    // ********************************************************************************************

    /// Fold constraints into safety property.
    /// A constraint is violated if it is 0.
    /// Ensures that a violation of a constraint in any clock cycle guarantees safety
    /// for this cycle and all future cycles.
    /// This is done via the following:
    /// 1. Adding a
    pub fn un_constraint(&mut self, fold_outputs_too: bool) {
        // do nothing if there are no constraints
        if self.constraints.is_empty() {
            return;
        }

        // make builder
        let mut builder = CircuitBuilder::from_circuit(self);

        // add latch that signifies if constraints were violated in a previous cycle
        let latch_for_constraints_violated = builder.get_unused_signal();

        // add and gate that checks if
        // 1. any constraint is violated (is 0), or
        // 2. if latch_for_constraints_violated is 1
        // if so the and gate should return 0
        let constraints_and_gate = builder.get_unused_signal();
        let mut inputs = vec![latch_for_constraints_violated.wire(true)];
        inputs.extend(self.constraints.iter());
        builder
            .add_and_gate(constraints_and_gate, UniqueSortedVec::from_sequence(inputs))
            .unwrap();

        builder.add_latch(
            latch_for_constraints_violated,
            constraints_and_gate.wire(true),
            TernaryValue::False,
        );

        // remove constraints
        for c in self.constraints.iter() {
            builder.un_mark_as_invariant_constraint(*c);
        }

        // deny property violation if constraint are, or were, violated.
        for b in self.get_bad_wires().iter() {
            // first un-mark the bad wire
            builder.un_mark_as_bad(*b);
            // add an and gate that masks a 1 in a bad wire, and returns 1 only if the constraints_and_gate is 1
            let bad_and = builder.get_unused_signal();
            let inputs = vec![constraints_and_gate.wire(false), *b];
            builder
                .add_and_gate(bad_and, UniqueSortedVec::from_sequence(inputs))
                .unwrap();
            // mark the new signal as bad
            builder.mark_as_bad(bad_and.wire(false));
        }

        if fold_outputs_too {
            for o in self.get_output_wires().iter() {
                builder.un_mark_as_output(*o);
                let bad_and = builder.get_unused_signal();
                let inputs = vec![constraints_and_gate.wire(false), *o];
                builder
                    .add_and_gate(bad_and, UniqueSortedVec::from_sequence(inputs))
                    .unwrap();
                builder.mark_as_output(bad_and.wire(false));
            }
        }

        // build the new circuit
        let (c, _) = builder.build().unwrap();
        *self = c;
    }
}
