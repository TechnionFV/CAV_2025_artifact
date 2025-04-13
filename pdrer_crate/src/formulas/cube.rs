// ************************************************************************************************
// use
// ************************************************************************************************

use super::{Clause, CNF};
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
pub struct Cube {
    literals: SortedVecOfLiterals,
}

// ************************************************************************************************
// impl
// ************************************************************************************************

impl Cube {
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

    pub fn new_true() -> Self {
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

    pub fn iter(&self) -> impl Iterator<Item = &Literal> {
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

    pub fn to_cnf(&self) -> CNF {
        let mut cnf = Vec::with_capacity(self.len());
        for lit in self.literals.iter() {
            cnf.push(Clause::from_ordered_set(vec![*lit]));
        }
        CNF::from_ordered_set(UniqueSortedVec::from_ordered_set(cnf))
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

impl Not for Cube {
    type Output = Clause;

    fn not(mut self) -> Self::Output {
        self.literals.negate_literals();
        Clause::from_ordered_set(self.literals.unpack().unpack())
    }
}

// ************************************************************************************************
// printing
// ************************************************************************************************

impl fmt::Display for Cube {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let string_vec = self
            .literals
            .iter()
            .map(|lit| lit.to_string())
            .collect::<Vec<String>>();
        write!(f, "q {} 0", string_vec.join(" "))
    }
}
