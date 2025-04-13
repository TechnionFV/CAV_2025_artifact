// ************************************************************************************************
// use
// ************************************************************************************************

// use rand::rngs::StdRng;
// use rand::{Rng, SeedableRng};

use super::Frames;
use crate::engines::pdr::PropertyDirectedReachabilitySolver;
use crate::formulas::{Clause, Cube};
use crate::function;
use crate::models::time_stats::function_timer::FunctionTimer;
use crate::solvers::dd::DecisionDiagramManager;

// use rand::prelude::SliceRandom;

// ************************************************************************************************
// impl
// ************************************************************************************************

impl<T: PropertyDirectedReachabilitySolver, D: DecisionDiagramManager> Frames<T, D> {
    // ********************************************************************************************
    // helper functions
    // ********************************************************************************************

    // ********************************************************************************************
    // Sat calls API
    // ********************************************************************************************

    /// get cube that satisfies Ri && !P
    pub fn get_bad_cube(&mut self, k: usize) -> Option<(Cube, Cube)> {
        self.solvers.get_bad_cube(k)
    }

    // pub fn is_clause_guaranteed_after_transition(&mut self, clause: &Clause, k: usize) -> bool {
    //     self.frames[k].is_clause_guaranteed_after_transition(clause)
    // }

    pub fn is_clause_guaranteed_after_transition_if_assumed(
        &mut self,
        clause: &Clause,
        k: usize,
    ) -> bool {
        self.solvers
            .is_clause_guaranteed_after_transition_if_assumed(k, clause)
    }

    // pub fn get_state_in_clause_a_that_has_a_predecessor_not_in_clause_b(
    //     &mut self,
    //     clause_a: &Clause,
    //     clause_b: &Clause,
    //     k: usize,
    // ) -> Option<Cube> {
    //     self.frames[k]
    //         .get_state_in_clause_a_that_has_a_predecessor_not_in_clause_b(clause_a, clause_b)
    // }

    // pub fn solve_is_cube_blocked(&mut self, cube: &Cube, k: usize) -> bool {
    //     self.frames[k].solve_is_cube_blocked(cube)
    // }

    /// gets predecessor of cube.
    /// Returns:
    /// * Ok((cube, input)) if a predecessor was found.
    /// * Err(cube) if no predecessor was found, and the cube is a
    ///   subset of the input cube that was used to prove un-sat.
    pub fn get_predecessor_of_cube(&mut self, cube: &Cube, k: usize) -> Result<(Cube, Cube), Cube> {
        self.solvers.get_predecessor_of_cube(k, cube)
    }

    pub fn is_cube_initial(&mut self, c: &Cube) -> bool {
        let _timer = FunctionTimer::start(function!(), self.s.time_stats.clone());

        let r = self
            .s
            .fin_state
            .borrow()
            .is_cube_satisfied_by_some_initial_state(c);
        let constraints = !self
            .s
            .fin_state
            .borrow()
            .get_invariant_constraints_on_internals()
            .is_empty();

        match (r, constraints) {
            (Some(false), _) => false,
            (Some(true), false) => true,
            _ => {
                // cube contains non latches
                let r = self.solvers.solve_is_cube_blocked(0, c);
                !r
            }
        }
    }

    pub fn is_clause_satisfied_by_all_initial_states(&mut self, c: &Clause) -> bool {
        let _timer = FunctionTimer::start(function!(), self.s.time_stats.clone());

        let r = self
            .s
            .fin_state
            .borrow()
            .is_clause_satisfied_by_all_initial_states(c);
        let constraints = !self
            .s
            .fin_state
            .borrow()
            .get_invariant_constraints_on_internals()
            .is_empty();

        match (r, constraints) {
            (Some(true), _) => true,
            (Some(false), false) => false,
            _ => {
                // check if there is an initial state that does not satisfy the clause
                self.solvers.solve_is_cube_blocked(0, &(!c.to_owned()))
            }
        }
    }

    pub fn is_clause_inductive_relative_to_frame(&mut self, c: &Clause, k: usize) -> bool {
        if !self.is_clause_satisfied_by_all_initial_states(c) {
            return false;
        }
        if !self.is_clause_guaranteed_after_transition_if_assumed(c, k) {
            return false;
        }

        true
    }
}
