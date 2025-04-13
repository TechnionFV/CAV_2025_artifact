// ************************************************************************************************
// use
// ************************************************************************************************

use super::Frames;
use crate::engines::pdr::PropertyDirectedReachabilitySolver;
use crate::formulas::{Clause, Literal};
use crate::function;
use crate::models::definition::DefinitionFunction;
use crate::models::time_stats::function_timer::FunctionTimer;
use crate::models::TernaryValue;
use crate::solvers::dd::DecisionDiagramManager;

// ************************************************************************************************
// impl
// ************************************************************************************************

// type Frame = usize;

impl<T: PropertyDirectedReachabilitySolver, D: DecisionDiagramManager> Frames<T, D> {
    // ********************************************************************************************
    // helper functions
    // ********************************************************************************************

    fn generalize_relative_to_frame(
        &mut self,
        mut clause: Vec<Literal>,
        k: usize,
        min_clause_len: usize,
    ) -> Clause {
        let _timer = FunctionTimer::start(function!(), self.s.time_stats.clone());

        let mut i = 0;
        while i < clause.len() {
            if clause.len() < min_clause_len {
                break;
            }
            let removed_literal = clause.remove(i);
            let c = Clause::from_sequence(clause.to_owned());

            if self.is_clause_inductive_relative_to_frame(&c, k) {
                continue;
            }

            clause.insert(i, removed_literal);
            i += 1;
        }

        Clause::from_sequence(clause)
    }

    fn generalize_using_definitions_relative_to_frame(
        &mut self,
        mut clause: Clause,
        k: usize,
    ) -> Clause {
        let _timer = FunctionTimer::start(function!(), self.s.time_stats.clone());
        // let ternary_simulation_clause = d_lib.ternary_propagation(clause.clone()).unwrap();

        for d in self.definition_library.get_definitions().clone().iter() {
            // let is_negated = if ternary_simulation_clause.contains(&d.variable.literal(false)) {
            //     false
            // } else if ternary_simulation_clause.contains(&d.variable.literal(true)) {
            //     true
            // } else {
            //     continue;
            // };

            // let x = d.variable.literal(is_negated);
            // debug_assert!(ternary_simulation_clause.contains(&x));

            let (x, to_remove): (Literal, Vec<Literal>) = match d.function {
                DefinitionFunction::And => {
                    // (a \/ ...) ^ (b \/ ...) <=>
                    // (x \/ ...)
                    // where x = a ^ b
                    // debug_assert_eq!(d.inputs.len(), 2, "This assert may be violated in the future and this code should still work.");
                    let clause_contains_exactly_one_input_of_and_gate =
                        d.inputs.iter().filter(|l| clause.contains(l)).count() == 1;
                    if !clause_contains_exactly_one_input_of_and_gate {
                        continue;
                    }
                    let r = d
                        .inputs
                        .iter()
                        .filter(|l| clause.contains(l))
                        .copied()
                        .collect();
                    (d.variable.literal(false), r)
                }
                DefinitionFunction::Xor => {
                    // 4 cases:
                    // (a \/ b \/ ...) ^ (!a \/ !b \/ ...)  <=>  (x \/ ...)     where x = a XOR b
                    // (a \/ b \/ ...) ^ (!a \/ !b \/ ...)  <=>  (x \/ ...)     where x = !a XOR !b
                    // (a \/ b \/ ...) ^ (!a \/ !b \/ ...)  <=>  (!x \/ ...)    where x = !a XOR b
                    // (a \/ b \/ ...) ^ (!a \/ !b \/ ...)  <=>  (!x \/ ...)    where x = a XOR !b
                    if d.inputs
                        .iter()
                        .any(|l| !clause.contains_variable(&l.variable()))
                    {
                        continue;
                    }
                    debug_assert_eq!(d.inputs.len(), 2);
                    let a = d.inputs.peek().peek()[0];
                    let b = d.inputs.peek().peek()[1];
                    debug_assert!(
                        clause.contains_variable(&a.variable())
                            && clause.contains_variable(&b.variable())
                    );

                    let r: Vec<_> = clause
                        .iter()
                        .filter(|l| l.variable() == a.variable() || l.variable() == b.variable())
                        .copied()
                        .collect();
                    debug_assert!(r.len() == 2);

                    match (clause.contains(&a), clause.contains(&b)) {
                        (true, true) | (false, false) => (d.variable.literal(false), r),
                        (true, false) | (false, true) => (d.variable.literal(true), r),
                    }
                }
            };

            let mut clause_clone = clause.clone();
            for l in to_remove.iter() {
                debug_assert!(clause_clone.contains(l));
                clause_clone.remove(l);
            }

            debug_assert!(
                {
                    let mut clause_without_x = clause_clone.to_owned();
                    clause_without_x.remove(&x);
                    if clause_without_x.is_empty() {
                        true
                    } else {
                        !self.definition_library.sat_solve_implies(
                            &clause_without_x,
                            &Clause::from_ordered_set(vec![x]),
                        )
                    }
                },
                "The extension variable does not actually generalize anything."
            );

            debug_assert!(!clause_clone.contains(&x));
            clause_clone.insert(x);
            debug_assert!(self
                .definition_library
                .sat_solve_implies(&clause_clone, &clause));
            debug_assert!(
                clause_clone == self.definition_library.backwards(clause_clone.to_owned())
            );
            if self.is_clause_inductive_relative_to_frame(&clause_clone, k) {
                clause = clause_clone;
            }
        }

        clause
    }

