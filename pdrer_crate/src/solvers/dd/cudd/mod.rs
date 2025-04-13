//! module that holds objects and traits that performing decision diagrams in various ways.

// ************************************************************************************************
// rust submodule declaration, they get searched in their respective file  names
// ************************************************************************************************

pub mod cudd_base;
pub mod cudd_bdd;
pub mod cudd_zdd;

// ************************************************************************************************
// re-exports of structs in these modules to simplify paths for other imports
// ************************************************************************************************

pub use cudd_bdd::CuddBdd;
pub use cudd_zdd::CuddZdd;
