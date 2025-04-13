// // ************************************************************************************************
// // use
// // ************************************************************************************************

// use super::CNF;
// use crate::formulas::{Clause, Variable};
// use crate::models::UniqueSortedHashMap;
// use crate::{formulas::Literal, models::UniqueSortedVec};
// use std::cmp::Ordering;
// use std::collections::BinaryHeap;

// // ************************************************************************************************
// // impl
// // ************************************************************************************************

// #[derive(Debug, Eq, PartialEq)]
// struct HeapElement {
//     priority: usize,
//     literal: Literal,
// }

// impl Ord for HeapElement {
//     fn cmp(&self, other: &Self) -> Ordering {
//         self.priority
//             .cmp(&other.priority)
//             .then(self.literal.cmp(&other.literal))
//     }
// }

// impl PartialOrd for HeapElement {
//     fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
//         Some(self.cmp(other))
//     }
// }

// #[derive(Debug, Eq, PartialEq)]
// struct PElement {
//     pub l_tag: Literal,
//     pub i_c: usize,
//     pub i_d: usize,
// }

// pub struct VariableAdditionInformation {
//     pub l: Literal,
//     pub matched_literals: UniqueSortedVec<Literal>,
//     pub matched_clauses: UniqueSortedVec<usize>,
//     pub clause_indexes_to_delete: UniqueSortedVec<usize>,
// }

// // ************************************************************************************************
// // impl
// // ************************************************************************************************

// impl CNF {
//     // ********************************************************************************************
//     // helper functions
//     // ********************************************************************************************

//     fn get_literal_queue(
//         occurences: &UniqueSortedHashMap<Literal, UniqueSortedVec<usize>>,
//     ) -> BinaryHeap<HeapElement> {
//         let mut queue = BinaryHeap::new();
//         for literal in occurences.iter_sorted() {
//             let priority = occurences.get(&literal).unwrap().len();
//             let heap_element = HeapElement { priority, literal };
//             queue.push(heap_element);
//         }
//         queue
//     }

//     /// Returns the literal in c (that is not l) that has the fewest occurences in the CNF.
//     /// If there are multiple literals with the same number of occurences, it returns the one with the lowst order.
//     /// If c contains only l, it returns None.
//     fn get_l_min(
//         l: Literal,
//         c: &Clause,
//         occurences: &UniqueSortedHashMap<Literal, UniqueSortedVec<usize>>,
//     ) -> Option<Literal> {
//         c.iter()
//             .filter(|x| **x != l)
//             .min_by_key(|x| occurences.get(x).unwrap().len())
//             .copied()
//     }

//     fn get_f_l(
//         l: Literal,
//         occurences: &UniqueSortedHashMap<Literal, UniqueSortedVec<usize>>,
//     ) -> &UniqueSortedVec<usize> {
//         occurences.get(&l).unwrap()
//     }

//     fn calculate_p(
//         clauses: &[Clause],
//         l: Literal,
//         matched_clauses: &UniqueSortedVec<usize>,
//         matched_literals: &UniqueSortedVec<Literal>,
//         occurences: &UniqueSortedHashMap<Literal, UniqueSortedVec<usize>>,
//     ) -> Vec<PElement> {
//         let mut p = vec![];
//         for i_c in matched_clauses.iter().copied() {
//             let c = &clauses[i_c];
//             let l_min = match Self::get_l_min(l, c, occurences) {
//                 Some(x) => x,
//                 None => return vec![],
//             };
//             for i_d in Self::get_f_l(l_min, occurences).iter().copied() {
//                 let d = &clauses[i_d];
//                 if c.len() == d.len() {
//                     let c_sub_d = c.peek().peek().subtract_without_consuming(d.peek().peek());
//                     if (c_sub_d.len() == 1) && (*c_sub_d.at(0) == l) {
//                         let l_tag = d.peek().peek().subtract_without_consuming(c.peek().peek());
//                         debug_assert!(l_tag.len() == 1);
//                         let l_tag = *l_tag.at(0);
//                         // only add to P literals that are not already in matched_literals
//                         if !matched_literals.contains(&l_tag) {
//                             p.push(PElement { l_tag, i_c, i_d });
//                         }
//                     }
//                 }
//             }
//         }
//         p
//     }

