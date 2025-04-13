//! This API is made private so no algorithms can bypass FiniteStateTransitionSystem and access
//! circuit directly.

// ************************************************************************************************
// use
// ************************************************************************************************

use super::FiniteStateTransitionSystem;
use crate::formulas::Variable;
use crate::formulas::{Cube, Literal};
use crate::models::{Signal, TernaryValue, Wire};

// ************************************************************************************************
// impl
// ************************************************************************************************

impl FiniteStateTransitionSystem {
    pub fn convert_signal_to_variable(&self, signal: Signal) -> Variable {
        (self.signal_to_variable)(signal)
    }

    pub fn convert_variable_to_signal(&self, variable: Variable) -> Signal {
        (self.variable_to_signal)(variable)
    }

    pub fn convert_wire_to_literal(&self, wire: &Wire) -> Literal {
        let variable = self.convert_signal_to_variable(wire.signal());
        variable.literal(wire.is_negated())
    }

    pub fn convert_literal_to_wire(&self, literal: &Literal) -> Wire {
        let signal = self.convert_variable_to_signal(literal.variable());
        signal.wire(literal.is_negated())
    }

    pub fn convert_literals_to_signal_and_value_pairs<'a, I>(
        &self,
        cube: I,
        size_hint: usize,
    ) -> Vec<(Signal, TernaryValue)>
    where
        I: IntoIterator<Item = &'a Literal>,
    {
        let mut result = Vec::with_capacity(size_hint);

        // insert cube literals and inputs into state
        for literal in cube {
            result.push((
                self.convert_literal_to_wire(literal).signal(),
                if literal.is_negated() {
                    TernaryValue::False
                } else {
                    TernaryValue::True
                },
            ));
        }

        result
    }

    pub fn convert_signal_and_value_pairs_to_cube(&self, pairs: &[(Signal, TernaryValue)]) -> Cube {
        let mut result = Vec::with_capacity(pairs.len());

        // insert cube literals and inputs into state
        for (signal, value) in pairs.iter() {
            match value {
                TernaryValue::True => {
                    let wire = signal.wire(false);
                    let literal = self.convert_wire_to_literal(&wire);
                    result.push(literal);
                }
                TernaryValue::False => {
                    let wire = signal.wire(true);
                    let literal = self.convert_wire_to_literal(&wire);
                    result.push(literal);
                }
                TernaryValue::X => {}
            }
        }

        Cube::from_ordered_set(result)
    }
}
