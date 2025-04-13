// ************************************************************************************************
// use
// ************************************************************************************************

use super::Utils;
use crate::{
    formulas::{Clause, Literal, CNF},
    solvers::sat::{
        incremental::{IncrementalSatSolver, IncrementalSolverUtils, SatResult},
        stateless::Assignment,
    },
};
use std::iter;

// ************************************************************************************************
// impl
// ************************************************************************************************

impl Utils {
    /// Functions that returns true iff a -> b.
    ///
    /// # Arguments
    ///
    /// * `a` - CNF formula.
    /// * `b` - CNF formula.
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_formal_verification::formulas::{CNF, Clause, Literal, Variable};
    /// use rust_formal_verification::solvers::sat::incremental::CaDiCalSolver;
    /// use rust_formal_verification::models::Utils;
    /// let l1 = Variable::new(1).literal(false);
    /// let l2 = Variable::new(2).literal(false);
    /// let l3 = Variable::new(3).literal(false);
    /// let l4 = Variable::new(4).literal(false);
    /// let l5 = Variable::new(5).literal(false);
    /// let l6 = Variable::new(6).literal(false);
    ///
    /// let mut all_literals_are_equal = CNF::from_sequence(vec![
    ///     Clause::from_sequence(vec![l1, !l2]),
    ///     Clause::from_sequence(vec![l2, !l3]),
    ///     Clause::from_sequence(vec![l3, !l4]),
    ///     Clause::from_sequence(vec![l4, !l5]),
    ///     Clause::from_sequence(vec![l5, !l6]),
    ///     Clause::from_sequence(vec![l6, !l1]),
    /// ]);
    ///
    /// let one_and_4_are_equal = CNF::from_sequence(vec![
    ///     Clause::from_sequence(vec![l1, !l4]),
    ///     Clause::from_sequence(vec![l4, !l1])
    /// ]);
    ///
    /// assert!(Utils::does_a_imply_b::<CaDiCalSolver>(&all_literals_are_equal, &one_and_4_are_equal).expect("a is always false"));
    /// ```
    pub fn does_a_imply_b<T: IncrementalSatSolver>(a: &CNF, b: &CNF) -> Option<bool> {
        // a implies b iff a implies every clause in b
        // println!("a = {}", a);
        // println!("b = {}", b);
        let mut solver = IncrementalSolverUtils::new_solver::<T>(a, 0);

        if SatResult::UnSat == solver.solve(iter::empty(), iter::empty()) {
            return None;
        }

        for c in b.iter() {
            let not_c = !c.to_owned();
            match solver.solve(not_c.iter().copied(), iter::empty()) {
                SatResult::Sat => {
                    return Some(false);
                }
                SatResult::UnSat => {}
            }
        }
        Some(true)
    }

    pub fn is_a_and_b_satisfiable<T: IncrementalSatSolver>(a: &CNF, b: &CNF) -> bool {
        let mut solver = IncrementalSolverUtils::new_solver::<T>(a, 0);
        IncrementalSolverUtils::add_cnf_to_solver(&mut solver, b);
        match solver.solve(iter::empty(), iter::empty()) {
            SatResult::Sat => true,
            SatResult::UnSat => false,
        }
    }

    pub fn evaluate_assignment_on_literal(literal: &Literal, assignment: &Assignment) -> bool {
        let value = assignment.get_value(&literal.variable()).unwrap();
        if literal.is_negated() {
            !value
        } else {
            value
        }
    }

    pub fn evaluate_assignment_on_clause(clause: &Clause, assignment: &Assignment) -> bool {
        for literal in clause.iter() {
            if Self::evaluate_assignment_on_literal(literal, assignment) {
                return true;
            }
        }
        false
    }

    pub fn evaluate_assignment_on_cnf(cnf: &CNF, assignment: &Assignment) -> bool {
        for clause in cnf.iter() {
            if !Self::evaluate_assignment_on_clause(clause, assignment) {
                return false;
            }
        }
        true
    }
}
