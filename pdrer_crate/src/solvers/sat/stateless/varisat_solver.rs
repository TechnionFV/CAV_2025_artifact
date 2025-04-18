// ************************************************************************************************
// use
// ************************************************************************************************

use crate::formulas::CNF;
use crate::solvers::sat::StatelessSatSolver;
use varisat::{ExtendFormula, Lit, Solver};

use super::{Assignment, StatelessResponse};

// use super::{Assignment, SatSolver};
// use std::time;

// ************************************************************************************************
// struct
// ************************************************************************************************

#[derive(Default, Clone, Copy)]
pub struct VarisatSolver {}

// ************************************************************************************************
// impl
// ************************************************************************************************

impl VarisatSolver {
    fn convert_cnf_to_varisat(cnf_to_solve: &CNF, solver_to_add_to: &mut Solver) {
        for clause in cnf_to_solve.iter() {
            let mut varisat_literals = Vec::new();
            for lit in clause.iter() {
                let number: isize = lit.variable().number().try_into().unwrap();
                let signed_number = if lit.is_negated() { -number } else { number };
                varisat_literals.push(Lit::from_dimacs(signed_number));
            }
            solver_to_add_to.add_clause(&varisat_literals);
        }
    }

    fn varisat_model_to_dimacs_assignment(assignment: &[varisat::Lit]) -> Vec<i32> {
        assignment
            .iter()
            .map(|l| l.to_dimacs().try_into().unwrap())
            .collect::<Vec<i32>>()
    }

    pub fn solve_cnf(&self, cnf_to_solve: &CNF) -> StatelessResponse {
        let mut solver = Solver::new();
        Self::convert_cnf_to_varisat(cnf_to_solve, &mut solver);

        // let start_time = time::Instant::now();
        // println!("Sat solver call - start!");
        let sat_call_response = solver.solve();
        // println!("Sat solver call - end! Duration was {} seconds.", start_time.elapsed().as_secs_f32());
        match sat_call_response {
            Ok(is_sat) => match is_sat {
                true => StatelessResponse::Sat {
                    assignment: Assignment::from_dimacs_assignment(
                        &Self::varisat_model_to_dimacs_assignment(&solver.model().unwrap()),
                    ),
                },
                false => StatelessResponse::UnSat {},
            },
            Err(_) => {
                panic!();
            }
        }
    }
}

// ************************************************************************************************
// impl trait
// ************************************************************************************************

impl StatelessSatSolver for VarisatSolver {
    fn solve_cnf(&self, cnf_to_solve: &CNF) -> StatelessResponse {
        self.solve_cnf(cnf_to_solve)
    }
}
