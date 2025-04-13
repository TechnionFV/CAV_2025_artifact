// ************************************************************************************************
// use
// ************************************************************************************************

use rand::seq::SliceRandom;

use super::Frames;
use crate::{
    engines::pdr::{delta_element::DeltaElement, PropertyDirectedReachabilitySolver},
    formulas::{Clause, Cube, Literal},
    function,
    models::{
        definition::DefinitionFunction, time_stats::function_timer::FunctionTimer,
        SortedVecOfLiterals, UniqueSortedVec,
    },
    solvers::dd::DecisionDiagramManager,
};

// ************************************************************************************************
// impl
// ************************************************************************************************

impl<T: PropertyDirectedReachabilitySolver, D: DecisionDiagramManager> Frames<T, D> {
    // fn get_extension_literal_in_clause(&self, c: &Clause) -> Option<Literal> {
    //     let l = c.max_literal().unwrap();
    //     if self.definition_library.is_extension_variable(l.variable()) {
    //         Some(l)
    //     } else {
    //         None
    //     }
    // }

    fn get_candidate_clauses(&mut self, c: &Clause, el: &Literal) -> Vec<UniqueSortedVec<Literal>> {
        let d = self
            .definition_library
            .iter()
            .find(|d| d.variable == el.variable())
            .unwrap();

        let mut c1 = c.peek().peek().clone();
        let r = c1.remove(el);

        let a = d.inputs.peek().peek()[0];
        let b = d.inputs.peek().peek()[1];

        match d.function {
            DefinitionFunction::And => {
                if el.is_negated() {
                    unreachable!("The clause should have already been backwards propagated.");
                } else {
                    d.inputs
                        .iter()
                        .map(|x| {
                            let mut c = c1.clone();
                            c.insert(*x);
                            c
                        })
                        .collect()
                    // c1.insert(a);
                    // c2.insert(b);
                    // [c1, c2]
                }
            }
            DefinitionFunction::Xor => {
                debug_assert!(r);
                let mut c2 = c1.clone();

                debug_assert!(d.inputs.len() == 2);
                if el.is_negated() {
                    c1.insert(a);
                    c1.insert(!b);

                    c2.insert(!a);
                    c2.insert(b);
                    vec![c1, c2]
                } else {
                    c1.insert(a);
                    c1.insert(b);
                    c2.insert(!a);
                    c2.insert(!b);
                    vec![c1, c2]
                }
            }
        }
    }

    // fn choose_from_candidates(&mut self, candidates: Vec<Clause>, k: usize) -> Option<Clause> {
    //     if candidates.is_empty() {
    //         return None;
    //     }
    //     let candidate_literals = UniqueSortedVec::k_merge_consuming(
    //         candidates
    //             .iter()
    //             .map(|c| c.peek().peek().to_owned())
    //             .collect(),
    //     );
    //     let mut candidate_vars = UniqueSortedVec::from_sequence(
    //         candidate_literals.iter().map(|l| l.variable()).collect(),
    //     );
    //     let fs = self.fin_state.borrow();
    //     candidate_vars.perform_operation_on_each_value(|v| fs.add_tags_to_variable(v, 1));
    //     let mut assignment =
    //         self.frames[k].extract_variables_from_solver(candidate_vars.iter().copied());
    //     fs.add_tags_to_cube(&mut assignment, -1);
    //     let is_satisfied: Vec<bool> = candidates
    //         .iter()
    //         .map(|c| c.iter().any(|l| assignment.contains(l)))
    //         .collect();
    //     let count_true = is_satisfied.iter().filter(|x| **x).count();
    //     debug_assert!(count_true <= 1, "More the one candidate is possible.");
    //     if count_true == 0 {
    //         return None;
    //     }

    //     let mut clause_to_push =
    //         if let Some((i, _)) = is_satisfied.iter().enumerate().find(|(_, &x)| x) {
    //             candidates[i].to_owned()
    //         } else {
    //             return None;
    //         };

    //     // check that the clause is not a contradiction
    //     self.definition_library
    //         .ternary_propagation(clause_to_push.to_owned())?;
    //     if self
    //         .definition_library
    //         .is_clause_a_contradiction(&clause_to_push)
    //     {
    //         return None;
    //     }

    //     debug_assert!(self.definition_library.is_clause_valid(&clause_to_push));

    //     clause_to_push = self
    //         .definition_library
    //         .make_clause_canonical(clause_to_push);
    //     if self
    //         .find_clause_already_subsumed_in_frame_or_higher(&clause_to_push, k + 1)
    //         .is_some()
    //     {
    //         return None;
    //     }
    //     Some(clause_to_push)
    // }

