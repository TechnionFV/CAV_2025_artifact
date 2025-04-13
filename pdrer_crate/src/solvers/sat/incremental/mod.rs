//! module that hold sat solvers that are mutable and hold some state.

// ************************************************************************************************
// rust submodule declaration, they get searched in their respective file  names
// ************************************************************************************************

pub mod cadical_solver;
pub mod utils;

// ************************************************************************************************
// re-exports of structs in these modules to simplify paths for other imports
// ************************************************************************************************

pub use cadical_solver::CaDiCalSolver;
pub use utils::IncrementalSolverUtils;

// ************************************************************************************************
// use
// ************************************************************************************************

use crate::formulas::Literal;

// ************************************************************************************************
// Response
// ************************************************************************************************

#[derive(PartialEq, Eq, Debug)]
pub enum SatResult {
    Sat,
    UnSat,
}

// ************************************************************************************************
// Sat Solver trait
// ************************************************************************************************

pub trait IncrementalSatSolver {
    /// Initialize a solver instance
    fn new(seed: u64) -> Self;
    /// add clause to solver
    fn add_clause<I>(&mut self, clause: I)
    where
        I: IntoIterator<Item = Literal>;
    /// solver under the assumptions and constraint clause X ^ ass1 ^ ass2 ^ ... ^ (con1 \/ con2 \/ con3 \/...)
    fn solve<I, U>(&mut self, assumptions: I, constraint_clause: U) -> SatResult
    where
        I: IntoIterator<Item = Literal>,
        U: IntoIterator<Item = Literal>;
    /// valid in the sat case, retrieves a variable's truth table
    /// The returned value is `None` if the formula is satisfied
    /// regardless of the value of the literal.
    fn val(&mut self, lit: Literal) -> Option<bool>;
    /// valid in the un-sat case, checks for a failed assumption.
    /// Returns true if the literal was assumed in the last sat call and
    /// was important for concluding that the call was un sat.
    fn failed(&mut self, lit: Literal) -> bool;
    /// valid in the un-sat case, checks if the constraint led to un-sat
    fn constraint_failed(&mut self) -> bool;
    /// simplify the cnf in the solver
    fn simplify(&mut self) -> Option<SatResult>;
}
