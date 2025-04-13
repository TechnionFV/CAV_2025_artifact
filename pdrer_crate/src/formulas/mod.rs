//! representation of boolean logic formulas.
//!
//! This module houses the most basic representation of formulas for the library.
//! This includes how a literal is formed, how clauses and cubes are represented, and finally
//! how a cnf is represented.

// ************************************************************************************************
// rust submodule declaration, they get searched in their respective file  names
// ************************************************************************************************

pub mod clause; // requires file in this directory with the name 'clause.rs'
pub mod cnf; // requires file in this directory with the name 'cnf.rs'
pub mod cube;
pub mod literal; // requires file in this directory with the name 'literal.rs'
pub mod variable;

// ************************************************************************************************
// re-exports of structs in these modules to simplify paths for other imports
// ************************************************************************************************

pub use clause::Clause;
pub use cnf::CNF;
pub use cube::Cube;
pub use literal::Literal;
pub use variable::Variable;
// pub use variable::Variable;
