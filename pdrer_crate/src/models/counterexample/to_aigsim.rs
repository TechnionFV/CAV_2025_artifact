// ************************************************************************************************
// use
// ************************************************************************************************

use crate::{
    formulas::{Cube, Variable},
    models::{AndInverterGraph, Signal, SignalTracker, TernaryValue},
};

use super::Counterexample;

// ************************************************************************************************
// impl
// ************************************************************************************************

impl Counterexample {
    fn get_var_from_cube<F: Fn(Signal) -> Variable>(
        mapping: &SignalTracker,
        cube: &Cube,
        original_aig_signal: Signal,
        s2v: &F,
    ) -> TernaryValue {
        let final_signal = match mapping.get(original_aig_signal) {
            Some(s) => s,
            _ => return TernaryValue::X,
        };

        let l = s2v(final_signal).literal(false); // fin_state.convert_wire_to_literal(&final_signal.wire(false));
        if cube.contains(&l) {
            TernaryValue::True
        } else if cube.contains(&!l) {
            TernaryValue::False
        } else {
            TernaryValue::X
        }
    }

    /// Gets the string that should be given to aigsim to reproduce the counterexample.
    /// <https://github.com/arminbiere/aiger.git>
    pub fn get_aigsim<F: Fn(Signal) -> Variable>(
        &self,
        mapping: &SignalTracker,
        aig: &AndInverterGraph,
        ground_x: bool,
        s2v: F,
    ) -> String {
        let latches = aig.get_latch_information();
        let inputs = aig.get_input_signals();
        let mut rows = Vec::with_capacity(self.inputs.len() + 4);
        rows.push("1".to_string());
        rows.push("b0".to_string());
        let x = if ground_x { '0' } else { 'x' };

        let mut first_row = String::with_capacity(latches.len());
        for latch in latches.iter() {
            let c = match latch.initial {
                TernaryValue::X => {
                    match Self::get_var_from_cube(mapping, &self.initial_cube, latch.output, &s2v) {
                        TernaryValue::X => x,
                        TernaryValue::False => '0',
                        TernaryValue::True => '1',
                    }
                }
                TernaryValue::False => {
                    assert_ne!(
                        Self::get_var_from_cube(mapping, &self.initial_cube, latch.output, &s2v),
                        TernaryValue::True
                    );
                    '0'
                }
                TernaryValue::True => {
                    assert_ne!(
                        Self::get_var_from_cube(mapping, &self.initial_cube, latch.output, &s2v),
                        TernaryValue::False
                    );

                    '1'
                }
            };
            first_row.push(c);
        }
        rows.push(first_row);

        for input in self.inputs.iter() {
            let mut row = String::with_capacity(latches.len());
            for input_signal in inputs.iter() {
                let c = match Self::get_var_from_cube(mapping, input, *input_signal, &s2v) {
                    TernaryValue::X => x,
                    TernaryValue::False => '0',
                    TernaryValue::True => '1',
                };
                row.push(c);
            }
            rows.push(row);
        }
        rows.push(".\n".to_string());

        rows.join("\n")
    }
}
