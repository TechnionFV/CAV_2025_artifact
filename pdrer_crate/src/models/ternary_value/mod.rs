// ************************************************************************************************
// use
// ************************************************************************************************

use std::fmt;

// ************************************************************************************************
// struct
// ************************************************************************************************

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TernaryValue {
    X,
    False,
    True,
}

// ************************************************************************************************
// impl
// ************************************************************************************************

// ************************************************************************************************
// printing
// ************************************************************************************************

impl fmt::Display for TernaryValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TernaryValue::True => write!(f, "1"),
            TernaryValue::False => write!(f, "0"),
            TernaryValue::X => write!(f, "X"),
        }
    }
}