    fn _generalize_using_definitions_relative_to_frame_new(
        &mut self,
        clause: Clause,
        k: usize,
    ) -> Clause {
        let _timer = FunctionTimer::start(function!(), self.s.time_stats.clone());
        let mut cube = !clause;
        // let ternary_simulation_clause = d_lib.ternary_propagation(clause.clone()).unwrap();

        for i in 0..self.definition_library.get_definitions().len() {
            let tri_sim_result = self
                .definition_library
                .ternary_propagation_using_bdds(i, &cube);

            if matches!(tri_sim_result, TernaryValue::X) {
                continue;
            }

            let literal_to_add = match tri_sim_result {
                TernaryValue::X => continue,
                TernaryValue::False => self.definition_library.at(i).variable.literal(true),
                TernaryValue::True => self.definition_library.at(i).variable.literal(false),
            };

            let variables_to_remove = self
                .definition_library
                .get_coi_of_variable(literal_to_add.variable());

            let mut cube_clone = cube.clone();
            cube_clone.retain(|l| !variables_to_remove.contains(&l.variable()));
            cube_clone.insert(literal_to_add);
            let clause = !cube_clone;

            if self.is_clause_inductive_relative_to_frame(&clause, k) {
                cube = !clause;
            }
        }

        !cube
    }

    // fn multi_generalize_relative_to_frame<R: Rng>(
    //     &mut self,
    //     rng: &mut R,
    //     clause: Clause,
    //     k: usize,
    //     continuously_insert: bool,
    // ) -> Vec<Clause> {
    //     if k == 0 {
    //         self.frames[0].multi_generalize_relative_to_frame(
    //             rng,
    //             &mut None,
    //             clause,
    //             continuously_insert,
    //         )
    //     } else {
    //         let (v1, elements_right) = self.frames.split_first_mut().unwrap();
    //         let v2 = &mut elements_right[k - 1];
    //         v2.multi_generalize_relative_to_frame(rng, &mut Some(v1), clause, continuously_insert)
    //     }
    // }

    // fn generalize_inductive_set(
    //     &mut self,
    //     clauses: Vec<Clause>,
    //     parameters: &PropertyDirectedReachabilityParameters,
    // ) -> Vec<Clause> {
    //     let _timer = FunctionTimer::start(function!(), self.s.time_stats.clone());
    //     // add all the clauses to F infinity
    //     for clause in clauses.iter() {
    //         self.add_unique_clause_to_frame(clause.to_owned(), self.frames.len() - 1, false);
    //     }

    //     let mut result = vec![];
    //     for clause in clauses.iter() {
    //         let r = self.generalize(clause.to_owned(), self.frames.len() - 1, parameters);
    //         result.push(r);
    //     }

    //     result
    // }

    // fn generalize_for_f_inf<R: Rng>(
    //     &mut self,
    //     rng: &mut R,
    //     clause: Clause,
    //     _parameters: &PropertyDirectedReachabilityParameters,
    // ) -> Vec<Clause> {
    //     let mut r = self.generalize(clause, self.frames.len() - 1, _parameters);
    //     r
    // }

    // ********************************************************************************************
    // API
    // ********************************************************************************************

