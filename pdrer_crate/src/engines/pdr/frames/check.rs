// ************************************************************************************************
// use
// ************************************************************************************************

use super::Frames;
use crate::{
    engines::pdr::PropertyDirectedReachabilitySolver,
    formulas::{Clause, CNF},
    function,
    models::{time_stats::function_timer::FunctionTimer, FiniteStateTransitionSystem, Utils},
    solvers::{
        dd::DecisionDiagramManager,
        sat::incremental::{CaDiCalSolver, SatResult},
    },
};
use std::iter;

// ************************************************************************************************
// impl
// ************************************************************************************************

impl<T: PropertyDirectedReachabilitySolver, D: DecisionDiagramManager> Frames<T, D> {
    // ********************************************************************************************
    // helper functions for regression check
    // ********************************************************************************************

    fn check_that_clauses_are_valid_under_definitions(&mut self) -> bool {
        for k in 0..self.len() {
            let frame = self.frames[k].get_delta_clauses();
            for c in frame.iter() {
                assert!(
                    self.definition_library.solve_is_clause_valid(c),
                    "Clause '{}' in frame {} is not valid under the definitions:\n{}",
                    c,
                    k,
                    self.definition_library
                );
            }
        }

        true
    }

    pub fn check_that_f_infinity_is_inductive(&self) -> bool {
        let _timer = FunctionTimer::start(function!(), self.s.time_stats.clone());

        assert!(self
            .s
            .fin_state
            .borrow()
            .is_cnf_semi_inductive_with_definitions::<CaDiCalSolver>(
                self.get_definitions(),
                &CNF::from_sequence(self.frames.last().unwrap().get_delta_clauses_cloned())
            ));
        assert!({
            let mut init = self.s.fin_state.borrow().construct_initial_cnf(true);
            init.append(FiniteStateTransitionSystem::definitions_to_cnf(
                self.get_definitions(),
            ));
            let f_inf = CNF::from_sequence(self.frames.last().unwrap().get_delta_clauses_cloned());
            Utils::does_a_imply_b::<CaDiCalSolver>(&init, &f_inf).unwrap_or(true)
        });

        true
    }

    fn check_that_the_sat_solvers_contain_the_frames(&mut self) -> bool {
        let _timer = FunctionTimer::start(function!(), self.s.time_stats.clone());

        for k in 0..self.len() {
            let r_i = self.get_cnf_of_frame(k);
            for c in r_i {
                let cube = !c;
                assert!(
                    self.solvers.solve_is_cube_blocked(k, &cube),
                    "The cube '{}' in frame {} is not implied by the sat solver.",
                    cube,
                    k
                );
            }
        }

        true
    }

    fn check_inductive_chain_1(&mut self) -> bool {
        let _timer = FunctionTimer::start(function!(), self.s.time_stats.clone());

        for k in 0..(self.len() - 1) {
            let mut solver = T::new(456789);

            let transition = self
                .s
                .fin_state
                .borrow()
                .construct_transition_cnf(false, false, true, true);
            for c in transition.iter() {
                solver.add_clause(c.iter().copied());
            }

            let mut definitions =
                FiniteStateTransitionSystem::definitions_to_cnf(self.get_definitions());
            for c in definitions.iter() {
                solver.add_clause(c.iter().copied());
            }

            self.s
                .fin_state
                .borrow()
                .add_tags_to_relation(&mut definitions, 1);
            for c in definitions.iter() {
                solver.add_clause(c.iter().copied());
            }

            let frame_k = self.get_cnf_of_frame(k);
            for c in frame_k.iter() {
                solver.add_clause(c.iter().copied());
            }

            let frame_k_plus_1 = self.get_cnf_of_frame(k + 1);
            for mut c in frame_k_plus_1 {
                self.s.fin_state.borrow().add_tags_to_clause(&mut c, 1);
                let r = solver.solve((!c).iter().copied(), iter::empty());
                assert!(r == SatResult::UnSat);
            }
        }

        true
    }

    fn check_inductive_chain_2(&mut self) -> bool {
        let _timer = FunctionTimer::start(function!(), self.s.time_stats.clone());

        for k in 0..(self.len() - 1) {
            let frame_k_plus_1 = self.get_cnf_of_frame(k + 1);
            for c in frame_k_plus_1 {
                assert!(self.solvers.is_clause_guaranteed_after_transition(k, &c));
            }
        }

        true
    }

