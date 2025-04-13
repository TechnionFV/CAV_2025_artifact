// ************************************************************************************************
// use
// ************************************************************************************************

use super::Frames;
use crate::engines::pdr::PropertyDirectedReachabilitySolver;
use crate::formulas::{Clause, Literal};
use crate::function;
use crate::models::time_stats::function_timer::FunctionTimer;
use crate::solvers::dd::DecisionDiagramManager;

// ************************************************************************************************
// impl
// ************************************************************************************************

impl<T: PropertyDirectedReachabilitySolver, D: DecisionDiagramManager> Frames<T, D> {
    // ********************************************************************************************
    // helper functions
    // ********************************************************************************************

    // MIC method
    fn mic(&mut self, clause: &mut Vec<Literal>, k: usize, d: usize) {
        // iterate ove the literals of the original clause
        let literals = clause.clone();
        for l in literals {
            // clone clause and check if current literal is still in clause
            let mut clause_clone = Vec::with_capacity(clause.len());
            let mut found = false;
            for lc in clause.iter().copied() {
                if lc == l {
                    found = true;
                    continue;
                }
                clause_clone.push(lc);
            }
            if !found {
                // this literal is no longer in the clause
                // (it was removed by a previous ctg_down call)
                continue;
            }

            if self.ctg_down(&mut clause_clone, k, d) {
                *clause = clause_clone;
            }
        }
    }

    // Helper method `ctg_down`
    fn ctg_down(&mut self, clause: &mut Vec<Literal>, k: usize, d: usize) -> bool {
        if d > self.s.parameters.generalize_using_ctg_max_depth {
            let c = Clause::from_sequence(clause.clone());
            if !self.is_clause_satisfied_by_all_initial_states(&c) {
                return false;
            }
            if !self.is_clause_guaranteed_after_transition_if_assumed(&c, k) {
                return false;
            }
            return true;
        }

        let mut ctgs = 0;
        loop {
            let c = Clause::from_sequence(clause.clone());
            if !self.is_clause_satisfied_by_all_initial_states(&c) {
                return false;
            }
            let not_c = !c;
            match self.get_predecessor_of_cube(&not_c, k) {
                Err(_) => return true,
                Ok((s, _)) => {
                    if d > self.s.parameters.generalize_using_ctg_max_depth {
                        return false;
                    }

                    let not_s = !s.to_owned();
                    if ctgs < self.s.parameters.generalize_using_ctg_max_ctgs
                        && k > 0
                        && self.is_clause_satisfied_by_all_initial_states(&not_s)
                        && self.is_clause_guaranteed_after_transition_if_assumed(&not_s, k - 1)
                    {
                        ctgs += 1;

                        let mut j = k;
                        while j < self.depth() {
                            if !self.is_clause_guaranteed_after_transition_if_assumed(&not_s, j) {
                                break;
                            }
                            j += 1;
                        }

                        let mut clause_to_add = not_s.unpack().unpack().unpack();
                        self.s
                            .weights
                            .borrow()
                            .sort_literals_by_weights_fast(&mut clause_to_add);
                        self.mic(&mut clause_to_add, j - 1, d + 1);
                        let clause_to_add = Clause::from_sequence(clause_to_add);

                        let de = self.make_delta_element(clause_to_add);
                        self.s
                            .weights
                            .borrow_mut()
                            .update_weights_on_add(de.clause().iter());
                        self.insert_clause_to_exact_frame(de, j, false);
                    } else {
                        ctgs = 0;
                        *clause = clause
                            .iter()
                            .filter(|l| not_s.contains(l))
                            .copied()
                            .collect();
                    }
                }
            }
        }
    }

    // ********************************************************************************************
    // API
    // ********************************************************************************************

    /// Implements the generalization algorithm using the Counterexample to Generalization (CTG)
    /// technique.
    pub fn generalize_relative_to_frame_using_ctg(
        &mut self,
        mut clause: Vec<Literal>,
        k: usize,
    ) -> Clause {
        let _timer = FunctionTimer::start(function!(), self.s.time_stats.clone());

        debug_assert!(clause
            .iter()
            .all(|l| self.s.fin_state.borrow().is_state_literal(l)));

        self.mic(&mut clause, k, 1);

        Clause::from_sequence(clause)
    }
}