//     fn calculate_histogram_of_p_literals(p: &[PElement]) -> Vec<(Literal, i32)> {
//         let mut literals = p.iter().map(|pe| pe.l_tag).collect::<Vec<Literal>>();
//         literals.sort_unstable();
//         let mut histogram = vec![];
//         for l in literals.iter().copied() {
//             match histogram.last_mut() {
//                 Some((last_literal, last_count)) => {
//                     if *last_literal == l {
//                         *last_count += 1;
//                     } else {
//                         debug_assert!(*last_literal < l);
//                         histogram.push((l, 1));
//                     }
//                 }
//                 None => {
//                     histogram.push((l, 1));
//                 }
//             }
//         }
//         histogram
//     }

//     /// Takes P and returns the literal with the highest count, along with the count.
//     /// If there are multiple literals with the same count, it returns the one with the highest value.
//     /// If p is empty, it panics.
//     ///
//     /// # Example
//     ///
//     /// p = [(l1, _), (!l1, _), (!l1, _)]
//     /// returns (!l1, 2)
//     fn get_l_max_and_count(p: &[PElement]) -> Option<(Vec<Literal>, i32)> {
//         let mut histogram = Self::calculate_histogram_of_p_literals(p);
//         // sort by occurance then by literal
//         histogram.sort_unstable_by(|a, b| a.1.cmp(&b.1).then(a.0.cmp(&b.0)));
//         let (_, max_count) = histogram.last()?;
//         let mut l_max_options = vec![];
//         for (l, c) in histogram.iter().rev() {
//             debug_assert!(c <= max_count);
//             if c == max_count {
//                 l_max_options.push(*l);
//             } else {
//                 break;
//             }
//         }
//         Some((l_max_options, *max_count))
//     }

//     fn calculate_reduction(m_lit_length: usize, m_cls_length: usize) -> i32 {
//         let before = m_lit_length * m_cls_length;
//         let after = m_lit_length + m_cls_length;
//         (before as i32) - (after as i32)
//     }

//     fn did_reduction_improve(
//         matched_clauses: &UniqueSortedVec<usize>,
//         matched_literals: &UniqueSortedVec<Literal>,
//         l_max_count: i32,
//     ) -> bool {
//         let prev_clause_count = matched_clauses.len();
//         let new_clause_count = l_max_count;

//         let prev_lit_count = matched_literals.len();
//         let new_lit_count = prev_lit_count + 1;

//         // if adding lmax to Mlit does not result in a reduction then stop
//         let current_reduction = Self::calculate_reduction(prev_lit_count, prev_clause_count);
//         let new_reduction = Self::calculate_reduction(new_lit_count, new_clause_count as usize);

//         new_reduction > current_reduction
//     }

//     /// We use a heuristic that is inspired by
//     /// https://arxiv.org/abs/2307.01904
//     /// Here we make a guess that variables that are closer together in terms of numbers
//     /// will also be close in terms of structure.
//     /// This is way easier to implement than the actual heuristic, and faster.
//     fn tie_breaker(l: Literal, mut l_max_options: Vec<Literal>) -> Literal {
//         l_max_options.sort_by_key(|x| l.variable().get().abs_diff(x.variable().get()));
//         l_max_options[0]
//     }

//     fn static_get_literal_occurences(
//         clauses: &[Clause],
//     ) -> Option<UniqueSortedHashMap<Literal, UniqueSortedVec<usize>>> {
//         let max_var = clauses
//             .iter()
//             .map(|c| c.get_highest_variable_number())
//             .max()?;
//         let max_lit = std::cmp::max(max_var.literal(false), max_var.literal(true));
//         let mut literal_to_occurences: UniqueSortedHashMap<Literal, UniqueSortedVec<usize>> =
//             UniqueSortedHashMap::new(max_lit);
//         for (i, clause) in clauses.iter().enumerate() {
//             for literal in clause.iter() {
//                 match literal_to_occurences.get_mut(literal) {
//                     Some(usv) => usv.push(i),
//                     None => {
//                         literal_to_occurences
//                             .insert(*literal, UniqueSortedVec::from_ordered_set(vec![i]));
//                     }
//                 }
//             }
//         }
//         Some(literal_to_occurences)
//     }

