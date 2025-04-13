// ************************************************************************************************
// use
// ************************************************************************************************

// use rand::rngs::StdRng;
// use rand::{Rng, SeedableRng};

use crate::engines::pdr::pdr_stats::PDRStats;
use crate::engines::pdr::PropertyDirectedReachabilitySolver;
use crate::formulas::{Clause, Cube, Literal, Variable};
use crate::function;
use crate::models::time_stats::function_timer::FunctionTimer;
use crate::solvers::sat::incremental::SatResult;
use std::cell::RefCell;
// use rand::prelude::SliceRandom;
use std::iter;
use std::rc::Rc;

use super::Solvers;

// ************************************************************************************************
// impl
// ************************************************************************************************

impl<T: PropertyDirectedReachabilitySolver> Solvers<T> {
    // ********************************************************************************************
    // helper functions
    // ********************************************************************************************

    pub fn note_sat_call(pdr_stats: &Rc<RefCell<PDRStats>>, r: &SatResult) {
        // let elapsed = time_before.elapsed();

        pdr_stats.borrow_mut().note_sat_call(r);
    }

    fn do_ternary_simulation(
        &mut self,
        state: Cube,
        input: Cube,
        successor: Option<&Cube>,
    ) -> (Cube, Cube) {
        let _timer = FunctionTimer::start(function!(), self.s.time_stats.clone());

        let size_before = state.len();

        let r = match successor {
            Some(s) => self.s.fin_state.borrow_mut().simplify_predecessor(
                state,
                input,
                s,
                |w| self.s.weights.borrow().sort_literals_by_weights_fast(w),
                true,
            ),
            None => self.s.fin_state.borrow_mut().simplify_bad_cube(
                state.to_owned(),
                input.to_owned(),
                |w| self.s.weights.borrow().sort_literals_by_weights_fast(w),
                true,
            ),
        };

        self.s
            .pdr_stats
            .borrow_mut()
            .note_ternary_simulation(size_before, r.0.len());

        r
    }

    // fn ord_func<R: Rng>(i: usize, weights: &Weights, w: &mut Vec<Literal>, rng: &mut R) {
    //     debug_assert!(i < 1);
    //     if i == 0 {
    //         // do nothing
    //         weights.sort_literals_by_weights_fast(w)
    //     } else if i == 1 {
    //         // weights.sort_literals_by_step(w)
    //     } else {
    //         w.shuffle(rng)
    //     }
    // }

    fn are_all_literals_state_literals<'a, I>(&self, literals: I) -> bool
    where
        I: IntoIterator<Item = &'a Literal>,
    {
        let fin_state = self.s.fin_state.borrow();
        for l in literals {
            if !fin_state.is_state_literal(l) {
                return false;
            }
        }
        true
    }

    fn get_invariant_in_second_cycle(&mut self) -> Cube {
        if self.s.parameters.assume_constraints_in_second_cycle {
            let mut cube = self
                .s
                .fin_state
                .borrow()
                .get_invariant_constraints_on_internals()
                .to_owned();
            self.s.fin_state.borrow().add_tags_to_cube(&mut cube, 1);
            cube
        } else {
            Cube::new_true()
        }
    }

    fn get_property_negation(&mut self) -> Clause {
        let property = self
            .s
            .fin_state
            .borrow()
            .get_property_on_internals()
            .to_owned();
        !property
    }

    // ********************************************************************************************
    // Sat calls API
    // ********************************************************************************************

