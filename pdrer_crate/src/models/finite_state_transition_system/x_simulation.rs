// ************************************************************************************************
// use
// ************************************************************************************************

use super::FiniteStateTransitionSystem;
use crate::formulas::{Cube, Literal};
use crate::models::{Signal, TernaryValue};

// ************************************************************************************************
// impl
// ************************************************************************************************

impl FiniteStateTransitionSystem {
    // ********************************************************************************************
    // helper functions
    // ********************************************************************************************

    fn simplify_using_ternary_simulation(
        &mut self,
        input: &Cube,
        cube: &[Literal],
        signals_to_preserve: &[Signal],
    ) -> Cube {
        let static_signals =
            self.convert_literals_to_signal_and_value_pairs(input.iter(), input.len());
        let x = self.convert_literals_to_signal_and_value_pairs(cube.iter(), cube.len());
        let mut signals_we_want_to_drop = self.circuit.automatic_simulation_for_dropping_signals(
            &x,
            &static_signals,
            signals_to_preserve,
        );
        signals_we_want_to_drop.sort_unstable_by_key(|x| x.0);
        self.convert_signal_and_value_pairs_to_cube(&signals_we_want_to_drop)
    }

    fn get_signals_to_preserve_for_bad_cubes(&self, maintain_constraints: bool) -> Vec<Signal> {
        let mut signals_to_preserve: Vec<Signal> = self
            .property_on_internals
            .iter()
            .map(|s| s.variable())
            .map(|v| self.convert_variable_to_signal(v))
            .collect();

        if maintain_constraints {
            signals_to_preserve.extend(
                self.invariant_constraints_on_internals
                    .iter()
                    .map(|s| s.variable())
                    .map(|v| self.convert_variable_to_signal(v)),
            )
        };

        signals_to_preserve.sort_unstable();
        signals_to_preserve.dedup();
        signals_to_preserve
    }

    fn get_signals_to_preserve_for_predecessor_cubes(
        &self,
        successor: &Cube,
        maintain_constraints: bool,
    ) -> Vec<Signal> {
        let mut signals_to_preserve: Vec<Signal> = successor
            .iter()
            .map(|l| l.variable())
            .map(|v| {
                self.get_internal_variable_in_cone_of_state_variable(&v)
                    .unwrap()
                    .0
            })
            .map(|v| self.convert_variable_to_signal(v))
            .collect();

        if maintain_constraints {
            signals_to_preserve.extend(
                self.invariant_constraints_on_internals
                    .iter()
                    .map(|l| self.convert_variable_to_signal(l.variable())),
            )
        }

        signals_to_preserve.sort_unstable();
        signals_to_preserve.dedup();
        debug_assert!(
            signals_to_preserve.len()
                <= self.invariant_constraints_on_internals.len() + successor.len()
        );
        signals_to_preserve
    }

    // ********************************************************************************************
    // aig api functions
    // ********************************************************************************************

    /// Uses ternary simulation to simplify a bad cube
    pub fn simplify_bad_cube<F>(
        &mut self,
        bad: Cube,
        input: Cube,
        ordering_function: F,
        maintain_constraints: bool,
    ) -> (Cube, Cube)
    where
        F: FnOnce(&mut Vec<Literal>),
    {
        // extract the literals
        let mut bad_vec = bad.unpack().unpack().unpack();

        // order using the function given
        ordering_function(&mut bad_vec);

        // signals to simulate on
        let signals_to_preserve = self.get_signals_to_preserve_for_bad_cubes(maintain_constraints);

        // ternary simulation
        let simplified_bad =
            self.simplify_using_ternary_simulation(&input, &bad_vec, &signals_to_preserve);

        (simplified_bad, input)
    }

    pub fn simplify_predecessor<F>(
        &mut self,
        predecessor: Cube,
        input: Cube,
        successor: &Cube,
        ordering_function: F,
        maintain_constraints: bool,
    ) -> (Cube, Cube)
    where
        F: FnOnce(&mut Vec<Literal>),
    {
        // extract the literals
        let mut predecessor_vec = predecessor.unpack().unpack().unpack();

        // order using the function given
        ordering_function(&mut predecessor_vec);

        // signals to simulate on
        let signals_to_preserve =
            self.get_signals_to_preserve_for_predecessor_cubes(successor, maintain_constraints);

        // ternary simulation
        let simp_pred =
            self.simplify_using_ternary_simulation(&input, &predecessor_vec, &signals_to_preserve);

        (simp_pred, input)
    }

    // ********************************************************************************************
    // internal signals
    // ********************************************************************************************

    pub fn get_state_implications(&mut self, literals: &[Literal], ignore_constants: bool) -> Cube {
        let initial_signals =
            self.convert_literals_to_signal_and_value_pairs(literals, literals.len());

        // get constant values before
        self.circuit.full_simulation([]);
        let constant_values: Vec<TernaryValue> = self
            .circuit
            .get_signal_simulation_values(self.signals_to_implicate_on.peek());

        // get values after putting the literals into the state
        self.circuit.full_simulation(initial_signals);
        let values_after_simulation = self
            .circuit
            .get_signal_simulation_values(self.signals_to_implicate_on.peek());

        // get the implications
        let signals_and_values: Vec<(Signal, TernaryValue)> = self
            .signals_to_implicate_on
            .iter()
            .zip(values_after_simulation.iter())
            .enumerate()
            .filter(|(i, _)| {
                let was_x = constant_values[*i] == TernaryValue::X;
                let is_no_longer_x = values_after_simulation[*i] != TernaryValue::X;
                !ignore_constants || (was_x && is_no_longer_x)
            })
            .filter(|(_, (s, _))| !s.is_constant())
            .map(|(_, (s, v))| (s.to_owned(), v.to_owned()))
            .collect();

        // convert to cube

        self.convert_signal_and_value_pairs_to_cube(&signals_and_values)
    }

    pub fn print_ternary_simulation_time_stats(&self) {
        self.circuit.print_time_stats();
    }
}
