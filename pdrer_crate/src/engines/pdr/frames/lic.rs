// // ************************************************************************************************
// // use
// // ************************************************************************************************

// use super::Frame;
// use crate::formulas::Clause;
// use crate::function;
// use crate::models::time_stats::function_timer::FunctionTimer;
// use crate::solvers::dd::DecisionDiagramManager;
// // use rand::Rng;

// // ************************************************************************************************
// // impl
// // ************************************************************************************************

// impl<D: DecisionDiagramManager> Frame<D> {
//     // ********************************************************************************************
//     // helper functions
//     // ********************************************************************************************

//     pub(super) fn is_clause_satisfied_by_all_initial_states_aux(
//         &mut self,
//         mut initial_frame: &mut Option<&mut Frame<D>>,
//         c: &Clause,
//     ) -> bool {
//         match &mut initial_frame {
//             Some(f) => f.is_clause_satisfied_by_all_initial_states(c),
//             None => {
//                 debug_assert!(self.initial.is_some());
//                 self.is_clause_satisfied_by_all_initial_states(c)
//             }
//         }
//     }

//     // ********************************************************************************************
//     // construction API
//     // ********************************************************************************************

//     pub fn largest_inductive_sub_clause_sequence(
//         &mut self,
//         initial_frame: &mut Option<&mut Frame<D>>,
//         mut clauses: Vec<Clause>,
//         previous_attempts: &mut Vec<Vec<Clause>>,
//     ) -> Vec<Clause> {
//         let _timer = FunctionTimer::start(function!(), self.s.time_stats.clone());

//         let mut i = 0;
//         let mut number_of_consecutive_un_sat = 0;
//         let mut current_attempts = vec![];
//         loop {
//             if previous_attempts.iter().any(|prev| {
//                 prev.iter()
//                     .zip(clauses.iter())
//                     .all(|(a, b)| b.peek().peek().is_subset_of(a.peek().peek()))
//             }) {
//                 return vec![];
//             }
//             current_attempts.push(clauses.clone());

//             let next = (i + 1) % clauses.len();
//             let r = self.get_state_in_clause_a_that_has_a_predecessor_not_in_clause_b(
//                 &clauses[i],
//                 &clauses[next],
//             );
//             match r {
//                 Some(counter_example_to_induction) => {
//                     number_of_consecutive_un_sat = 0;
//                     clauses[i].retain(|l| !(counter_example_to_induction.peek().contains(l)));

//                     if !self
//                         .is_clause_satisfied_by_all_initial_states_aux(initial_frame, &clauses[i])
//                     {
//                         previous_attempts.append(&mut current_attempts);
//                         return vec![];
//                     }
//                 }
//                 None => {
//                     number_of_consecutive_un_sat += 1;
//                     if number_of_consecutive_un_sat == clauses.len() {
//                         return clauses;
//                     }
//                     i = next;
//                 }
//             }
//         }
//     }

//     pub fn largest_inductive_sub_clause(
//         &mut self,
//         initial_frame: &mut Option<&mut Frame<D>>,
//         clause: Clause,
//         previous_attempts: &mut Vec<Vec<Clause>>,
//     ) -> Option<Clause> {
//         let mut r = self.largest_inductive_sub_clause_sequence(
//             initial_frame,
//             vec![clause],
//             previous_attempts,
//         );
//         if r.is_empty() {
//             None
//         } else {
//             debug_assert!(r.len() == 1);
//             Some(r.remove(0))
//         }
//     }
// }
