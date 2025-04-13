// ************************************************************************************************
// use
// ************************************************************************************************

use super::proof_obligations::ProofObligation;
use super::{
    PropertyDirectedReachability, PropertyDirectedReachabilityProofError,
    PropertyDirectedReachabilitySolver,
};
use crate::formulas::Clause;
use crate::formulas::Cube;
use crate::function;
use crate::models::time_stats::function_timer::FunctionTimer;
use crate::models::Counterexample;
use crate::solvers::dd::DecisionDiagramManager;

// ************************************************************************************************
// impl
// ************************************************************************************************

impl<T: PropertyDirectedReachabilitySolver, D: DecisionDiagramManager>
    PropertyDirectedReachability<T, D>
{
    pub fn add_clause_to_frame_at_least(&mut self, clause: Clause, k: usize) -> usize {
        let _timer = FunctionTimer::start(function!(), self.s.time_stats.clone());
        if self.s.parameters.should_print_clauses_when_added {
            println!("Adding clause '{}' at index = {}", clause, k);
        }
        if self.s.parameters.should_print_clauses_when_added_as_ternary {
            println!(
                "Adding clause at index {}:\t{}",
                k,
                self.s
                    .fin_state
                    .borrow()
                    .represent_cube_on_states(clause.peek())
            );
        }
        debug_assert!(k > 0);

        // self.weights_of_literals_in_clause
        //     .update_weights_on_add(clause.iter());
        self.frames
            .insert_clause_to_highest_frame_possible(clause, k)
    }

    pub fn add_to_f_inf(&mut self, clause: Clause) {
        self.add_clause_to_frame_at_least(clause, self.frames.len() - 1);
    }

    pub fn make_simplified_cube_non_initial(
        &mut self,
        mut simplified_cube: Cube,
        original: &Cube,
    ) -> Cube {
        let _timer = FunctionTimer::start(function!(), self.s.time_stats.clone());

        if self.frames.is_cube_initial(&simplified_cube) {
            // need to add back some literal that negates init
            let mut cube_literals = self
                .s
                .weights
                .borrow()
                .get_cube_literals_sorted_by_weights(original);
            cube_literals.reverse();
            for lit in cube_literals {
                let tmp_cube = Cube::from_ordered_set(vec![lit]);
                if !self.frames.is_cube_initial(&tmp_cube) {
                    // found literal that contradicts init
                    let mut simplified_cube_literals = simplified_cube.unpack().unpack();
                    simplified_cube_literals.insert(lit);
                    simplified_cube = Cube::from_ordered_set(simplified_cube_literals.unpack());
                    // simplified_cube.push(lit.to_owned());
                    break;
                }
            }
        }
        simplified_cube
    }

    // fn call_lic(&mut self, cube: Cube) -> bool {
    //     let _timer = FunctionTimer::start(function!(), self.s.time_stats.clone());
    //     if self.s.parameters.perform_lic_analysis {
    //         let (calls, successful) = self.s.pdr_stats.borrow().get_lic_stats();
    //         let is_in_stage_1 = calls < self.s.parameters.lic_calls_in_stage_1;
    //         let success_rate = successful as f64 / calls as f64;
    //         let was_successful_enough =
    //             self.s.parameters.lic_min_success_rate_in_stage_2 < success_rate;
    //         // if LIC is too successful, don't call it, we will probably get inductive clauses anyway
    //         let is_not_too_successful =
    //             success_rate < self.s.parameters.lic_max_success_rate_in_stage_2;
    //         if is_in_stage_1 || (was_successful_enough && is_not_too_successful) {
    //             self.frames.lic_analysis(!cube)
    //         } else {
    //             false
    //         }
    //     } else {
    //         false
    //     }
    // }

    fn no_predecessor_of_cube(&mut self, po: ProofObligation, simplified_cube: Cube) {
        let _timer = FunctionTimer::start(function!(), self.s.time_stats.clone());

        debug_assert!(!self.frames.is_cube_blocked_in_frame(&po.cube, po.frame));
        debug_assert!(!self
            .frames
            .is_cube_blocked_in_frame(&simplified_cube, po.frame));

        let cube = self.make_simplified_cube_non_initial(simplified_cube, &po.cube);
        debug_assert!(!self.frames.is_cube_blocked_in_frame(&cube, po.frame));

        let clause = !cube;

        // if self.call_lic(po.cube.to_owned()) {
        //     debug_assert!(self.frames.sanity_check());
        //     return;
        // }

        debug_assert!(self
            .frames
            .is_clause_satisfied_by_all_initial_states(&clause));

        let h = if self.s.parameters.generalize_using_ctg {
            self.frames.iter().map(|f| f.get_hash()).collect()
        } else {
            vec![]
        };

        let gh = self.frames.generalize(clause, po.frame - 1);

        debug_assert!(self.frames.is_clause_satisfied_by_all_initial_states(&gh));
        // debug_assert!(self.frames.check());
        // debug_assert!(self.frames.regression_check());
        let mut max_k = self.add_clause_to_frame_at_least(gh, po.frame);
        // debug_assert!(self.frames.regression_check());

        debug_assert!(max_k <= self.frames.depth());

        if self.s.parameters.generalize_using_ctg {
            let start = max_k + 1;
            #[allow(clippy::needless_range_loop)]
            for i in start..=self.depth() {
                if h[i] == self.frames.at(i).get_hash() {
                    break;
                }
                if self.frames.is_cube_blocked_in_frame(&po.cube, i) {
                    debug_assert_eq!(max_k + 1, i);
                    max_k += 1;
                } else {
                    break;
                }
            }
        }

        if max_k < self.frames.depth() {
            let mut po = po;
            po.frame = max_k + 1;
            po.hash_when_added = self.frames.at(po.frame).get_hash();
            debug_assert!(!self.frames.is_cube_blocked_in_frame(&po.cube, po.frame));
            debug_assert!(self.frames.is_cube_blocked_in_frame(&po.cube, po.frame - 1));
            self.proof_obligations.re_push(po);
        }

        debug_assert!(self.frames.sanity_check());
    }

    fn print_po(&self, po: &ProofObligation) {
        if self.s.parameters.should_print_proof_obligations {
            println!(
                "Proof obligation {} (Frame {}, Length {}) = {}",
                self.proof_obligations.len(),
                po.frame,
                po.cube.len(),
                po.cube
            );
        }
    }

    // pub(super) fn recursively_block_cube_aux(
    //     &mut self,
    // ) -> Result<Result<(), PropertyDirectedReachabilityProofError>, Counterexample> {
    // }

    pub(super) fn recursively_block_cube(
        &mut self,
        cube: Cube,
        input: Cube,
        k: usize,
    ) -> Result<Result<(), PropertyDirectedReachabilityProofError>, Counterexample> {
        let _timer = FunctionTimer::start(function!(), self.s.time_stats.clone());

        let is_initial = self.frames.is_cube_initial(&cube);
        self.proof_obligations.push(
            is_initial,
            cube,
            k,
            self.frames.at(k).get_hash(),
            input,
            None,
        )?;

        while !self.proof_obligations.is_empty() {
            if self.s.parameters.start_time.unwrap().elapsed() > self.s.parameters.timeout {
                return Ok(Err(PropertyDirectedReachabilityProofError::TimeOutReached));
            }

            self.s
                .pdr_stats
                .borrow_mut()
                .increment_generic_count("Total Proof Obligations");

            self.frames.call_condense();

            let po = self.proof_obligations.pop().unwrap().to_owned();
            self.print_po(&po);
            let is_initial = self.frames.is_cube_initial(&po.cube);
            debug_assert!(!is_initial);
            debug_assert!(po.frame > 0);

            if po.frame > self.frames.depth() {
                self.proof_obligations.re_push(po);
                break;
                // this check allows us to make less sat calls when the frame has not
                // changed since the proof obligation was added ->
            } else if (po.hash_when_added != self.frames.at(po.frame).get_hash())
                && self.frames.is_cube_blocked_in_frame(&po.cube, po.frame)
            {
                continue;
            } else {
                debug_assert!(!self.frames.is_cube_blocked_in_frame(&po.cube, po.frame));
                let solver_relative_result =
                    self.frames.get_predecessor_of_cube(&po.cube, po.frame - 1);

                match solver_relative_result {
                    Err(simplified_cube) => {
                        // let cube = simplified_cube;
                        self.no_predecessor_of_cube(po, simplified_cube);
                    }
                    Ok((cube, input)) => {
                        // block this new bad cube from z
                        let is_initial = self.frames.is_cube_initial(&cube);
                        self.proof_obligations.push(
                            is_initial,
                            cube,
                            po.frame - 1,
                            self.frames.at(po.frame - 1).get_hash(),
                            input,
                            Some(po.cube),
                        )?;
                    }
                }
            }
        }
        Ok(Ok(()))
    }
}
