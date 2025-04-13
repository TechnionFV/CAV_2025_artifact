// ************************************************************************************************
// use
// ************************************************************************************************

use super::Frames;
use crate::{
    engines::pdr::PropertyDirectedReachabilitySolver,
    formulas::{Clause, CNF},
    function,
    models::time_stats::function_timer::FunctionTimer,
    solvers::{dd::DecisionDiagramManager, sat::incremental::CaDiCalSolver},
};

// ************************************************************************************************
// impl
// ************************************************************************************************

impl<T: PropertyDirectedReachabilitySolver, D: DecisionDiagramManager> Frames<T, D> {
    // ********************************************************************************************
    // helper functions
    // ********************************************************************************************

    // fn get_inductive_subset_of_depth_frame_old(&self) -> Vec<Clause> {
    //     let clauses_in_last_frame: Vec<Clause> = self
    //         .frames
    //         .iter()
    //         .rev()
    //         .skip(1)
    //         .flat_map(|f| f.get_delta_clauses_cloned().to_owned())
    //         .collect();

    //     let clauses_in_infinite_frame = CNF::from_sequence(
    //         self.frames[self.frames.len() - 1]
    //             .get_delta_clauses_cloned()
    //             .to_owned(),
    //     );
    //     let definitions = CNF::from_sequence(
    //         self.definition_library
    //             .iter()
    //             .flat_map(|d| d.to_cnf())
    //             .collect(),
    //     );
    //     let mut definitions_tag = definitions.clone();
    //     self.s
    //         .fin_state
    //         .borrow()
    //         .add_tags_to_relation(&mut definitions_tag, 1);
    //     let transition_with_definitions = {
    //         let mut a = self.transition.clone();
    //         a.append(clauses_in_infinite_frame);
    //         a.append(definitions);
    //         a.append(definitions_tag);
    //         a
    //     };

    //     self.s
    //         .fin_state
    //         .borrow()
    //         .get_inductive_subset_relative_to_cnf::<CaDiCalSolver>(
    //             clauses_in_last_frame,
    //             &transition_with_definitions,
    //             false,
    //         )
    // }

    fn get_inductive_subset_of_depth_frame(&mut self) -> Vec<Clause> {
        // let i = self.depth();
        for j in 0..self.s.parameters.infinite_frame_propagation_limit {
            let i = self.depth() + j;
            debug_assert!(i <= (self.frames.len() - 2));
            if i == (self.frames.len() - 2) {
                self.new_frame();
            }
            if self.frames[i].is_empty() {
                debug_assert!(self
                    .frames
                    .iter()
                    .skip(i)
                    .rev()
                    .skip(1)
                    .all(|f| f.is_empty()));
                break;
            }
            if let Some(i) = self.propagate_blocked_cubes_in_range(
                i,
                i + 1,
                true,
                self.s.parameters.er_fp && self.s.parameters.er_fp_for_f_inf,
            ) {
                debug_assert!(self.frames[i].is_empty());
                let v: Vec<Clause> = (i..=self.frames.len() - 2)
                    .flat_map(|x| self.frames[x].get_delta_clauses_cloned().to_owned())
                    .collect();
                debug_assert!(!v.is_empty());
                return v;
            }
        }
        vec![]
    }

    // fn check_same_as_b4(&self, now: &[Clause]) -> bool {
    //     let b4 = self.get_inductive_subset_of_depth_frame_old();
    //     assert!(b4.iter().all(|c| now.contains(c)));
    //     true
    // }

    // ********************************************************************************************
    // API
    // ********************************************************************************************

    pub fn propagate_to_infinite_frame(&mut self) -> Option<usize> {
        let _timer = FunctionTimer::start(function!(), self.s.time_stats.clone());

        // debug_assert!(self.check_no_redundancy());
        let lemmas = self.get_inductive_subset_of_depth_frame();
        // if !parameters.use_extension_variables {
        if lemmas.is_empty() {
            return None;
        }
        // debug_assert!(self.check_same_as_b4(&lemmas));
        debug_assert!(self
            .s
            .fin_state
            .borrow()
            .is_cnf_semi_inductive_with_definitions::<CaDiCalSolver>(self.get_definitions(), &{
                let mut a = CNF::from_sequence(self.get_cnf_of_frame(self.frames.len() - 1));
                a.append(CNF::from_sequence(lemmas.to_owned()));
                a
            }));
        // }
        self.generalize_and_add_to_f_infinity(lemmas);
        // self.generalize_and_add_to_f_infinity(lemmas, parameters);
        let r = self.is_invariant_found();
        debug_assert!(self
            .s
            .fin_state
            .borrow()
            .is_cnf_semi_inductive_with_definitions::<CaDiCalSolver>(
                self.get_definitions(),
                &CNF::from_sequence(self.get_cnf_of_frame(self.frames.len() - 1))
            ));
        debug_assert!(
            {
                let mut is_correct = true;
                let mut is_empty_seen = false;
                for i in self.depth()..(self.frames.len() - 1) {
                    if self.frames[i].is_empty() {
                        is_empty_seen = true;
                    } else {
                        is_correct = is_correct && !is_empty_seen;
                    }
                }
                is_correct
            },
            "Lemmas found that did not clear the frames where these lemmas are."
        );
        debug_assert!(self.check_that_f_infinity_is_inductive());
        r
    }
}
