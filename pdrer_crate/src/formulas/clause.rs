// ************************************************************************************************
// use
// ************************************************************************************************

use super::{Cube, CNF};
use crate::formulas::Literal;
use crate::formulas::Variable;
use crate::models::SortedVecOfLiterals;
use crate::models::UniqueSortedVec;
use std::fmt;
use std::hash::Hash;
use std::ops::Not;

// ************************************************************************************************
// struct
// ************************************************************************************************

#[derive(Eq, PartialEq, Clone, Hash, PartialOrd, Ord, Debug, Default)]
pub struct Clause {
    literals: SortedVecOfLiterals,
}

// ************************************************************************************************
// impl
// ************************************************************************************************

impl Clause {
    pub fn from_ordered_set(set: Vec<Literal>) -> Self {
        Self {
            literals: SortedVecOfLiterals::from_ordered_set(set),
        }
    }

    pub fn from_sequence(sequence: Vec<Literal>) -> Self {
        Self {
            literals: SortedVecOfLiterals::from_sequence(sequence),
        }
    }

    pub fn new_false() -> Self {
        Self {
            literals: SortedVecOfLiterals::from_ordered_set(vec![]),
        }
    }

    pub fn len(&self) -> usize {
        self.literals.len()
    }

    pub fn is_empty(&self) -> bool {
        self.literals.is_empty()
    }

    pub fn iter(&self) -> impl DoubleEndedIterator<Item = &Literal> + ExactSizeIterator {
        self.literals.iter()
    }

    pub fn max_variable(&self) -> Variable {
        self.literals.max_variable()
    }

    pub fn min_variable(&self) -> Variable {
        self.literals.min_variable()
    }

    pub fn max_literal(&self) -> Option<Literal> {
        self.literals.max_literal()
    }

    pub fn min_literal(&self) -> Option<Literal> {
        self.literals.min_literal()
    }

    pub fn to_cnf(self) -> CNF {
        CNF::from_ordered_set(UniqueSortedVec::from_ordered_set(vec![self]))
    }

    pub fn bump_all_literals(&mut self, delta: i32) {
        self.literals.bump_all_literals(delta);
    }

    pub fn unpack(self) -> SortedVecOfLiterals {
        self.literals
    }

    pub fn peek(&self) -> &SortedVecOfLiterals {
        &self.literals
    }

    pub fn insert(&mut self, lit: Literal) -> bool {
        self.literals.insert(lit)
    }

    pub fn remove(&mut self, lit: &Literal) -> bool {
        self.literals.remove(lit)
    }

    pub fn remove_index(&mut self, index: usize) -> Literal {
        self.literals.remove_index(index)
    }

    pub fn retain<F>(&mut self, f: F)
    where
        F: FnMut(&Literal) -> bool,
    {
        self.literals.retain(f)
    }

    pub fn contains(&self, lit: &Literal) -> bool {
        self.literals.contains(lit)
    }

    pub fn contains_variable(&self, var: &Variable) -> bool {
        self.literals.contains_variable(var)
    }

    pub fn position(&self, lit: &Literal) -> Option<usize> {
        self.literals.position(lit)
    }

    pub fn shrink_to_fit(&mut self) {
        self.literals.shrink_to_fit();
    }
}

// ************************************************************************************************
// negation
// ************************************************************************************************

impl Not for Clause {
    type Output = Cube;

    fn not(mut self) -> Self::Output {
        self.literals.negate_literals();
        Cube::from_ordered_set(self.literals.unpack().unpack())
    }
}

// ************************************************************************************************
// printing
// ************************************************************************************************

impl fmt::Display for Clause {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let string_vec = self
            .literals
            .iter()
            .map(|lit| lit.to_string())
            .collect::<Vec<String>>();
        write!(f, "{} 0", string_vec.join(" "))
    }
}

// ************************************************************************************************
// testing
// ************************************************************************************************

#[test]
fn test() {
    // make some variables
    let v1 = Variable::new(1);
    let v2 = Variable::new(2);
    let v3 = Variable::new(3);
    // make some literals
    let l1 = Literal::new(v1);
    let l2 = Literal::new(v2);
    let l3 = Literal::new(v3);
    // check that clauses repetition is removed
    let c = Clause::from_ordered_set(vec![l1, l2, l3]);
    let c2 = Clause::from_sequence(vec![l1, l2, l2, l3]);
    assert!(c == c2);
    assert!(c.len() == 3);
}