    fn get_clause_if_not_contradiction(
        &mut self,
        clause: UniqueSortedVec<Literal>,
    ) -> Option<Clause> {
        if !SortedVecOfLiterals::are_variables_sorted_and_unique(clause.peek()) {
            return None;
        }

        let clause = Clause::from_ordered_set(clause.peek().to_owned());

        self.definition_library
            .ternary_propagation(clause.to_owned())?;

        if self
            .definition_library
            .solve_is_clause_a_contradiction(&clause)
        {
            return None;
        }

        debug_assert!(self.definition_library.solve_is_clause_valid(&clause));

        Some(clause)
    }

    fn find_next_fraction_to_try(
        &mut self,
        assignments: &[Cube],
        k: usize,
        clause: &Clause,
    ) -> Option<DeltaElement<D>> {
        let mut els: Vec<_> = clause
            .iter()
            .rev()
            .filter(|l| self.definition_library.is_extension_variable(l.variable()))
            .collect();

        if els.is_empty() {
            return None;
        }

        els.shuffle(&mut *self.s.rng.borrow_mut());

        for el in els {
            let candidates = self.get_candidate_clauses(clause, el);

            let is_sat: Vec<bool> = candidates
                .iter()
                .map(|ca| {
                    !Self::is_clause_propagation_disproved_by_assignments(assignments, ca.peek())
                })
                .collect();

            debug_assert!(is_sat.iter().filter(|x| !**x).count() >= 1);
            let i = is_sat.iter().position(|x| *x);

            // pick the first candidate that is satisfied
            // TODO: this code should be improved to pick multiple candidates
            let next = if let Some(i) = i {
                candidates[i].clone()
            } else {
                continue;
            };

            let next = match self.get_clause_if_not_contradiction(next) {
                Some(x) => x,
                None => continue,
            };

            if self.frames[k].was_fraction_already_propagated(&next) {
                // This fraction was already propagated
                continue;
            }

            let next = self.make_delta_element(next);

            // check that we did not previously add this clause (or if the clause is already in the higher frame)
            let iter_bigger_clauses = || {
                self.frames[k + 1..]
                    .iter()
                    .flat_map(|f| f.get_delta().iter())
            };

            if iter_bigger_clauses()
                .any(|de| Self::does_a_imply_b(de, &next, &mut self.definition_library, &self.s))
            {
                self.frames[k].mark_fraction_as_propagated(next.clause().to_owned());
                continue;
            }

            return Some(next);
        }

        None
    }

    fn get_assignment(&mut self, k: usize) -> Cube {
        let mut assignment = self.solvers.extract_variables_from_solver(
            k,
            self.s
                .fin_state
                .borrow()
                .get_state_variables()
                .iter()
                .copied()
                .chain(self.definition_library.iter().map(|d| d.variable))
                .map(|mut v| {
                    self.s.fin_state.borrow().add_tags_to_variable(&mut v, 1);
                    v
                }),
        );
        self.s
            .fin_state
            .borrow()
            .add_tags_to_cube(&mut assignment, -1);
        assignment
    }

    fn is_clause_propagation_disproved_by_assignments(
        assignments: &[Cube],
        clause: &[Literal],
    ) -> bool {
        assignments
            .iter()
            .any(|a| clause.iter().all(|cl| !a.contains(cl)))
    }

    fn is_guaranteed_cached(
        &mut self,
        assignments: &mut Vec<Cube>,
        k: usize,
        clause: &Clause,
    ) -> bool {
        // first check if some assignment already violates this clause
        if Self::is_clause_propagation_disproved_by_assignments(
            assignments,
            clause.peek().peek().peek(),
        ) {
            return false;
        }

        let r = self
            .solvers
            .is_clause_guaranteed_after_transition(k, clause);

        if !r {
            let assignment = self.get_assignment(k);
            assignments.push(assignment);
        }

        r
    }

