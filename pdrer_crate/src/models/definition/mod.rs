//! Object for holding a finite state transition system, and performing operations like adding
//! tags and so on.

// ************************************************************************************************
// use
// ************************************************************************************************

use super::SortedVecOfLiterals;
use crate::formulas::Variable;
use std::fmt::Display;

// ************************************************************************************************
// Definition function
// ************************************************************************************************

#[derive(Debug, Clone, PartialEq, Eq, Copy, Hash, PartialOrd, Ord)]
pub enum DefinitionFunction {
    And,
    Xor,
}

impl Display for DefinitionFunction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DefinitionFunction::And => write!(f, "AND"),
            DefinitionFunction::Xor => write!(f, "XOR"),
        }
    }
}

// ************************************************************************************************
// Definition
// ************************************************************************************************

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Definition {
    pub variable: Variable,
    pub function: DefinitionFunction,
    pub inputs: SortedVecOfLiterals,
}

// ************************************************************************************************
// rust submodule declaration, they get searched in their respective file  names
// ************************************************************************************************

pub mod api;

// ************************************************************************************************
// re-exports of structs in these modules to simplify paths for other imports
// ************************************************************************************************
