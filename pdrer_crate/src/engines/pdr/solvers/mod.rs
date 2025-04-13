use std::{cell::RefCell, rc::Rc};

use crate::{
    formulas::{Clause, Literal, Variable, CNF},
    solvers::sat::incremental::SatResult,
};

use super::shared_objects::SharedObjects;

// ************************************************************************************************
// Solver trait
// ************************************************************************************************

pub trait PropertyDirectedReachabilitySolver {
    /// Initialize a solver instance
    fn new(seed: u64) -> Self;

    // /// reserve variables in the solver
    // fn reserve_variables(&mut self, max_var: Variable);

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

    // /// valid in the un-sat case, checks if the constraint led to un-sat
    // fn constraint_failed(&mut self) -> bool;

    // / simplify the cnf in the solver
    // fn simplify(&mut self) -> Option<SatResult>;
}

// ************************************************************************************************
// SolverHolder
// ************************************************************************************************

#[derive(Debug)]
enum SolverHolder<T: PropertyDirectedReachabilitySolver> {
    /// One solver for all frames with the use of activation literals
    #[allow(dead_code)]
    Shared(Rc<RefCell<T>>, Vec<Variable>),
    /// One solver per frame
    Owned(Vec<T>),
}

#[derive(Debug)]
pub struct Solvers<T: PropertyDirectedReachabilitySolver> {
    /// The solvers
    h: SolverHolder<T>,
    /// indicate whether the solver must be reset
    must_reset: bool,
    /// Model variable to solver variable mapping
    var_map: Vec<Variable>,
    /// The simplified transition relation of the system
    transition: CNF,
    /// Extension variables transition relation
    ext: Vec<Clause>,
    /// shared objects across pdr implementation
    s: SharedObjects,
}

// ************************************************************************************************
// import modules
// ************************************************************************************************

pub mod cadical;
pub mod sat_calls;
pub mod solvers_api;
