// ************************************************************************************************
// use
// ************************************************************************************************

use crate::{
    formulas::{Clause, Variable},
    models::Utils,
};

use super::power_set_iterator::PowerSetIterator;

// ************************************************************************************************
// struct
// ************************************************************************************************

type NegationsEntry = u32;

pub struct ClauseCombinationsIterator {
    iter: PowerSetIterator<Variable>,
    last_vars: Vec<Variable>,
    negations: NegationsEntry,
    negation_mask: NegationsEntry,
    is_done: bool,
}

// ************************************************************************************************
// impl
// ************************************************************************************************

impl ClauseCombinationsIterator {
    pub fn new(variable_to_build_clauses_over: Vec<Variable>, clause_size: usize) -> Self {
        let is_done = !(0 < clause_size && clause_size <= variable_to_build_clauses_over.len());
        debug_assert!(clause_size <= (NegationsEntry::BITS as usize));
        debug_assert!(Utils::is_sorted_and_unique(&variable_to_build_clauses_over));

        let iter = PowerSetIterator::new(variable_to_build_clauses_over, clause_size);
        let mut negation_mask = NegationsEntry::MAX;
        negation_mask <<= clause_size;
        negation_mask = !negation_mask;
        Self {
            iter,
            last_vars: vec![],
            negations: NegationsEntry::MIN,
            negation_mask,
            is_done,
        }
    }
}

// ************************************************************************************************
// Iterator
// ************************************************************************************************

impl Iterator for ClauseCombinationsIterator {
    type Item = Clause;

    fn next(&mut self) -> Option<Self::Item> {
        if self.is_done {
            return None;
        } else if self.last_vars.is_empty() || (self.negations == self.negation_mask + 1) {
            self.last_vars = self.iter.next()?;
            self.negations = NegationsEntry::MIN;
        }

        let mut clause = Vec::with_capacity(self.last_vars.len());
        for (i, var) in self.last_vars.iter().rev().enumerate().rev() {
            let negation = (self.negations >> i & 1) == 1;
            clause.push(var.literal(negation));
        }

        self.negations += 1;

        Some(Clause::from_ordered_set(clause))
    }
}

// ************************************************************************************************
// fmt
// ************************************************************************************************