    /// get cube that satisfies Ri && !P
    pub fn get_bad_cube(&mut self, frame: usize) -> Option<(Cube, Cube)> {
        let _timer = FunctionTimer::start(function!(), self.s.time_stats.clone());

        let not_property = self.get_property_negation();
        if not_property.is_empty() {
            return Option::None;
        }

        let sat_response = self.solve(frame, iter::empty(), not_property.iter().copied());
        Self::note_sat_call(&self.s.pdr_stats, &sat_response);

        match sat_response {
            SatResult::Sat => {
                Option::Some({
                    let bad = self.s.fin_state.borrow().extract_bad_cube_from_solver(
                        |l| Self::val(&mut self.h, &self.var_map, frame, l),
                        true,
                    );
                    let input = self.s.fin_state.borrow().extract_input_from_solver(|l| {
                        Self::val(&mut self.h, &self.var_map, frame, l)
                    });

                    debug_assert!({
                        let implications = self
                            .s
                            .fin_state
                            .borrow_mut()
                            .get_implications_of_state_and_input(bad.to_owned(), input.to_owned());
                        self.s
                            .fin_state
                            .borrow()
                            .do_implications_violate_the_property(&implications)
                    });

                    // let size_before = bad.len();
                    // let r =

                    let r = self.do_ternary_simulation(bad, input, None);

                    r.to_owned()
                })
            }
            SatResult::UnSat => {
                // clear memory of solver
                Option::None
            }
        }
    }

    pub fn is_clause_guaranteed_after_transition(&mut self, frame: usize, clause: &Clause) -> bool {
        let _timer = FunctionTimer::start(function!(), self.s.time_stats.clone());

        let clause_tag = {
            let mut a = clause.to_owned();
            self.s.fin_state.borrow().add_tags_to_clause(&mut a, 1);
            a
        };
        let not_clause_tag = !clause_tag;
        let inv_tag = self.get_invariant_in_second_cycle();

        let sat_response = self.solve(
            frame,
            not_clause_tag.iter().chain(inv_tag.iter()).copied(),
            iter::empty(),
        );

        Self::note_sat_call(&self.s.pdr_stats, &sat_response);
        match sat_response {
            SatResult::Sat => false,
            SatResult::UnSat => true,
        }
    }

    pub fn is_clause_guaranteed_after_transition_if_assumed(
        &mut self,
        frame: usize,
        clause: &Clause,
    ) -> bool {
        let _timer = FunctionTimer::start(function!(), self.s.time_stats.clone());

        let clause_tag = {
            let mut a = clause.to_owned();
            self.s.fin_state.borrow().add_tags_to_clause(&mut a, 1);
            a
        };
        let not_clause_tag = !clause_tag;
        let inv_tag = self.get_invariant_in_second_cycle();

        let sat_response = self.solve(
            frame,
            not_clause_tag.iter().chain(inv_tag.iter()).copied(),
            clause.iter().copied(),
        );

        Self::note_sat_call(&self.s.pdr_stats, &sat_response);
        match sat_response {
            SatResult::Sat => false,
            SatResult::UnSat => true,
        }
    }

    pub fn get_state_in_clause_a_that_has_a_predecessor_not_in_clause_b(
        &mut self,
        frame: usize,
        clause_a: &Clause,
        clause_b: &Clause,
    ) -> Option<Cube> {
        let _timer = FunctionTimer::start(function!(), self.s.time_stats.clone());

        let clause_b_tag = {
            let mut a = clause_b.to_owned();
            self.s.fin_state.borrow().add_tags_to_clause(&mut a, 1);
            a
        };
        let not_clause_b_tag = !clause_b_tag;
        let inv_tag = self.get_invariant_in_second_cycle();

        let sat_response = self.solve(
            frame,
            not_clause_b_tag.iter().chain(inv_tag.iter()).copied(),
            clause_a.iter().copied(),
        );

        Self::note_sat_call(&self.s.pdr_stats, &sat_response);
        match sat_response {
            SatResult::Sat => {
                // let counter_example_to_induction =
                let state = self.s.fin_state.borrow().extract_variables_from_solver(
                    |l| Self::val(&mut self.h, &self.var_map, frame, l),
                    clause_a.iter().map(|l| l.variable()),
                );
                Some(state)
            }
            SatResult::UnSat => None,
        }
    }