//     // ********************************************************************************************
//     // API
//     // ********************************************************************************************

//     pub fn simple_bounded_variable_addition_search_addition_using_l(
//         clauses: &[Clause],
//         l: Literal,
//         occurences: &UniqueSortedHashMap<Literal, UniqueSortedVec<usize>>,
//     ) -> Option<VariableAdditionInformation> {
//         // initialize the matching literals.
//         let mut matched_literals = UniqueSortedVec::from_ordered_set(vec![l]);
//         let mut matched_clauses = Self::get_f_l(l, occurences).to_owned();
//         let mut clauses_that_maybe_deleted: Vec<(usize, usize)> = vec![];
//         loop {
//             let p = Self::calculate_p(clauses, l, &matched_clauses, &matched_literals, occurences);
//             debug_assert!(p.iter().all(|pe| pe.l_tag != l));
//             let (l_max_options, l_max_count) = match Self::get_l_max_and_count(&p) {
//                 Some(x) => x,
//                 None => {
//                     break;
//                 }
//             };
//             debug_assert!(l_max_options.iter().all(|x| *x != l));
//             debug_assert!(l_max_options.iter().all(|x| !matched_literals.contains(x)));
//             let improve =
//                 Self::did_reduction_improve(&matched_clauses, &matched_literals, l_max_count);
//             if improve {
//                 let l_max = Self::tie_breaker(l, l_max_options);
//                 debug_assert!(!matched_literals.contains(&l_max));

//                 matched_literals.insert(l_max);
//                 matched_clauses.clear();
//                 for pe in p.iter().filter(|pe| pe.l_tag == l_max) {
//                     matched_clauses.push(pe.i_c);
//                     clauses_that_maybe_deleted.push((pe.i_c, pe.i_d));
//                 }
//             } else {
//                 break;
//             }
//         }

//         if (matched_literals.len() == 1)
//             || (matched_literals.len() <= 2 && matched_clauses.len() <= 2)
//         {
//             return None;
//         }

//         let mut indexes_to_delete: Vec<usize> = clauses_that_maybe_deleted
//             .iter()
//             .filter(|x| matched_clauses.contains(&x.0))
//             .map(|x| x.1)
//             .collect();
//         indexes_to_delete.extend(matched_clauses.iter());

//         // println!(
//         //     "Indexes to delete: [{}]",
//         //     indexes_to_delete
//         //         .iter()
//         //         .map(|x| x.to_string())
//         //         .collect::<Vec<String>>()
//         //         .join(", ")
//         // );

//         // debug_assert!(indexes_to_delete.iter().all(|x| *x < clauses.len()));
//         // let clause_indexes_to_delete = UniqueSortedVec::from_sequence(indexes_to_delete);
//         // debug_assert!(
//         //     clause_indexes_to_delete.len() == matched_literals.len() * matched_clauses.len()
//         // );
//         Some(VariableAdditionInformation {
//             l,
//             matched_literals,
//             matched_clauses,
//             clause_indexes_to_delete: UniqueSortedVec::from_sequence(indexes_to_delete),
//         })
//     }

//     pub fn simple_bounded_variable_addition_rewrite_cnf(
//         clauses: &mut Vec<Clause>,
//         new_variable: Variable,
//         variable_addition_info: VariableAdditionInformation,
//     ) -> Vec<Clause> {
//         debug_assert!(clauses
//             .iter()
//             .all(|c| c.iter().all(|l| l.variable().get() != new_variable.get())));

