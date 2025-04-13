// ************************************************************************************************
// use
// ************************************************************************************************

use std::fmt::{self, Debug};

use crate::models::unique_sorted_hash_map::UniqueSortedHash;

use super::Literal;

// ************************************************************************************************
// struct
// ************************************************************************************************

#[derive(Clone, PartialEq, Eq, Hash, Copy, PartialOrd, Ord)]
pub struct Variable {
    number: u32,
}

// ************************************************************************************************
// impl
// ************************************************************************************************

impl Variable {
    pub fn new(number: u32) -> Self {
        Self { number }
    }

    pub fn number(&self) -> u32 {
        self.number
    }

    pub fn literal(&self, is_negated: bool) -> Literal {
        if is_negated {
            !Literal::new(*self)
        } else {
            Literal::new(*self)
        }
    }

    pub fn bump(&mut self, delta: i32) {
        let new_var_number: u32 = self.number.checked_add_signed(delta).unwrap();
        debug_assert!(new_var_number > 0);
        self.number = new_var_number;
    }
}

// ************************************************************************************************
// printing
// ************************************************************************************************

impl fmt::Display for Variable {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.number)
    }
}

impl Debug for Variable {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self)
    }
}

// ************************************************************************************************
// impl UniqueSortedHash
// ************************************************************************************************

impl UniqueSortedHash for Variable {
    fn hash(&self) -> usize {
        self.number as usize
    }

    fn un_hash(i: usize) -> Self {
        Variable::new(i as u32)
    }
}
