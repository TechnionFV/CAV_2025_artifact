// ************************************************************************************************
// use
// ************************************************************************************************

use super::{BoundedVariableAdditionParameters, PElement, VariableAddition, CNF};
// use crate::formulas::cnf::bounded_variable_addition::PElement;
use crate::formulas::{Clause, Variable};
use crate::models::{SortedVecOfLiterals, UniqueSortedHashMap, Utils};
use crate::{formulas::Literal, models::UniqueSortedVec};
use std::cmp::Ordering;
use std::collections::BinaryHeap;

// ************************************************************************************************
// internal structs
// ************************************************************************************************

#[derive(Debug, Eq, PartialEq)]
struct HeapElement {
    priority: usize,
    literal: Literal,
}

impl Ord for HeapElement {
    fn cmp(&self, other: &Self) -> Ordering {
        self.priority
            .cmp(&other.priority)
            .then(self.literal.cmp(&other.literal))
    }
}

impl PartialOrd for HeapElement {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

struct VariableAdditionInformation {
    pub l: Literal,
    pub matched_literals: UniqueSortedVec<Literal>,
    pub matched_clauses: UniqueSortedVec<usize>,
    pub clause_indexes_to_delete: UniqueSortedVec<usize>,
}

// ************************************************************************************************
// impl
// ************************************************************************************************

impl CNF {
    // ********************************************************************************************
    // helper functions
    // ********************************************************************************************

    fn get_literal_queue(
        occurences: &UniqueSortedHashMap<Literal, UniqueSortedVec<usize>>,
    ) -> BinaryHeap<HeapElement> {
        let mut queue = Vec::with_capacity(occurences.len());
        for literal in occurences.iter_sorted() {
            let priority = occurences.get(&literal).unwrap().len();
            let heap_element = HeapElement { priority, literal };
            queue.push(heap_element);
        }
        BinaryHeap::from(queue)
    }

    fn update_literal_queue(
        queue: BinaryHeap<HeapElement>,
        occurences: &UniqueSortedHashMap<Literal, UniqueSortedVec<usize>>,
        l: Literal,
        new_variable: Option<Variable>,
        parameters: &BoundedVariableAdditionParameters,
    ) -> BinaryHeap<HeapElement> {
        let mut v = queue.into_vec();
        let mut to_remove = UniqueSortedVec::new();
        for (i, he) in v.iter_mut().enumerate() {
            match occurences.get(&he.literal) {
                Some(o) => {
                    he.priority = o.len();
                }
                None => {
                    to_remove.push(i);
                }
            }
        }
        Utils::remove_indexes(&mut v, &to_remove);
        if parameters.remove_previous_clauses {
            if let Some(o) = occurences.get(&l) {
                v.push(HeapElement {
                    priority: o.len(),
                    literal: l,
                });
            }

            if let Some(nv) = new_variable {
                let x = nv.literal(false);
                if parameters.add_constraint_clauses {
                    v.push(HeapElement {
                        priority: occurences.get(&x).unwrap().len(),
                        literal: x,
                    });
                }
                if parameters.add_definition_clauses {
                    v.push(HeapElement {
                        priority: occurences.get(&!x).unwrap().len(),
                        literal: !x,
                    });
                }
            }
        }
        BinaryHeap::from(v)
    }

    /// Returns the literal in c (that is not l) that has the fewest occurences in the CNF.
    /// If there are multiple literals with the same number of occurences, it returns the one with the lowst order.
    /// If c contains only l, it returns None.
    fn get_l_min(
        l: Literal,
        c: &Clause,
        occurences: &UniqueSortedHashMap<Literal, UniqueSortedVec<usize>>,
    ) -> Option<Literal> {
        c.iter()
            .filter(|x| **x != l)
            .min_by_key(|x| occurences.get(x).unwrap().len())
            .copied()
    }

    fn get_f_l(
        l: Literal,
        occurences: &UniqueSortedHashMap<Literal, UniqueSortedVec<usize>>,
    ) -> &UniqueSortedVec<usize> {
        occurences.get(&l).unwrap()
    }