//         let x = new_variable.literal(false);
//         let mut clauses_added = vec![];
//         for c_i in variable_addition_info.matched_clauses.iter().copied() {
//             let mut clause = clauses[c_i].to_owned();
//             clause.remove(&variable_addition_info.l);
//             clause.insert(x);
//             clauses_added.push(clause.clone());
//             clauses.push(clause)
//         }
//         for l_tag in variable_addition_info.matched_literals.iter().copied() {
//             let clause = Clause::from_ordered_set(vec![l_tag, !x]);
//             clauses_added.push(clause.clone());
//             clauses.push(clause);
//         }
//         for r in variable_addition_info
//             .clause_indexes_to_delete
//             .iter()
//             .rev()
//             .copied()
//         {
//             clauses.remove(r);
//         }

//         clauses_added
//     }

//     /// Bounded Variable Addition (BVA)
//     /// This version of BVA is static, meaning that it does not require the CNF to be a CNF object.
//     ///
//     /// For more information checkout `simple_bounded_variable_addition`
//     ///
//     /// # Example
//     ///
//     /// ```
//     /// use rust_formal_verification::formulas::{Variable, Literal, Clause, CNF};
//     /// let a = Literal::new(Variable::new(1));
//     /// let b = Literal::new(Variable::new(2));
//     /// let c = Literal::new(Variable::new(3));
//     /// let d = Literal::new(Variable::new(4));
//     /// let e = Literal::new(Variable::new(5));
//     /// let x = Literal::new(Variable::new(6));
//     ///
//     /// let mut clauses = vec![
//     ///     Clause::from_ordered_set(vec![a, c]),
//     ///     Clause::from_ordered_set(vec![a, d]),
//     ///     Clause::from_ordered_set(vec![a, e]),
//     ///     Clause::from_ordered_set(vec![b, c]),
//     ///     Clause::from_ordered_set(vec![b, d]),
//     ///     Clause::from_ordered_set(vec![b, e]),
//     /// ];
//     ///
//     /// CNF::static_simple_bounded_variable_addition(&mut clauses, usize::MAX, || Variable::new(6));
//     ///
//     /// assert_eq!(clauses, vec![
//     ///     Clause::from_ordered_set(vec![c, x]),
//     ///     Clause::from_ordered_set(vec![d, x]),
//     ///     Clause::from_ordered_set(vec![e, x]),
//     ///     Clause::from_ordered_set(vec![a, !x]),
//     ///     Clause::from_ordered_set(vec![b, !x]),
//     /// ]);
//     /// ```
//     pub fn static_simple_bounded_variable_addition<F>(
//         clauses: &mut Vec<Clause>,
//         max_variable_introductions: usize,
//         mut new_variable_getter: F,
//     ) -> Vec<(Variable, Vec<Clause>)>
//     where
//         F: FnMut() -> Variable,
//     {
//         let mut introduced_variables = vec![];
//         let mut occurences = match Self::static_get_literal_occurences(clauses) {
//             Some(x) => x,
//             None => {
//                 // clauses is empty
//                 return introduced_variables;
//             }
//         };
//         let mut q = Self::get_literal_queue(&occurences);
//         while !q.is_empty() {
//             let l = q.pop().unwrap().literal;
//             let r = Self::simple_bounded_variable_addition_search_addition_using_l(
//                 clauses,
//                 l,
//                 &occurences,
//             );
//             if let Some(info) = r {
//                 let new_variable = new_variable_getter();
//                 // println!("Adding variable {}", new_variable);
//                 let length_before = clauses.len();
//                 let clauses_added =
//                     Self::simple_bounded_variable_addition_rewrite_cnf(clauses, new_variable, info);
//                 introduced_variables.push((new_variable, clauses_added));
//                 let length_after = clauses.len();
//                 debug_assert!(length_after < length_before);

//                 // early stopping
//                 if introduced_variables.len() >= max_variable_introductions {
//                     break;
//                 }

//                 // update parameters for next iteration after having introduced the variable
//                 occurences = Self::static_get_literal_occurences(clauses).unwrap();
//                 q = Self::get_literal_queue(&occurences);
//             }
//         }
//         introduced_variables
//     }

