//! module that holds sat solvers that are not mutable and don't hold a state.

// ************************************************************************************************
// rust submodule declaration, they get searched in their respective file  names
// ************************************************************************************************

pub mod assignment;
pub mod cadical_solver;
pub mod varisat_solver;

// ************************************************************************************************
// re-exports of structs in these modules to simplify paths for other imports
// ************************************************************************************************

pub use assignment::Assignment;
pub use varisat_solver::VarisatSolver;

// ************************************************************************************************
// use
// ************************************************************************************************

use crate::formulas::CNF;

// ************************************************************************************************
// enum
// ************************************************************************************************

#[derive(PartialEq, Eq)]
pub enum StatelessResponse {
    Sat { assignment: Assignment },
    UnSat,
}

// ************************************************************************************************
// Sat Solver trait
// ************************************************************************************************

pub trait StatelessSatSolver: Default {
    fn solve_cnf(&self, cnf_to_solve: &CNF) -> StatelessResponse;
}