    fn calculate_p(
        clauses: &[Clause],
        l: Literal,
        matched_clauses: &UniqueSortedVec<usize>,
        matched_literals: &UniqueSortedVec<Literal>,
        occurences: &UniqueSortedHashMap<Literal, UniqueSortedVec<usize>>,
    ) -> Vec<PElement> {
        let mut p = vec![];
        let mut subtraction_vec = Vec::with_capacity(2);
        for i_c in matched_clauses.iter().copied() {
            let c = &clauses[i_c];
            let l_min = match Self::get_l_min(l, c, occurences) {
                Some(x) => x,
                None => return vec![],
            };
            for i_d in Self::get_f_l(l_min, occurences).iter().copied() {
                let d = &clauses[i_d];
                if c.len() == d.len() {
                    subtraction_vec.clear();
                    c.peek()
                        .peek()
                        .subtract_custom(d.peek().peek(), 2, &mut subtraction_vec);
                    if (subtraction_vec.len() == 1) && (subtraction_vec[0] == l) {
                        subtraction_vec.clear();
                        d.peek()
                            .peek()
                            .subtract_custom(c.peek().peek(), 1, &mut subtraction_vec);
                        debug_assert!(subtraction_vec.len() == 1);
                        let l_tag = subtraction_vec[0];
                        // only add to P literals that are not already in matched_literals
                        if !matched_literals.contains(&l_tag) {
                            p.push(PElement { l_tag, i_c, i_d });
                        }
                    }
                }
            }
        }
        p
    }

    // ********************************************************************************************
    // API
    // ********************************************************************************************

    fn simple_bounded_variable_addition_search_addition_using_l(
        clauses: &[Clause],
        l: Literal,
        occurences: &UniqueSortedHashMap<Literal, UniqueSortedVec<usize>>,
        parameters: &mut BoundedVariableAdditionParameters,
    ) -> Option<VariableAdditionInformation> {
        // initialize the matching literals.
        debug_assert!({
            let mut v = clauses.to_owned();
            v.sort();
            v.dedup();
            v.len() == clauses.len()
        });
        let mut matched_literals = UniqueSortedVec::from_ordered_set(vec![l]);
        let mut matched_clauses = Self::get_f_l(l, occurences).to_owned();
        let mut clauses_that_maybe_deleted: Vec<(usize, usize)> = vec![];
        loop {
            let p = Self::calculate_p(clauses, l, &matched_clauses, &matched_literals, occurences);
            if p.is_empty() {
                break;
            }

            match (parameters.choose_search_continuation_literal)(
                &l,
                &p,
                &matched_clauses,
                &matched_literals,
            ) {
                Some(l_continue) => {
                    matched_literals.insert(l_continue);
                    matched_clauses.clear();
                    for pe in p.iter().filter(|pe| pe.l_tag == l_continue) {
                        matched_clauses.push(pe.i_c);
                        clauses_that_maybe_deleted.push((pe.i_c, pe.i_d));
                    }
                }
                None => break,
            };

            debug_assert!(p.iter().all(|pe| pe.l_tag != l));
        }

        if (matched_literals.len() == 1)
            || (matched_literals.len() <= 2 && matched_clauses.len() <= 2)
        {
            return None;
        }

        let mut indexes_to_delete: Vec<usize> = clauses_that_maybe_deleted
            .iter()
            .filter(|x| matched_clauses.contains(&x.0))
            .map(|x| x.1)
            .collect();
        indexes_to_delete.extend(matched_clauses.iter());

        // println!(
        //     "Indexes to delete: [{}]",
        //     indexes_to_delete
        //         .iter()
        //         .map(|x| x.to_string())
        //         .collect::<Vec<String>>()
        //         .join(", ")
        // );

        // debug_assert!(indexes_to_delete.iter().all(|x| *x < clauses.len()));
        // let clause_indexes_to_delete = UniqueSortedVec::from_sequence(indexes_to_delete);
        // debug_assert!(
        //     clause_indexes_to_delete.len() == matched_literals.len() * matched_clauses.len()
        // );
        Some(VariableAdditionInformation {
            l,
            matched_literals,
            matched_clauses,
            clause_indexes_to_delete: UniqueSortedVec::from_sequence(indexes_to_delete),
        })
    }