    /// Reduce literals in the clause as long as the clause remains inductive
    pub fn generalize(&mut self, clause: Clause, k: usize) -> Clause {
        let _timer = FunctionTimer::start(function!(), self.s.time_stats.clone());

        debug_assert!(self.is_clause_guaranteed_after_transition_if_assumed(&clause, k));
        let original_cube = !clause.to_owned();
        let size_before = clause.len();

        // let cw = self.s.weights.clone();
        let sorted = |clause: Clause| {
            let mut lits: Vec<Literal> = clause.unpack().unpack().unpack();
            let (w, f) = (self.s.weights.borrow(), self.s.fin_state.borrow());

            lits.sort_unstable_by(
                |a, b| match (f.is_state_literal(a), f.is_state_literal(b)) {
                    (true, true) => w
                        .get_weight(&a.variable())
                        .total_cmp(&w.get_weight(&b.variable()))
                        .then(a.cmp(b)),
                    (true, false) => std::cmp::Ordering::Less,
                    (false, true) => std::cmp::Ordering::Greater,
                    (false, false) => a.cmp(b),
                },
            );
            debug_assert!(
                f.is_state_literal(&lits[0]) || lits.iter().all(|l| !f.is_state_literal(l))
            );
            lits
        };

        let mut generalized_clause = if self.s.parameters.generalize_using_ctg
            && clause
                .iter()
                .all(|l| self.s.fin_state.borrow().is_state_literal(l))
        {
            self.generalize_relative_to_frame_using_ctg(sorted(clause), k)
        } else {
            self.generalize_relative_to_frame(
                sorted(clause),
                k,
                self.s.parameters.minimum_clause_length_to_generalize,
            )
        };

        self.s.weights.borrow_mut().update_weights_on_add(
            generalized_clause
                .iter()
                .filter(|l| self.s.fin_state.borrow().is_state_literal(l)),
        );

        debug_assert!(self.is_clause_satisfied_by_all_initial_states(&generalized_clause));

        if self.s.parameters.er && self.s.parameters.er_generalization {
            generalized_clause =
                self.generalize_using_definitions_relative_to_frame(generalized_clause, k);
        }

        self.s
            .pdr_stats
            .borrow_mut()
            .note_generalization(size_before, generalized_clause.len());

        generalized_clause = self
            .definition_library
            .make_clause_canonical(generalized_clause.to_owned());
        // debug_assert_eq!(
        //     generalized_clause,
        //     self.definition_library
        //         .make_clause_canonical(generalized_clause.to_owned())
        // );
        debug_assert!(self.is_clause_guaranteed_after_transition_if_assumed(&generalized_clause, k));
        debug_assert!(self.is_clause_satisfied_by_all_initial_states(&generalized_clause));
        // if original_cube.len() == 15 && self.depth() == 7 {
        //     for i in k..self.depth() {
        //         let is_blocked = self.is_cube_blocked_in_frame(&original_cube, i);
        //         println!("i = {}, is_blocked = {}", i, is_blocked);
        //     }
        // }
        debug_assert!({
            let a = self.make_delta_element(generalized_clause.clone());
            let b = self.make_delta_element(!original_cube);
            Frames::does_a_imply_b(&a, &b, &mut self.definition_library, &self.s)
        });

        generalized_clause
    }

    pub(super) fn generalize_and_add_to_f_infinity(&mut self, clauses: Vec<Clause>) {
        let _timer = FunctionTimer::start(function!(), self.s.time_stats.clone());
        let k = self.frames.len() - 1;

        if false {
            println!("Inductive set found:");
            for c in clauses.iter() {
                println!("{}", c);
            }
        }

        // First add all the clauses to f_infinity
        for c in clauses.iter() {
            self.solvers.add_frame_clause(k, c);
            self.frames[k].increment_hash();
        }

        // Then generalize them
        for c in clauses {
            debug_assert!(self.sanity_check());
            let c = self.generalize(c.to_owned(), self.len() - 1);
            if false {
                println!("Clause {} ->\t{}", c, c);
            }
            debug_assert!(self.is_clause_satisfied_by_all_initial_states(&c));
            // if self.frames[k].get_delta_clauses().iter().any(|c| {
            //     c.peek()
            //         .peek()
            //         .is_subset_of(gh.representative().peek().peek())
            // }) {
            //     for c in gh.peek().iter() {
            //         self.mark_clause_added(c, k, false);
            //     }
            //     continue;
            // }
            let de = self.make_delta_element(c);
            if self.is_clause_redundant(&de, self.len() - 1) {
                continue;
            }

            self.insert_clause_to_exact_frame(de, self.frames.len() - 1, false);
            debug_assert!(self.sanity_check());
        }

        // self.analyzer.mark_f_inf_changed();
    }

    // pub fn generalize_f_infinity<R: Rng>(
    //     &mut self,
    //     rng: &mut R,
    //     parameters: &PropertyDirectedReachabilityParameters,
    // ) {
    //     let _timer = FunctionTimer::start(function!(), self.s.time_stats.clone());

    //     loop {
    //         let mut changed = false;
    //         let lemmas = self
    //             .frames
    //             .last()
    //             .unwrap()
    //             .get_unique_clauses()
    //             .to_owned()
    //             .unpack();

    //         for c in lemmas {
    //             let clauses = self.generalize_for_f_inf(rng, c, parameters);
    //             for g in clauses {
    //                 if self
    //                     .frames
    //                     .last()
    //                     .unwrap()
    //                     .get_unique_clauses()
    //                     .contains(&g)
    //                 {
    //                     continue;
    //                 } else {
    //                     changed = true;
    //                     self.add_unique_clause_to_frame(g, self.frames.len() - 1, false);
    //                 }
    //             }
    //         }

    //         if !changed {
    //             break;
    //         }
    //     }

    //     self.length_of_f_inf_after_the_last_time_it_was_generalized =
    //         self.frames.last().unwrap().len();
    // }

    // pub fn should_generalize_f_infinity(&self) -> bool {
    //     self.frames.last().unwrap().len()
    //         > self.length_of_f_inf_after_the_last_time_it_was_generalized
    // }
}
