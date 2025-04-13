// ************************************************************************************************
// use
// ************************************************************************************************

use std::iter;

use crate::{
    formulas::{Clause, Cube, CNF},
    solvers::sat::incremental::{IncrementalSatSolver, SatResult},
};

// ************************************************************************************************
// enum
// ************************************************************************************************

// pub enum CallResult {
//     Sat(Cube),
//     UnSat(Cube),
// }

// ************************************************************************************************
// struct
// ************************************************************************************************

pub struct IncrementalSolverUtils;

// ************************************************************************************************
// impl
// ************************************************************************************************

impl IncrementalSolverUtils {
    // ********************************************************************************************
    // helper functions
    // ********************************************************************************************

    // ********************************************************************************************
    // API
    // ********************************************************************************************

    pub fn new_solver<T: IncrementalSatSolver>(cnf: &CNF, seed: u64) -> T {
        let mut solver = T::new(seed);
        Self::add_cnf_to_solver(&mut solver, cnf);
        solver
    }

    pub fn add_cnf_to_solver<T: IncrementalSatSolver>(solver: &mut T, cnf: &CNF) {
        for c in cnf.iter() {
            Self::add_clause_to_solver(solver, c);
        }
    }

    // add !cube to solver
    pub fn block_cube_in_solver<T: IncrementalSatSolver>(solver: &mut T, cube: Cube) {
        let clause = !cube;
        Self::add_clause_to_solver(solver, &clause);
    }

    pub fn add_clause_to_solver<T: IncrementalSatSolver>(solver: &mut T, clause: &Clause) {
        solver.add_clause(clause.iter().copied());
    }

    /// Checks R ^ !c
    pub fn is_cube_blocked_by_solver<T: IncrementalSatSolver>(solver: &mut T, cube: &Cube) -> bool {
        // return true iff Ri ^ c == UnSat
        let sat_response = solver.solve(cube.iter().copied(), iter::empty());

        match sat_response {
            SatResult::Sat => false,
            SatResult::UnSat => true,
        }
    }

    pub fn clear_solver<T: IncrementalSatSolver>(
        solver: &mut T,
        cnf1: &CNF,
        cnf2: &CNF,
        seed: u64,
    ) {
        let mut cleared_solver = T::new(seed);
        Self::add_cnf_to_solver(&mut cleared_solver, cnf1);
        Self::add_cnf_to_solver(&mut cleared_solver, cnf2);
        // cleared_solver.simplify();
        *solver = cleared_solver;
    }
}