    pub fn try_simple_bounded_variable_addition_on_l(
        clauses: &mut Vec<Clause>,
        occurences: &UniqueSortedHashMap<Literal, UniqueSortedVec<usize>>,
        l: Literal,
        parameters: &mut BoundedVariableAdditionParameters,
    ) -> Option<VariableAddition> {
        let r = Self::simple_bounded_variable_addition_search_addition_using_l(
            clauses, l, occurences, parameters,
        );
        if let Some(info) = r {
            let va = Self::simple_bounded_variable_addition_rewrite_cnf(clauses, info, parameters);

            Some(va)
        } else {
            None
        }
    }

    fn simple_bounded_variable_addition_rewrite_cnf(
        clauses: &mut Vec<Clause>,
        info: VariableAdditionInformation,
        parameters: &mut BoundedVariableAdditionParameters,
    ) -> VariableAddition {
        if !SortedVecOfLiterals::are_variables_sorted_and_unique(info.matched_literals.peek()) {
            // a literal and its negation are in the matched literals
            // we do not need a new variable
            let mut constraint_clauses = vec![];

            for c_i in info.matched_clauses.iter().copied() {
                let mut clause = clauses[c_i].to_owned();
                clause.remove(&info.l);
                constraint_clauses.push(clause.clone());
                if parameters.add_constraint_clauses && !clauses.contains(&clause) {
                    clauses.push(clause)
                }
            }

            let removed_clauses: Vec<Clause> = info
                .clause_indexes_to_delete
                .iter()
                .map(|i| clauses[*i].clone())
                .collect();
            if parameters.remove_previous_clauses {
                Utils::remove_indexes(clauses, &info.clause_indexes_to_delete);
            }

            return VariableAddition {
                variable: None,
                constraint_clauses,
                matched_literals: info.matched_literals,
                definition_clauses: vec![],
                removed_clauses,
                removed_clauses_indexes: info.clause_indexes_to_delete,
            };
        }

        let variable = (parameters.new_variable_getter)(&info.matched_literals);

        let mut to_remove = info.clause_indexes_to_delete;

        let x = variable.literal(false);
        let mut constraint_clauses = vec![];
        let mut definition_clauses = vec![];
        for c_i in info.matched_clauses.iter().copied() {
            let mut clause = clauses[c_i].to_owned();
            clause.remove(&info.l);
            if clause.contains(&!x) {
                to_remove.insert(c_i);
                continue;
            }
            // else if clause.contains(&x) {
            //     continue;
            // }
            clause.insert(x);
            constraint_clauses.push(clause.clone());
            if parameters.add_constraint_clauses {
                clauses.push(clause)
            }
        }

        for l_tag in info.matched_literals.iter().copied() {
            let clause = Clause::from_ordered_set(vec![l_tag, !x]);
            definition_clauses.push(clause.clone());
            if parameters.add_definition_clauses {
                clauses.push(clause);
            }
        }

        let removed_clauses: Vec<Clause> = to_remove.iter().map(|i| clauses[*i].clone()).collect();
        if parameters.remove_previous_clauses {
            Utils::remove_indexes(clauses, &to_remove);
        }

        VariableAddition {
            variable: Some(variable),
            constraint_clauses,
            matched_literals: info.matched_literals,
            definition_clauses,
            removed_clauses,
            removed_clauses_indexes: to_remove,
        }
    }