    pub fn solve_is_cube_blocked(&mut self, frame: usize, cube: &Cube) -> bool {
        let _timer = FunctionTimer::start(function!(), self.s.time_stats.clone());

        let sat_response = self.solve(frame, cube.iter().copied(), iter::empty());
        Self::note_sat_call(&self.s.pdr_stats, &sat_response);

        match sat_response {
            SatResult::Sat => false,
            SatResult::UnSat => true,
        }
    }

    /// gets predecessor of cube.
    /// Returns:
    /// * Ok((cube, input)) if a predecessor was found.
    /// * Err(cube) if no predecessor was found, and the cube is a
    ///   subset of the input cube that was used to prove un-sat.
    pub fn get_predecessor_of_cube(
        &mut self,
        frame: usize,
        cube: &Cube,
    ) -> Result<(Cube, Cube), Cube> {
        let _timer = FunctionTimer::start(function!(), self.s.time_stats.clone());

        debug_assert!(self.are_all_literals_state_literals(cube.iter()));
        let cube_tag = {
            let mut a = cube.peek().peek().peek().to_owned();
            // sort assumptions by weights starting from highest to lowest
            // hopefully this leads the failed literals to have higher weights
            self.s
                .weights
                .borrow()
                .sort_literals_by_weights_fast(&mut a);
            a.reverse();
            for x in a.iter_mut() {
                self.s.fin_state.borrow().add_tags_to_literal(x, 1);
            }
            a
        };
        let extra_clause = !cube.to_owned();
        // these two are the same sat call Ri-1 ^ T ^ !s.cube ^ s.cube'
        let sat_response = self.solve(
            frame,
            cube_tag.iter().copied(),
            extra_clause.iter().copied(),
        );
        Self::note_sat_call(&self.s.pdr_stats, &sat_response);

        match sat_response {
            SatResult::Sat => {
                let predecessor = self
                    .s
                    .fin_state
                    .borrow()
                    .extract_predecessor_cube_from_solver(
                        |l| Self::val(&mut self.h, &self.var_map, frame, l),
                        cube,
                        true,
                    );
                let input =
                    self.s.fin_state.borrow().extract_input_from_solver(|l| {
                        Self::val(&mut self.h, &self.var_map, frame, l)
                    });

                debug_assert!(predecessor.iter().all(|l| self
                    .s
                    .fin_state
                    .borrow()
                    .is_state_literal(l)));
                debug_assert!(input
                    .iter()
                    .all(|l| self.s.fin_state.borrow().is_input_literal(l)));

                let r = self.do_ternary_simulation(predecessor, input, Some(cube));

                Ok(r)
            }
            SatResult::UnSat => {
                if true {
                    debug_assert_eq!(cube.len(), cube_tag.len());
                    let mut r = vec![];
                    for mut l_tag in cube_tag.into_iter() {
                        let failed = Self::failed(&mut self.h, &self.var_map, frame, l_tag);
                        if failed {
                            self.s
                                .fin_state
                                .borrow()
                                .add_tags_to_literal(&mut l_tag, -1);
                            r.push(l_tag);
                        }
                    }
                    let r = Cube::from_sequence(r);
                    debug_assert!(r.peek().peek().is_subset_of(cube.peek().peek()));
                    Err(r)
                } else {
                    Err(cube.to_owned())
                }
            }
        }
    }

    pub fn extract_variables_from_solver<I>(&mut self, frame: usize, literals: I) -> Cube
    where
        I: IntoIterator<Item = Variable>,
    {
        let _timer = FunctionTimer::start(function!(), self.s.time_stats.clone());
        self.s.fin_state.borrow().extract_variables_from_solver(
            |l| Self::val(&mut self.h, &self.var_map, frame, l),
            literals,
        )
    }

    // pub fn simplify(&mut self) {
    //     let _timer = FunctionTimer::start(function!(), self.s.time_stats.clone());
    //     self.frame_solver.simplify()
    // }
}
