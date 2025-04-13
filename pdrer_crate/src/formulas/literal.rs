// ************************************************************************************************
// use
// ************************************************************************************************

use crate::{formulas::Variable, models::unique_sorted_hash_map::UniqueSortedHash};
use std::{
    fmt::{self, Debug},
    ops::Not,
};

// ************************************************************************************************
// struct
// ************************************************************************************************

#[derive(Clone, PartialEq, Eq, Hash, Copy)]
pub struct Literal {
    literal_number: i32,
}

// ************************************************************************************************
// impl
// ************************************************************************************************

impl Literal {
    /// Function that lets us make a literal
    pub fn new(number: Variable) -> Self {
        debug_assert!(number.number() > 0, "Literal number may not be zero.");
        debug_assert!(
            number.number().leading_zeros() > 0,
            "Literal number is too big."
        );
        Self {
            literal_number: number.number() as i32,
        }
    }

    /// Get the variable for this literal
    pub fn variable(&self) -> Variable {
        Variable::new(self.literal_number.unsigned_abs())
    }

    /// Get the DIMACS representation of this literal
    pub fn get_dimacs_number(&self) -> i32 {
        self.literal_number
    }

    pub fn from_dimacs_number(dimacs_number: i32) -> Self {
        Self {
            literal_number: dimacs_number,
        }
    }

    // /// Construct a negated version of this literal if the
    // /// boolean is true, otherwise return a copy
    // pub fn negate_if_true(&self, is_negated: bool) -> Self {
    //     if is_negated {
    //         !self.to_owned()
    //     } else {
    //         self.to_owned()
    //     }
    // }

    pub fn is_negated(&self) -> bool {
        self.literal_number.is_negative()
    }

    pub fn negate_if_true(&self, is_negated: bool) -> Self {
        if is_negated {
            !self.to_owned()
        } else {
            self.to_owned()
        }
    }

    pub fn bump(&mut self, delta: i32) {
        let new_var_number: u32 = (self.literal_number.unsigned_abs())
            .checked_add_signed(delta)
            .unwrap();
        let is_negated = self.is_negated();
        debug_assert!(new_var_number > 0);
        self.literal_number = if is_negated {
            -(new_var_number as i32)
        } else {
            new_var_number as i32
        };
    }
}

// ************************************************************************************************
// negation
// ************************************************************************************************

impl Not for Literal {
    type Output = Self;

    fn not(mut self) -> Self::Output {
        self.literal_number = -self.literal_number;
        self
    }
}

// ************************************************************************************************
// printing
// ************************************************************************************************

impl fmt::Display for Literal {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.literal_number)
    }
}

// ************************************************************************************************
// Ordering
// ************************************************************************************************

/// implementation of `PartialOrd` for `Literal`.
/// Under this order we compare first using the absolute value of the literal number, then the sign.
/// This is useful for when going over a vector of sorted variables and creating a literal for each.
/// That way the created vector will also be sorted.
///
/// ```
/// use rust_formal_verification::{formulas::{Literal, Variable}, models::Utils};
/// let x1 = Variable::new(1).literal(false);
/// let x2 = Variable::new(2).literal(false);
/// let x3 = Variable::new(3).literal(false);
/// let x4 = Variable::new(4).literal(false);
/// assert!(Utils::is_sorted(&vec![x1, x2, x3, x4]));
/// assert!(Utils::is_sorted(&vec![x1, !x2, x3, !x4]));
/// assert!(Utils::is_sorted(&vec![x1, !x1, x2, !x2, x3, !x3, x4, !x4]));
///
/// assert!(!Utils::is_sorted(&vec![!x1, x1, x2, x2, x3, x3, x4, x4]));
/// assert!(!Utils::is_sorted(&vec![x1, !x2, x2, x3, x4]));
/// ```
impl PartialOrd for Literal {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Literal {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        let a = self.literal_number;
        let b = other.literal_number;
        let x = a.abs().cmp(&b.abs());
        x.then(b.cmp(&a))
    }
}

impl Debug for Literal {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self)
    }
}

// ************************************************************************************************
// impl UniqueSortedHash
// ************************************************************************************************

impl UniqueSortedHash for Literal {
    fn hash(&self) -> usize {
        ((self.literal_number.unsigned_abs() << 1) | (self.is_negated() as u32)) as usize
    }

    fn un_hash(i: usize) -> Self {
        let var = Variable::new((i >> 1) as u32);
        var.literal(i % 2 == 1)
    }
}

// ************************************************************************************************
// tests
// ************************************************************************************************

#[cfg(test)]
mod tests {
    use crate::models::Utils;

    use super::*;

    #[test]
    fn test_literal_ordering() {
        let x1 = Variable::new(1).literal(false);
        let x2 = Variable::new(2).literal(false);
        let x3 = Variable::new(3).literal(false);
        let x4 = Variable::new(4).literal(false);
        assert!(Utils::is_sorted(&[x1, x2, x3, x4]));
        assert!(Utils::is_sorted(&[x1, !x2, x3, !x4]));
        assert!(Utils::is_sorted(&[x1, !x1, x2, !x2, x3, !x3, x4, !x4]));

        assert!(!Utils::is_sorted(&[!x1, x1, x2, x2, x3, x3, x4, x4]));
        assert!(!Utils::is_sorted(&[x1, !x2, x2, x3, x4]));
    }

