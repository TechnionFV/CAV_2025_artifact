// ************************************************************************************************
// use
// ************************************************************************************************

use super::Frames;
use crate::{
    engines::pdr::PropertyDirectedReachabilitySolver,
    formulas::{Clause, Cube},
    function,
    models::time_stats::function_timer::FunctionTimer,
    solvers::dd::DecisionDiagramManager,
};

// ************************************************************************************************
// impl
// ************************************************************************************************

impl<T: PropertyDirectedReachabilitySolver, D: DecisionDiagramManager> Frames<T, D> {
    /// Given a sequence of frames, and an index i, get the clauses of Ri
    ///
    /// R[i] = F[i] U F[i+1] U ... U F[n]
    pub fn get_cnf_of_frame(&self, i: usize) -> Vec<Clause> {
        let _timer = FunctionTimer::start(function!(), self.s.time_stats.clone());

        debug_assert!(self.frames[0].is_empty());
        let mut r_i = vec![];
        if i == 0 {
            r_i.extend(
                self.s
                    .fin_state
                    .borrow()
                    .get_initial_relation()
                    .to_cnf()
                    .iter()
                    .cloned(),
            );
        } else {
            for i in i..self.frames.len() {
                r_i.extend(self.frames[i].get_delta_clauses_cloned().iter().cloned());
            }
        }
        r_i
    }

    /// Returns whether a cube is blocked in the frame that you require
    pub fn is_cube_blocked_in_frame(&mut self, cube: &Cube, k: usize) -> bool {
        let _timer = FunctionTimer::start(function!(), self.s.time_stats.clone());

        // check syntactic submission (faster than SAT)
        let mut not_cube = !(cube.to_owned());
        not_cube = self
            .definition_library
            .ternary_propagation(not_cube)
            .unwrap();
        for d in k..self.frames.len() {
            for i in 0..self.frames[d].len() {
                let clause = self.frames[d].get_delta_at(i).clause().peek().peek();
                if clause.is_subset_of(not_cube.peek().peek()) {
                    debug_assert!(self.solvers.solve_is_cube_blocked(k, cube));
                    return true;
                }
            }
        }

        self.solvers.solve_is_cube_blocked(k, cube)
    }
}
