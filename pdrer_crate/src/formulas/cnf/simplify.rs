// ************************************************************************************************
// use
// ************************************************************************************************

use std::mem;

use super::CNF;
use crate::{
    formulas::{Clause, Variable},
    models::UniqueSortedVec,
    solvers::sat::incremental::{CaDiCalSolver, SatResult},
};

// ************************************************************************************************
// impl
// ************************************************************************************************

impl CNF {
    // ********************************************************************************************
    // helper functions
    // ********************************************************************************************

    // ********************************************************************************************
    // API
    // ********************************************************************************************

    fn match_pass(clauses: &mut Vec<Clause>) {
        // let mut clauses = mem::take(&mut self.clauses).unpack();
        let mut i = 0;
        while i < (clauses.len() - 1) {
            let mut j = i + 1;
            while j < clauses.len() {
                // if clauses differ by only one literal
                if clauses[i].len() != clauses[j].len() {
                    j += 1;
                    continue;
                }

                let mut index_to_remove = -1;
                let mut should_skip = false;
                for (index, (x, y)) in clauses[i].iter().zip(clauses[j].iter()).enumerate() {
                    if x != y {
                        if index_to_remove == -1 {
                            index_to_remove = index as isize;
                        } else {
                            // if they differ by more than one literal the merge is not allowed
                            should_skip = true;
                            break;
                        }
                    }
                }

                if should_skip {
                    j += 1;
                    continue;
                }
                debug_assert!(index_to_remove != -1, "two clauses are identical");

                let index_to_remove: usize = index_to_remove as usize;
                clauses.swap_remove(j);
                clauses[i].remove_index(index_to_remove);
                // move on to the "next i" (i didn't change but the clause in that position did)
                j = i + 1;
            }
            i += 1;
        }
        // self.clauses = UniqueSortedVec::from_sequence(clauses);
    }

    pub fn static_simple_bounded_variable_elimination(clauses: &mut Vec<Clause>) {
        let mut previous_len = clauses.len();

        loop {
            Self::match_pass(clauses);
            if clauses.len() == previous_len {
                break;
            }
            previous_len = clauses.len();
        }
    }

    /// simplify using
    pub fn simple_bounded_variable_elimination(&mut self) {
        let mut clauses = mem::take(&mut self.clauses).unpack();
        Self::static_simple_bounded_variable_elimination(&mut clauses);
        self.clauses = UniqueSortedVec::from_sequence(clauses);
    }

    pub fn simplify_using_cadical(
        seed: u64,
        cnf: &[Clause],
        frozen: &[Variable],
        rounds: i32,
    ) -> (Vec<Clause>, Option<SatResult>) {
        let mut solver = CaDiCalSolver::new(seed);

        for c in cnf.iter() {
            solver.add_clause(c.iter().copied());
        }

        for f in frozen {
            solver.freeze(*f);
        }

        // solver.optimize(9);
        let result = solver.simplify(rounds);
        let clauses = solver.get_clauses();

        (clauses, result)

        //         let mut solver = cadical::Solver::new();
        //         for c in self.iter() {
        //             solver.add_clause(c.iter().map(|l| l.get_dimacs_number()));
        //         }
        //         for v in frozen {
        // solver.f        }

        //         cadical::Solver::simplify(&mut self)
        // todo!()
    }
}
