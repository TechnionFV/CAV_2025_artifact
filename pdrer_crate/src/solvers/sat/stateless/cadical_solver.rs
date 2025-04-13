// ************************************************************************************************
// use
// ************************************************************************************************

use cadical_sys::Status;

use super::{Assignment, StatelessSatSolver};
use crate::formulas::CNF;
use crate::solvers::sat::incremental::CaDiCalSolver;
use crate::solvers::sat::stateless::StatelessResponse;
// use std::time;

// ************************************************************************************************
// struct
// ************************************************************************************************

// ************************************************************************************************
// impl CadicalSolver
// ************************************************************************************************

impl CaDiCalSolver {
    fn convert_cnf_to_dimacs_into_solver(cnf_to_solve: &CNF) -> Vec<Vec<i32>> {
        let mut result = Vec::new();
        for clause in cnf_to_solve.iter() {
            let mut i32_lits = Vec::new();
            for lit in clause.iter() {
                let signed_number = lit.get_dimacs_number();
                i32_lits.push(signed_number);
            }
            result.push(i32_lits);
        }
        result
    }

    pub fn solve_cnf(&self, cnf_to_solve: &CNF) -> StatelessResponse {
        let mut solver: cadical_sys::CaDiCal = Default::default();

        let dimacs_format = Self::convert_cnf_to_dimacs_into_solver(cnf_to_solve);
        dimacs_format
            .iter()
            .for_each(|clause| solver.clause6(clause));
        // let start_time = time::Instant::now();
        // println!("Sat solver call - start!");
        let sat_call_response = solver.solve();
        // println!("Sat solver call - end! Duration was {} seconds.", start_time.elapsed().as_secs_f32());
        match sat_call_response {
            Status::SATISFIABLE => {
                let dimacs_assignment = (1..(solver.vars() + 1))
                    .map(|var_num| solver.val(var_num))
                    .collect::<Vec<i32>>();

                StatelessResponse::Sat {
                    assignment: Assignment::from_dimacs_assignment(&dimacs_assignment),
                }
            }
            Status::UNSATISFIABLE => StatelessResponse::UnSat,
            Status::UNKNOWN => {
                unreachable!()
            }
        }
    }
}

// ************************************************************************************************
// impl trait
// ************************************************************************************************

impl StatelessSatSolver for CaDiCalSolver {
    fn solve_cnf(&self, cnf_to_solve: &CNF) -> StatelessResponse {
        self.solve_cnf(cnf_to_solve)
    }
}