    #[test]
    fn test_literal_bump() {
        let mut x1 = Variable::new(1).literal(false);
        let mut x2 = Variable::new(2).literal(false);
        let mut x3 = Variable::new(3).literal(false);
        let mut x4 = Variable::new(4).literal(false);
        assert_eq!(x1.variable(), Variable::new(1));
        assert_eq!(x2.variable(), Variable::new(2));
        assert_eq!(x3.variable(), Variable::new(3));
        assert_eq!(x4.variable(), Variable::new(4));

        x1.bump(1);
        x2.bump(2);
        x3.bump(3);
        x4.bump(4);
        assert_eq!(x1.variable(), Variable::new(2));
        assert_eq!(x2.variable(), Variable::new(4));
        assert_eq!(x3.variable(), Variable::new(6));
        assert_eq!(x4.variable(), Variable::new(8));

        x1.bump(-1);
        x2.bump(-2);
        x3.bump(-3);
        x4.bump(-4);
        assert_eq!(x1.variable(), Variable::new(1));
        assert_eq!(x2.variable(), Variable::new(2));
        assert_eq!(x3.variable(), Variable::new(3));
        assert_eq!(x4.variable(), Variable::new(4));

        x1.bump(1);
        x2.bump(2);
        x3.bump(3);
        x4.bump(4);
        assert_eq!(x1.variable(), Variable::new(2));
        assert_eq!(x2.variable(), Variable::new(4));
        assert_eq!(x3.variable(), Variable::new(6));
        assert_eq!(x4.variable(), Variable::new(8));

        x1.bump(-1);
        x2.bump(-2);
        x3.bump(-3);
        x4.bump(-4);
        assert_eq!(x1.variable(), Variable::new(1));
        assert_eq!(x2.variable(), Variable::new(2));
        assert_eq!(x3.variable(), Variable::new(3));
        assert_eq!(x4.variable(), Variable::new(4));
    }

    #[test]
    fn test_literal_negation() {
        let x1 = Variable::new(1).literal(false);
        let x2 = Variable::new(2).literal(false);
        let x3 = Variable::new(3).literal(false);
        let x4 = Variable::new(4).literal(false);
        assert_eq!(x1.variable(), Variable::new(1));
        assert_eq!(x2.variable(), Variable::new(2));
        assert_eq!(x3.variable(), Variable::new(3));
        assert_eq!(x4.variable(), Variable::new(4));

        let x1_not = !x1;
        let x2_not = !x2;
        let x3_not = !x3;
        let x4_not = !x4;
        assert_eq!(x1_not.variable(), Variable::new(1));
        assert_eq!(x2_not.variable(), Variable::new(2));
        assert_eq!(x3_not.variable(), Variable::new(3));
        assert_eq!(x4_not.variable(), Variable::new(4));

        let x1_not_not = !x1_not;
        let x2_not_not = !x2_not;
        let x3_not_not = !x3_not;
        let x4_not_not = !x4_not;
        assert_eq!(x1_not_not.variable(), Variable::new(1));
        assert_eq!(x2_not_not.variable(), Variable::new(2));
        assert_eq!(x3_not_not.variable(), Variable::new(3));
        assert_eq!(x4_not_not.variable(), Variable::new(4));

        assert_eq!(x1_not_not, x1);
        assert_eq!(x2_not_not, x2);
        assert_eq!(x3_not_not, x3);
        assert_eq!(x4_not_not, x4);
    }

    #[test]
    fn test_literal_creation() {
        let x1 = Variable::new(1).literal(false);
        let x2 = Variable::new(2).literal(false);
        let x3 = Variable::new(3).literal(false);
        let x4 = Variable::new(4).literal(false);
        assert_eq!(x1.variable(), Variable::new(1));
        assert_eq!(x2.variable(), Variable::new(2));
        assert_eq!(x3.variable(), Variable::new(3));
        assert_eq!(x4.variable(), Variable::new(4));

        let x1_not = Variable::new(1).literal(true);
        let x2_not = Variable::new(2).literal(true);
        let x3_not = Variable::new(3).literal(true);
        let x4_not = Variable::new(4).literal(true);
        assert_eq!(x1_not.variable(), Variable::new(1));
        assert_eq!(x2_not.variable(), Variable::new(2));
        assert_eq!(x3_not.variable(), Variable::new(3));
        assert_eq!(x4_not.variable(), Variable::new(4));

        assert_eq!(x1_not, !x1);
        assert_eq!(x2_not, !x2);
        assert_eq!(x3_not, !x3);
        assert_eq!(x4_not, !x4);
    }