    /// Propagate a clause, returns true if propagation / fractional propagation was successful.
    fn propagate_clause(
        &mut self,
        assignments: &mut Vec<Cube>,
        k: usize,
        clause_index: usize,
        fractional_propagation: bool,
    ) -> bool {
        let mut de: DeltaElement<D> = self.frames[k].get_delta_at(clause_index).to_owned();

        let mut i = 0;
        loop {
            i += 1;
            if self.is_guaranteed_cached(assignments, k, de.clause()) {
                if i > 1 {
                    self.s
                        .pdr_stats
                        .borrow_mut()
                        .increment_generic_count("Fractional Propagation Successful ");
                    self.frames[k].mark_fraction_as_propagated(de.clause().to_owned());
                } else {
                    self.s
                        .pdr_stats
                        .borrow_mut()
                        .increment_generic_count("Propagation Successful");
                }
                self.insert_clause_to_exact_frame(de, k + 1, true);
                return true;
            }

            if (!fractional_propagation) || self.definition_library.is_empty() {
                self.s
                    .pdr_stats
                    .borrow_mut()
                    .increment_generic_count("Propagation Unsuccessful");
                return false;
            }

            if let Some(next) = self.find_next_fraction_to_try(assignments, k, de.clause()) {
                de = next;
            } else {
                self.s
                    .pdr_stats
                    .borrow_mut()
                    .increment_generic_count("Fractional Propagation Unsuccessful");
                return false;
            }
        }
    }

    fn propagate_frame(
        &mut self,
        k: usize,
        early_stop: bool,
        fractional_propagation: bool,
    ) -> bool {
        if self.frame_hash_after_last_propagate[k] == self.frames[k].get_hash() {
            self.s
                .pdr_stats
                .borrow_mut()
                .increment_generic_count("Frame propagation skipped due to no changes");
            return early_stop && self.frames[k].is_empty();
        }

        let mut assignments = vec![];

        let mut clause_index = 0;
        while clause_index < self.frames[k].len() {
            let b4 = self.frames[k]
                .get_delta_at(clause_index)
                .clause()
                .to_owned();
            self.propagate_clause(&mut assignments, k, clause_index, fractional_propagation);
            let mut was_clause_removed = clause_index >= self.frames[k].len();
            was_clause_removed =
                was_clause_removed || self.frames[k].get_delta_at(clause_index).clause() != &b4;
            if !was_clause_removed {
                clause_index += 1;
            }
        }

        self.frame_hash_after_last_propagate[k] = self.frames[k].get_hash();

        early_stop && self.frames[k].is_empty()
    }

    pub(super) fn propagate_blocked_cubes_in_range(
        &mut self,
        start_frame: usize,
        limit: usize,
        early_stop: bool,
        fractional_propagation: bool,
    ) -> Option<usize> {
        let _timer = FunctionTimer::start(function!(), self.s.time_stats.clone());

        debug_assert!(1 <= start_frame);
        debug_assert!(start_frame <= limit);
        debug_assert!(limit <= (self.frames.len() - 2));

        (start_frame..limit).find(|&k| self.propagate_frame(k, early_stop, fractional_propagation))
    }

    /// Propagate cubes to the highest frame where they are guaranteed to hold
    pub fn propagate(&mut self) -> Option<usize> {
        let _timer = FunctionTimer::start(function!(), self.s.time_stats.clone());
        // let start_frame = 1; //self.lowest_frame_that_was_updated_since_last_propagate;
        let start_frame = if self.s.parameters.propagate_from_lowest_changed_frame {
            self.lowest_frame_that_was_updated_since_last_propagate
        } else {
            1
        };
        debug_assert!(start_frame > 0);
        debug_assert!(start_frame <= self.depth());
        let size_of_f_inf_before = self.frames.last().as_ref().unwrap().len();

        let r = self.propagate_blocked_cubes_in_range(
            start_frame,
            self.depth(),
            true,
            self.s.parameters.er_fp,
        );
        self.lowest_frame_that_was_updated_since_last_propagate = self.depth();

        // F infinity should not have changed at this point
        debug_assert_eq!(
            self.frames.last().as_ref().unwrap().len(),
            size_of_f_inf_before
        );

        debug_assert!(if self.s.parameters.propagate_from_lowest_changed_frame {true} else {
            let frames_before: Vec<_> = self
                .frames
                .iter()
                .map(|f| f.get_delta_clauses_cloned().clone())
                .collect();
            self.propagate_blocked_cubes_in_range(1, start_frame, false, self.s.parameters.er_fp);
            let frames_after: Vec<_> = self
                .frames
                .iter()
                .map(|f| f.get_delta_clauses_cloned().clone())
                .collect();
            frames_before == frames_after
        }, "This assert might fail when propagate_from_lowest_changed_frame = true, this is not a bug, this assert is just a check to find cases where this happens.");

        r
    }

    pub fn is_invariant_found(&self) -> Option<usize> {
        let start_frame = 1;
        let end_frame = self.depth() - 1;
        (start_frame..=end_frame).find(|&i| self.frames[i].is_empty())
    }
}