//     /// Bounded Variable Addition (BVA) as implemented in:
//     /// https://fmv.jku.at/bva/
//     ///
//     /// We also took into consideration the heuristic from the paper:
//     /// https://arxiv.org/abs/2307.01904
//     ///
//     /// (We did not implement the actual heuristic, but a simpler alternative)
//     pub fn simple_bounded_variable_addition<F>(
//         &mut self,
//         max_variable_introductions: usize,
//         new_variable_getter: F,
//     ) -> Vec<(Variable, Vec<Clause>)>
//     where
//         F: FnMut() -> Variable,
//     {
//         let mut clauses = std::mem::take(&mut self.clauses).unpack();
//         let vars = Self::static_simple_bounded_variable_addition(
//             &mut clauses,
//             max_variable_introductions,
//             new_variable_getter,
//         );
//         self.clauses = UniqueSortedVec::from_sequence(clauses);
//         for (v, _) in vars.iter() {
//             self.max_variable_number = std::cmp::max(self.max_variable_number, *v);
//         }
//         vars
//     }
// }

// #[cfg(test)]
// mod tests {

//     use super::*;
//     use crate::formulas::Variable;

//     #[test]
//     fn test_simple_bounded_variable_addition() {
//         let a = Literal::new(Variable::new(1));
//         let b = Literal::new(Variable::new(2));
//         let c = Literal::new(Variable::new(3));
//         let d = Literal::new(Variable::new(4));
//         let e = Literal::new(Variable::new(5));
//         let x = Literal::new(Variable::new(6));

//         let mut clauses = CNF::from_sequence(vec![
//             Clause::from_sequence(vec![a, c]),
//             Clause::from_sequence(vec![a, d]),
//             Clause::from_sequence(vec![a, e]),
//             Clause::from_sequence(vec![b, c]),
//             Clause::from_sequence(vec![b, d]),
//             Clause::from_sequence(vec![b, e]),
//         ]);

//         let mut h = 5;
//         clauses.simple_bounded_variable_addition(usize::MAX, move || {
//             h += 1;
//             Variable::new(h)
//         });

//         let expected_result = CNF::from_sequence(vec![
//             Clause::from_sequence(vec![a, !x]),
//             Clause::from_sequence(vec![b, !x]),
//             Clause::from_sequence(vec![c, x]),
//             Clause::from_sequence(vec![d, x]),
//             Clause::from_sequence(vec![e, x]),
//         ]);

//         assert_eq!(clauses, expected_result);
//     }

//     #[test]
//     fn test_simple_bounded_variable_addition_2() {
//         let a = Literal::new(Variable::new(1));
//         let b = Literal::new(Variable::new(2));
//         let p = Literal::new(Variable::new(3));
//         let q = Literal::new(Variable::new(4));
//         let r = Literal::new(Variable::new(5));
//         let s = Literal::new(Variable::new(6));
//         let t = Literal::new(Variable::new(7));

//         let mut clauses = CNF::from_sequence(vec![
//             Clause::from_sequence(vec![a, p, q]),
//             Clause::from_sequence(vec![a, p, r]),
//             Clause::from_sequence(vec![a, r, s]),
//             Clause::from_sequence(vec![a, t]),
//             Clause::from_sequence(vec![b, p, q]),
//             Clause::from_sequence(vec![b, p, r]),
//             Clause::from_sequence(vec![b, r, s]),
//             Clause::from_sequence(vec![b, t]),
//         ]);

//         let mut h = 7;
//         clauses.simple_bounded_variable_addition(usize::MAX, move || {
//             h += 1;
//             Variable::new(h)
//         });

//         let x = Literal::new(Variable::new(8));
//         let expected_result = CNF::from_sequence(vec![
//             Clause::from_sequence(vec![x, p, q]),
//             Clause::from_sequence(vec![x, p, r]),
//             Clause::from_sequence(vec![x, r, s]),
//             Clause::from_sequence(vec![x, t]),
//             Clause::from_sequence(vec![!x, a]),
//             Clause::from_sequence(vec![!x, b]),
//         ]);
//         assert_eq!(clauses, expected_result);
//     }
// }