    #[test]
    fn test_literal_to_string() {
        let x1 = Variable::new(1).literal(false);
        let x2 = Variable::new(2).literal(true);
        let x3 = Variable::new(3).literal(false);
        let x4 = Variable::new(4).literal(true);
        assert_eq!(x1.to_string(), ("1"));
        assert_eq!(x2.to_string(), ("-2"));
        assert_eq!(x3.to_string(), ("3"));
        assert_eq!(x4.to_string(), ("-4"));

        let x1_not = !x1;
        let x2_not = !x2;
        let x3_not = !x3;
        let x4_not = !x4;
        assert_eq!(x1_not.to_string(), ("-1"));
        assert_eq!(x2_not.to_string(), ("2"));
        assert_eq!(x3_not.to_string(), ("-3"));
        assert_eq!(x4_not.to_string(), ("4"));
    }

    #[test]
    fn test_literal_dimacs_number() {
        let x1 = Variable::new(1).literal(false);
        let x2 = Variable::new(2).literal(true);
        let x3 = Variable::new(3).literal(false);
        let x4 = Variable::new(4).literal(true);
        assert_eq!(x1.get_dimacs_number(), (1));
        assert_eq!(x2.get_dimacs_number(), (-2));
        assert_eq!(x3.get_dimacs_number(), (3));
        assert_eq!(x4.get_dimacs_number(), (-4));

        let x1_not = !x1;
        let x2_not = !x2;
        let x3_not = !x3;
        let x4_not = !x4;
        assert_eq!(x1_not.get_dimacs_number(), (-1));
        assert_eq!(x2_not.get_dimacs_number(), (2));
        assert_eq!(x3_not.get_dimacs_number(), (-3));
        assert_eq!(x4_not.get_dimacs_number(), (4));
    }

    #[test]
    fn test_literal_is_negated() {
        let x1 = Variable::new(1).literal(false);
        let x2 = Variable::new(2).literal(true);
        let x3 = Variable::new(3).literal(false);
        let x4 = Variable::new(4).literal(true);
        assert!(!x1.is_negated());
        assert!(x2.is_negated());
        assert!(!x3.is_negated());
        assert!(x4.is_negated());

        let x1_not = !x1;
        let x2_not = !x2;
        let x3_not = !x3;
        let x4_not = !x4;
        assert!(x1_not.is_negated());
        assert!(!x2_not.is_negated());
        assert!(x3_not.is_negated());
        assert!(!x4_not.is_negated());
    }

    #[test]
    fn test_literal_get_variable() {
        let x1 = Variable::new(1).literal(false);
        let x2 = Variable::new(2).literal(true);
        let x3 = Variable::new(3).literal(false);
        let x4 = Variable::new(4).literal(true);
        assert_eq!(x1.variable(), Variable::new(1));
        assert_eq!(x2.variable(), Variable::new(2));
        assert_eq!(x3.variable(), Variable::new(3));
        assert_eq!(x4.variable(), Variable::new(4));

        let x1_not = !x1;
        let x2_not = !x2;
        let x3_not = !x3;
        let x4_not = !x4;
        assert_eq!(x1_not.variable(), Variable::new(1));
        assert_eq!(x2_not.variable(), Variable::new(2));
        assert_eq!(x3_not.variable(), Variable::new(3));
        assert_eq!(x4_not.variable(), Variable::new(4));
    }

    #[test]
    #[should_panic]
    fn test_literal_creation_zero() {
        let _ = Variable::new(0).literal(false);
    }

    #[test]
    #[should_panic]
    fn test_literal_creation_too_big() {
        let _ = Variable::new(1 << 31).literal(false);
    }

    #[test]
    #[should_panic]
    fn test_literal_creation_too_big_negated() {
        let _ = Variable::new(1 << 31).literal(true);
    }

    #[test]
    fn test_literal_hash() {
        let x1 = Variable::new(1).literal(false);
        let x2 = Variable::new(2).literal(true);
        let x3 = Variable::new(3).literal(false);
        let x4 = Variable::new(4).literal(true);
        assert_eq!(x1.hash(), (2));
        assert_eq!(x2.hash(), (5));
        assert_eq!(x3.hash(), (6));
        assert_eq!(x4.hash(), (9));

        let x1_not = !x1;
        let x2_not = !x2;
        let x3_not = !x3;
        let x4_not = !x4;
        assert_eq!(x1_not.hash(), (3));
        assert_eq!(x2_not.hash(), (4));
        assert_eq!(x3_not.hash(), (7));
        assert_eq!(x4_not.hash(), (8));
    }

    #[test]
    fn test_literal_un_hash() {
        let x1 = Variable::new(1).literal(false);
        let x2 = Variable::new(2).literal(true);
        let x3 = Variable::new(3).literal(false);
        let x4 = Variable::new(4).literal(true);
        assert_eq!(Literal::un_hash(2), (x1));
        assert_eq!(Literal::un_hash(5), (x2));
        assert_eq!(Literal::un_hash(6), (x3));
        assert_eq!(Literal::un_hash(9), (x4));

        let x1_not = !x1;
        let x2_not = !x2;
        let x3_not = !x3;
        let x4_not = !x4;
        assert_eq!(Literal::un_hash(3), (x1_not));
        assert_eq!(Literal::un_hash(4), (x2_not));
        assert_eq!(Literal::un_hash(7), (x3_not));
        assert_eq!(Literal::un_hash(8), (x4_not));
    }
}