    /// Bounded Variable Addition (BVA)
    /// This version of BVA is static, meaning that it does not require the CNF to be a CNF object.
    ///
    /// For more information checkout `simple_bounded_variable_addition`
    ///
    /// # Example
    ///
    /// ```
    /// use rust_formal_verification::formulas::{Variable, Literal, Clause, CNF, cnf::bounded_variable_addition_1::BoundedVariableAdditionParameters};
    /// let a = Literal::new(Variable::new(1));
    /// let b = Literal::new(Variable::new(2));
    /// let c = Literal::new(Variable::new(3));
    /// let d = Literal::new(Variable::new(4));
    /// let e = Literal::new(Variable::new(5));
    /// let x = Literal::new(Variable::new(6));
    ///
    /// let mut clauses = vec![
    ///     Clause::from_ordered_set(vec![a, c]),
    ///     Clause::from_ordered_set(vec![a, d]),
    ///     Clause::from_ordered_set(vec![a, e]),
    ///     Clause::from_ordered_set(vec![b, c]),
    ///     Clause::from_ordered_set(vec![b, d]),
    ///     Clause::from_ordered_set(vec![b, e]),
    /// ];
    ///
    /// let mut par = BoundedVariableAdditionParameters::new();
    /// par.new_variable_getter = Box::new(|_| Variable::new(6));
    /// CNF::static_simple_bounded_variable_addition(&mut clauses, par);
    ///
    /// assert_eq!(clauses, vec![
    ///     Clause::from_ordered_set(vec![c, x]),
    ///     Clause::from_ordered_set(vec![d, x]),
    ///     Clause::from_ordered_set(vec![e, x]),
    ///     Clause::from_ordered_set(vec![a, !x]),
    ///     Clause::from_ordered_set(vec![b, !x]),
    /// ]);
    /// ```
    pub fn static_simple_bounded_variable_addition(
        clauses: &mut Vec<Clause>,
        mut parameters: BoundedVariableAdditionParameters,
    ) -> Vec<VariableAddition> {
        debug_assert!(
            {
                let mut v = clauses.to_owned();
                v.sort();
                v.dedup();
                v.len() == clauses.len()
            },
            "The clauses provided to BVA must be unique."
        );
        let mut introduced_variables = vec![];
        let mut occurences = Self::static_get_literal_occurences(clauses);
        let mut q = Self::get_literal_queue(&occurences);
        while !q.is_empty() {
            let l = q.pop().unwrap().literal;
            let length_before = clauses.len();
            let introduced = Self::try_simple_bounded_variable_addition_on_l(
                clauses,
                &occurences,
                l,
                &mut parameters,
            );
            if let Some(va) = introduced {
                let new_variable = va.variable;
                introduced_variables.push(va);
                let length_after = clauses.len();
                debug_assert!(if parameters.remove_previous_clauses {
                    length_after < length_before
                } else {
                    true
                });

                // early stopping
                if introduced_variables.len() >= parameters.max_variable_introductions {
                    break;
                }

                // update parameters for next iteration after having introduced the variable
                drop(occurences);
                occurences = Self::static_get_literal_occurences(clauses);
                q = Self::update_literal_queue(q, &occurences, l, new_variable, &parameters);
            }
        }
        introduced_variables
    }

    /// Bounded Variable Addition (BVA) as implemented in:
    /// <https://fmv.jku.at/bva/>
    ///
    /// We also took into consideration the heuristic from the paper:
    /// <https://arxiv.org/abs/2307.01904>
    ///
    /// (We did not implement the actual heuristic, but a simpler alternative)
    pub fn simple_bounded_variable_addition(
        &mut self,
        parameters: BoundedVariableAdditionParameters,
    ) -> Vec<VariableAddition> {
        let mut clauses = std::mem::take(&mut self.clauses).unpack();
        let vars = Self::static_simple_bounded_variable_addition(&mut clauses, parameters);
        self.clauses = UniqueSortedVec::from_sequence(clauses);
        self.max_variable_number = self
            .clauses
            .iter()
            .map(|c| c.max_variable())
            .max()
            .unwrap_or(Variable::new(0));
        vars
    }
}
