// ************************************************************************************************
// use
// ************************************************************************************************

use super::FiniteStateTransitionSystem;
use crate::formulas::Variable;
use crate::formulas::{Cube, Literal};
use crate::solvers::sat::stateless::Assignment;

// ************************************************************************************************
// impl
// ************************************************************************************************

impl FiniteStateTransitionSystem {
    // ********************************************************************************************
    // Assignment
    // ********************************************************************************************

    pub fn extract_state_from_assignment(&self, assignment: &Assignment) -> Cube {
        let mut literals = Vec::new();
        for state_lit_num in self.get_state_variables().iter() {
            let is_negated = !assignment.get_value(state_lit_num).unwrap();
            literals.push(state_lit_num.literal(is_negated))
        }

        Cube::from_sequence(literals)
    }

    pub fn extract_input_from_assignment(&self, assignment: &Assignment) -> Cube {
        let mut literals = Vec::new();
        for input_lit_num in self.get_input_variables().iter() {
            let is_negated = !assignment.get_value(input_lit_num).unwrap();
            literals.push(input_lit_num.literal(is_negated))
        }

        Cube::from_sequence(literals)
    }

    // ********************************************************************************************
    // IncrementalSatSolver
    // ********************************************************************************************

    pub fn extract_variables_from_solver<F: FnMut(Literal) -> Option<bool>, I>(
        &self,
        mut val: F,
        vars: I,
    ) -> Cube
    where
        I: IntoIterator<Item = Variable>,
    {
        let mut literals = Vec::new();

        for state_lit_num in vars {
            let literal = Literal::new(state_lit_num.to_owned());
            let is_true = val(literal);
            if let Some(it) = is_true {
                let is_negated = !it; // if (x=0, y=0, z=1) then clause = (!x ^ !y ^ z)
                literals.push(state_lit_num.literal(is_negated))
            }
        }

        Cube::from_ordered_set(literals)
    }

    /// Extracts a state variables in cone of the successor from the solver.
    /// successor should have no tags.
    pub fn extract_predecessor_cube_from_solver<F: FnMut(Literal) -> Option<bool>>(
        &self,
        val: F,
        successor: &Cube,
        add_invariant_constraint: bool,
    ) -> Cube {
        // first get which state variables are important
        let mut variables_in_cone_of_successor =
            self.get_state_variables_in_cone_of_state(successor);
        if add_invariant_constraint {
            variables_in_cone_of_successor = variables_in_cone_of_successor
                .merge(&self.state_variables_in_cone_of_invariant_constraint);
        }

        // // then get the predecessor and the input
        // let predecessor = self.extract_variables_from_solver(solver, vars_to_simulate_on);
        // let input = self.extract_input_from_solver(solver);
        // let vars =
        self.extract_variables_from_solver(val, variables_in_cone_of_successor.iter().copied())
    }

    /// Extracts a state variables in cone of safety from the solver
    pub fn extract_bad_cube_from_solver<F: FnMut(Literal) -> Option<bool>>(
        &self,
        val: F,
        add_invariant_constraint: bool,
    ) -> Cube {
        // let vars =
        self.extract_variables_from_solver(
            val,
            if add_invariant_constraint {
                &self.state_variables_in_cone_of_safety_and_invariant_constraint
            } else {
                &self.state_variables_in_cone_of_safety
            }
            .iter()
            .copied(),
        )
    }

    /// Extracts all state variables in the solver
    pub fn extract_state_from_solver<F: FnMut(Literal) -> Option<bool>>(&self, val: F) -> Cube {
        self.extract_variables_from_solver(val, self.state_variables.iter().copied())
    }

    /// Extracts all input variables in the solver
    pub fn extract_input_from_solver<F: FnMut(Literal) -> Option<bool>>(&self, val: F) -> Cube {
        self.extract_variables_from_solver(val, self.input_variables.iter().copied())
    }
}
