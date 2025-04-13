// ************************************************************************************************
// use
// ************************************************************************************************

use crate::formulas::Clause;
use crate::formulas::Variable;
use crate::models::UniqueSortedVec;

// ************************************************************************************************
// struct
// ************************************************************************************************

#[derive(Clone, Debug)]
pub struct CNF {
    max_variable_number: Variable,
    clauses: UniqueSortedVec<Clause>,
}

// ************************************************************************************************
// rust submodule declaration, they get searched in their respective file  names
// ************************************************************************************************

pub mod bounded_variable_addition_1;
pub mod bounded_variable_addition_2;
pub mod bounded_variable_addition_3;
pub mod bounded_variable_elimination;
pub mod construction;
pub mod implementations;
pub mod operations;
pub mod rename_variable;
pub mod simplify;

// ************************************************************************************************
// re-exports of structs in these modules to simplify paths for other imports
// ************************************************************************************************
