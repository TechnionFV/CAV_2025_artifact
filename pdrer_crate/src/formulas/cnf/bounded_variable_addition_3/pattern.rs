// ************************************************************************************************
// use
// ************************************************************************************************

use crate::formulas::{Literal, Variable};
use std::fmt::Debug;

// ************************************************************************************************
// parameters
// ************************************************************************************************

#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub enum BVA3Pattern {
    AndPattern(Literal, Literal),
    XorPattern(Variable, Variable),
    HalfAdderPattern(Literal, Literal, Literal, Literal),
}

// ************************************************************************************************
// impl
// ************************************************************************************************

impl Debug for BVA3Pattern {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AndPattern(a, b) => {
                write!(f, "AndPattern({}, {})", a, b)
            }
            Self::XorPattern(a, b) => {
                write!(f, "XorPattern({}, {})", a, b)
            }
            Self::HalfAdderPattern(a, b, c, d) => {
                write!(f, "HalfAdderPattern({}, {}, {}, {})", a, b, c, d)
            }
        }
    }
}