    fn _is_clause_redundant(&self, clause: &Clause, k: usize) -> bool {
        let mut frame = self.get_cnf_of_frame(k);
        frame.retain(|c| c != clause);
        let mut solver = T::new(0);
        for c in frame.iter() {
            solver.add_clause(c.iter().copied());
        }
        match solver.solve((!clause.to_owned()).iter().copied(), iter::empty()) {
            SatResult::Sat => false,
            SatResult::UnSat => true,
        }
    }

    fn _check_that_no_clause_is_redundant(&mut self) -> bool {
        let _timer = FunctionTimer::start(function!(), self.s.time_stats.clone());

        for k in 1..self.len() {
            for c in self.get_cnf_of_frame(k).iter() {
                assert!(
                    !self._is_clause_redundant(c, k),
                    "The clause '{}' in frame {} is redundant. The frame:\n{}",
                    c,
                    k,
                    CNF::from_sequence(self.get_cnf_of_frame(k))
                );
            }
        }

        true
    }

    // ********************************************************************************************
    // helper functions for sanity check
    // ********************************************************************************************

    fn check_that_all_clauses_are_canonical(&self) -> bool {
        let _timer = FunctionTimer::start(function!(), self.s.time_stats.clone());

        for frame in self.frames.iter() {
            for c in frame.get_delta_clauses() {
                let canon = self.definition_library.make_clause_canonical(c.to_owned());
                assert_eq!(
                    c, &canon,
                    "The clause '{}' is not canonical. The canonical form is: {}",
                    c, canon
                );
            }
        }

        true
    }

    pub fn check_no_redundancy(&mut self) -> bool {
        let _timer = FunctionTimer::start(function!(), self.s.time_stats.clone());

        for i in 0..self.frames.len() {
            for j in i..self.frames.len() {
                for (x, ci) in self.frames[i].get_delta().iter().enumerate() {
                    for (y, cj) in self.frames[j].get_delta().iter().enumerate() {
                        if i == j && x == y {
                            continue;
                        }
                        assert!(!Self::does_a_imply_b(cj, ci, &mut self.definition_library, &self.s), "The clause '{}' in delta of frame {} implies\nthe clause '{}' in delta of frame {}", cj.clause(), j, ci.clause(), i);
                    }
                }
            }
        }

        true
    }

    fn check_that_all_clauses_are_satisfied_by_all_initial_states_without_sat_calls(&self) -> bool {
        let _timer = FunctionTimer::start(function!(), self.s.time_stats.clone());

        for i in 0..self.frames.len() {
            for c in self.frames[i].get_delta_clauses().iter() {
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

                assert!(match (r, constraints) {
                    (Some(true), _) => true,
                    (Some(false), false) => false,
                    _ => {
                        true
                    }
                });
            }
        }

        true
    }

    pub fn check_that_all_clauses_are_satisfied_by_all_initial_states(&mut self) -> bool {
        let _timer = FunctionTimer::start(function!(), self.s.time_stats.clone());

        for i in 0..self.frames.len() {
            for c in self.frames[i].get_delta_clauses_cloned() {
                assert!(
                    self.is_clause_satisfied_by_all_initial_states(&c),
                    "The clause '{}' in frame {} is not satisfied by all initial states. Initial states: {}",
                    c,
                    i,
                    self.s.fin_state.borrow().get_initial_relation()
                );
            }
        }

        true
    }

    // ********************************************************************************************
    // API
    // ********************************************************************************************

    /// Checks that the data structure guarantees some of the invariants.
    /// These checks call SAT solvers and can take a long time.
    pub fn regression_check(&mut self) -> bool {
        let _timer = FunctionTimer::start(function!(), self.s.time_stats.clone());

        assert!(self.sanity_check());
        assert!(self.check_no_redundancy());
        assert!(self.check_that_all_clauses_are_satisfied_by_all_initial_states());
        assert!(self.check_that_the_sat_solvers_contain_the_frames());
        assert!(self.check_inductive_chain_1());
        assert!(self.check_inductive_chain_2());
        assert!(self.check_that_f_infinity_is_inductive());
        assert!(self.check_that_clauses_are_valid_under_definitions());
        // assert!(self.check_that_no_clause_is_redundant());s
        true
    }

    /// Checks that the data structure guarantees some of the invariants.
    /// These checks do not call sat solvers.
    pub fn sanity_check(&self) -> bool {
        let _timer = FunctionTimer::start(function!(), self.s.time_stats.clone());

        assert_eq!(
            self.frames.len(),
            self.frame_hash_after_last_propagate.len()
        );
        assert!(self.frames[0].is_empty(), "The initial frame is not empty.");
        assert!(self.check_that_all_clauses_are_canonical(),);
        assert!(self.check_that_all_clauses_are_satisfied_by_all_initial_states_without_sat_calls());

        true
    }
}