#[test]
fn test() {
    let v1 = Variable::new(1);
    let v2 = Variable::new(2);
    let v3 = Variable::new(3);
    let v4 = Variable::new(4);
    let variables_to_build_clauses_on = vec![v1, v2, v3, v4];

    let l1 = v1.literal(false);
    let l2 = v2.literal(false);
    let l3 = v3.literal(false);
    let l4 = v4.literal(false);

    let mut iter = ClauseCombinationsIterator::new(variables_to_build_clauses_on.clone(), 0);
    assert_eq!(iter.next(), None);

    let mut iter = ClauseCombinationsIterator::new(variables_to_build_clauses_on.clone(), 1);
    assert_eq!(iter.next(), Some(Clause::from_ordered_set(vec![l1])));
    assert_eq!(iter.next(), Some(Clause::from_ordered_set(vec![!l1])));
    assert_eq!(iter.next(), Some(Clause::from_ordered_set(vec![l2])));
    assert_eq!(iter.next(), Some(Clause::from_ordered_set(vec![!l2])));
    assert_eq!(iter.next(), Some(Clause::from_ordered_set(vec![l3])));
    assert_eq!(iter.next(), Some(Clause::from_ordered_set(vec![!l3])));
    assert_eq!(iter.next(), Some(Clause::from_ordered_set(vec![l4])));
    assert_eq!(iter.next(), Some(Clause::from_ordered_set(vec![!l4])));
    assert_eq!(iter.next(), None);

    let mut iter = ClauseCombinationsIterator::new(variables_to_build_clauses_on.clone(), 2);
    assert_eq!(iter.next(), Some(Clause::from_ordered_set(vec![l1, l2])));
    assert_eq!(iter.next(), Some(Clause::from_ordered_set(vec![l1, !l2])));
    assert_eq!(iter.next(), Some(Clause::from_ordered_set(vec![!l1, l2])));
    assert_eq!(iter.next(), Some(Clause::from_ordered_set(vec![!l1, !l2])));

    assert_eq!(iter.next(), Some(Clause::from_ordered_set(vec![l1, l3])));
    assert_eq!(iter.next(), Some(Clause::from_ordered_set(vec![l1, !l3])));
    assert_eq!(iter.next(), Some(Clause::from_ordered_set(vec![!l1, l3])));
    assert_eq!(iter.next(), Some(Clause::from_ordered_set(vec![!l1, !l3])));

    assert_eq!(iter.next(), Some(Clause::from_ordered_set(vec![l1, l4])));
    assert_eq!(iter.next(), Some(Clause::from_ordered_set(vec![l1, !l4])));
    assert_eq!(iter.next(), Some(Clause::from_ordered_set(vec![!l1, l4])));
    assert_eq!(iter.next(), Some(Clause::from_ordered_set(vec![!l1, !l4])));

    assert_eq!(iter.next(), Some(Clause::from_ordered_set(vec![l2, l3])));
    assert_eq!(iter.next(), Some(Clause::from_ordered_set(vec![l2, !l3])));
    assert_eq!(iter.next(), Some(Clause::from_ordered_set(vec![!l2, l3])));
    assert_eq!(iter.next(), Some(Clause::from_ordered_set(vec![!l2, !l3])));

    assert_eq!(iter.next(), Some(Clause::from_ordered_set(vec![l2, l4])));
    assert_eq!(iter.next(), Some(Clause::from_ordered_set(vec![l2, !l4])));
    assert_eq!(iter.next(), Some(Clause::from_ordered_set(vec![!l2, l4])));
    assert_eq!(iter.next(), Some(Clause::from_ordered_set(vec![!l2, !l4])));

    assert_eq!(iter.next(), Some(Clause::from_ordered_set(vec![l3, l4])));
    assert_eq!(iter.next(), Some(Clause::from_ordered_set(vec![l3, !l4])));
    assert_eq!(iter.next(), Some(Clause::from_ordered_set(vec![!l3, l4])));
    assert_eq!(iter.next(), Some(Clause::from_ordered_set(vec![!l3, !l4])));

    assert_eq!(iter.next(), None);

    let mut iter = ClauseCombinationsIterator::new(variables_to_build_clauses_on.clone(), 4);
    assert_eq!(
        iter.next(),
        Some(Clause::from_ordered_set(vec![l1, l2, l3, l4]))
    );
    assert_eq!(
        iter.next(),
        Some(Clause::from_ordered_set(vec![l1, l2, l3, !l4]))
    );
    assert_eq!(
        iter.next(),
        Some(Clause::from_ordered_set(vec![l1, l2, !l3, l4]))
    );
    assert_eq!(
        iter.next(),
        Some(Clause::from_ordered_set(vec![l1, l2, !l3, !l4]))
    );
    assert_eq!(
        iter.next(),
        Some(Clause::from_ordered_set(vec![l1, !l2, l3, l4]))
    );
    assert_eq!(
        iter.next(),
        Some(Clause::from_ordered_set(vec![l1, !l2, l3, !l4]))
    );
    assert_eq!(
        iter.next(),
        Some(Clause::from_ordered_set(vec![l1, !l2, !l3, l4]))
    );
    assert_eq!(
        iter.next(),
        Some(Clause::from_ordered_set(vec![l1, !l2, !l3, !l4]))
    );
    assert_eq!(
        iter.next(),
        Some(Clause::from_ordered_set(vec![!l1, l2, l3, l4]))
    );
    assert_eq!(
        iter.next(),
        Some(Clause::from_ordered_set(vec![!l1, l2, l3, !l4]))
    );
    assert_eq!(
        iter.next(),
        Some(Clause::from_ordered_set(vec![!l1, l2, !l3, l4]))
    );
    assert_eq!(
        iter.next(),
        Some(Clause::from_ordered_set(vec![!l1, l2, !l3, !l4]))
    );
    assert_eq!(
        iter.next(),
        Some(Clause::from_ordered_set(vec![!l1, !l2, l3, l4]))
    );
    assert_eq!(
        iter.next(),
        Some(Clause::from_ordered_set(vec![!l1, !l2, l3, !l4]))
    );
    assert_eq!(
        iter.next(),
        Some(Clause::from_ordered_set(vec![!l1, !l2, !l3, l4]))
    );
    assert_eq!(
        iter.next(),
        Some(Clause::from_ordered_set(vec![!l1, !l2, !l3, !l4]))
    );
    assert_eq!(iter.next(), None);

    let mut iter = ClauseCombinationsIterator::new(variables_to_build_clauses_on.clone(), 5);
    assert_eq!(iter.next(), None);
}
