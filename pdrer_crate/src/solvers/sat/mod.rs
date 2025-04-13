//! module that holds objects and traits that performing sat solving in various ways.

// ************************************************************************************************
// rust submodule declaration, they get searched in their respective file  names
// ************************************************************************************************

pub mod incremental;
pub mod stateless;

// ************************************************************************************************
// re-exports of structs in these modules to simplify paths for other imports
// ************************************************************************************************

pub use stateless::StatelessSatSolver;
