// ************************************************************************************************
// use
// ************************************************************************************************

use super::PElement;
use crate::formulas::Variable;
use crate::{formulas::Literal, models::UniqueSortedVec};

// ************************************************************************************************
// struct
// ************************************************************************************************

pub struct BoundedVariableAdditionParameters<'a, 'b> {
    pub max_variable_introductions: usize,
    pub add_definition_clauses: bool,
    pub add_constraint_clauses: bool,
    pub remove_previous_clauses: bool,
    #[allow(clippy::type_complexity)]
    pub new_variable_getter: Box<dyn FnMut(&UniqueSortedVec<Literal>) -> Variable + 'a>,
    #[allow(clippy::type_complexity)]
    pub choose_search_continuation_literal: Box<
        dyn FnMut(
                &Literal,
                &[PElement],
                &UniqueSortedVec<usize>,
                &UniqueSortedVec<Literal>,
            ) -> Option<Literal>
            + 'b,
    >,
}

// ************************************************************************************************
// impl
// ************************************************************************************************

impl BoundedVariableAdditionParameters<'_, '_> {
    // ********************************************************************************************
    // helper function for default implementation
    // ********************************************************************************************

    fn calculate_histogram_of_p_literals(p: &[PElement]) -> Vec<(Literal, i32)> {
        let mut literals = p.iter().map(|pe| pe.l_tag).collect::<Vec<Literal>>();
        literals.sort_unstable();
        let mut histogram = vec![];
        for l in literals.iter().copied() {
            match histogram.last_mut() {
                Some((last_literal, last_count)) => {
                    if *last_literal == l {
                        *last_count += 1;
                    } else {
                        debug_assert!(*last_literal < l);
                        histogram.push((l, 1));
                    }
                }
                None => {
                    histogram.push((l, 1));
                }
            }
        }
        histogram
    }

    /// Takes P and returns the literal with the highest count, along with the count.
    /// If there are multiple literals with the same count, it returns the one with the highest value.
    /// If p is empty, it panics.
    ///
    /// # Example
    ///
    /// p = [(l1, _), (!l1, _), (!l1, _)]
    /// returns (!l1, 2)
    fn get_l_max_and_count(mut histogram: Vec<(Literal, i32)>) -> (Vec<Literal>, i32) {
        // sort by occurance then by literal
        histogram.sort_unstable_by(|a, b| a.1.cmp(&b.1).then(a.0.cmp(&b.0)));
        let (_, max_count) = histogram.last().unwrap();
        let mut l_max_options = vec![];
        for (l, c) in histogram.iter().rev() {
            debug_assert!(c <= max_count);
            if c == max_count {
                l_max_options.push(*l);
            } else {
                break;
            }
        }
        (l_max_options, *max_count)
    }

    fn calculate_reduction(m_lit_length: usize, m_cls_length: usize) -> i32 {
        let before = m_lit_length * m_cls_length;
        let after = m_lit_length + m_cls_length;
        (before as i32) - (after as i32)
    }

    fn did_reduction_improve(
        matched_clauses: &UniqueSortedVec<usize>,
        matched_literals: &UniqueSortedVec<Literal>,
        l_max_count: i32,
    ) -> bool {
        let prev_clause_count = matched_clauses.len();
        let new_clause_count = l_max_count;

        let prev_lit_count = matched_literals.len();
        let new_lit_count = prev_lit_count + 1;

        // if adding lmax to Mlit does not result in a reduction then stop
        let current_reduction = Self::calculate_reduction(prev_lit_count, prev_clause_count);
        let new_reduction = Self::calculate_reduction(new_lit_count, new_clause_count as usize);

        new_reduction > current_reduction
    }

    /// We use a heuristic that is inspired by
    /// https://arxiv.org/abs/2307.01904
    /// Here we make a guess that variables that are closer together in terms of numbers
    /// will also be close in terms of structure.
    /// This is way easier to implement than the actual heuristic, and faster.
    fn tie_breaker(l: Literal, mut l_max_options: Vec<Literal>) -> Literal {
        l_max_options.sort_by_key(|x| l.variable().number().abs_diff(x.variable().number()));
        l_max_options[0]
    }

    // ********************************************************************************************
    // API
    // ********************************************************************************************

    pub fn new() -> Self {
        // let f1 =;
        Self {
            max_variable_introductions: usize::MAX,
            add_definition_clauses: true,
            add_constraint_clauses: true,
            remove_previous_clauses: true,
            new_variable_getter: Box::new(|_| Variable::new(0)),
            choose_search_continuation_literal: Box::new(
                |l, p, matched_clauses, matched_literals| {
                    // if matched literals already include negating literals then stop the search
                    if matched_literals.peek().windows(2).any(|w| w[0] == !w[1]) {
                        return None;
                    }

                    let histogram = Self::calculate_histogram_of_p_literals(p);

                    // if we can add a literal that negates the something in the matched literals then we should
                    if let Some(i) = histogram
                        .iter()
                        .find(|bar| matched_literals.contains(&!bar.0))
                    {
                        return Some(i.0);
                    }

                    let (l_max_options, l_max_count) = Self::get_l_max_and_count(histogram);
                    debug_assert!(l_max_options.iter().all(|x| x != l));
                    debug_assert!(l_max_options.iter().all(|x| !matched_literals.contains(x)));
                    let improve =
                        Self::did_reduction_improve(matched_clauses, matched_literals, l_max_count);
                    if improve {
                        let l_max = Self::tie_breaker(*l, l_max_options);
                        debug_assert!(!matched_literals.contains(&l_max));
                        Some(l_max)
                    } else {
                        None
                    }
                },
            ),
        }
    }
}

// ************************************************************************************************
// default impl
// ************************************************************************************************
impl Default for BoundedVariableAdditionParameters<'_, '_> {
    fn default() -> Self {
        Self::new()
    }
}
