// ************************************************************************************************
// use
// ************************************************************************************************

use std::iter;

use super::FiniteStateTransitionSystem;
use crate::{
    formulas::{Clause, CNF},
    solvers::sat::incremental::{IncrementalSatSolver, IncrementalSolverUtils, SatResult},
};

// ************************************************************************************************
// impl
// ************************************************************************************************

impl FiniteStateTransitionSystem {
    // ********************************************************************************************
    // helper functions
    // ********************************************************************************************

    // fn solve_is_initial<T: IncrementalSatSolver>(c: &Cube, initial_solver: &mut T) -> bool {
    //     let r = initial_solver.solve(c.iter(), iter::empty());
    //     match r {
    //         IncrementalSatResponse::Sat => true, // found state that is initial and in clause
    //         IncrementalSatResponse::UnSat => false, // no such state exists
    //     }
    // }

    // fn is_initial<T: IncrementalSatSolver>(&self, c: &Cube, initial_solver: &mut T) -> bool {
    //     // make sure all clauses in initial are in cube.
    //     let r = self.is_cube_satisfied_by_some_initial_state(c);
    //     match r {
    //         Some(b) => b,
    //         None => {
    //             // cube contains non latches
    //             Self::solve_is_initial(c, initial_solver)
    //         }
    //     }
    // }

    // /// Checks if a clause is inductive relative to the lemmas.\
    // /// This is done using the sat call:\
    // /// ```text
    // /// lemmas(x) ^ clause(x) ^ Tr(x, i, x') ^ !clause(x')
    // /// ```
    // ///
    // /// UN-SAT: If the clause is inductive then an `Ok(inductive_subclause)` of it is returned (using failed information).
    // ///
    // /// SAT: Otherwise an `Err(counter_example_of_inductiveness)` is returned (using the assignment).
    // /// This counter example is a cube that satisfies the clause but has a successor that does not satisfy the clause.
    // /// If clause(x) is not satisfied by all initial states the counter example to inductiveness is None.
    // fn is_clause_inductive_relative_to_solver<T: IncrementalSatSolver>(
    //     &self,
    //     solver: &mut T,
    //     clause: Clause,
    //     initial_solver: &mut T,
    // ) -> Result<Clause, Option<Cube>> {
    //     let cube = !clause.to_owned();
    //     if self.is_initial(&cube, initial_solver) {
    //         return Err(None);
    //     }

    //     let r = IncrementalSolverUtils::call_with_clause_and_not_clause_tag(
    //         solver,
    //         clause.to_owned(),
    //         self,
    //     );

    //     match r {
    //         IncrementalSatResponse::Sat => {
    //             let counter_example_to_induction = self
    //                 .extract_variables_from_solver(solver, clause.iter().map(|l| l.get_variable()));
    //             Err(Some(counter_example_to_induction))
    //         }
    //         IncrementalSatResponse::UnSat => {
    //             // here you could use "failed" to simplify the cube even further
    //             // let cube = Cube::new(literals.to_owned());
    //             let mut clause = clause.unpack().unpack();
    //             let mut i = 0;
    //             while i < clause.len() {
    //                 let removed_literal: Literal = clause.swap_remove(i);
    //                 let tag_lit = {
    //                     let mut a = removed_literal;
    //                     self.add_tags_to_literal(&mut a, 1);
    //                     a
    //                 };
    //                 let not_clause = !Clause::new(clause.to_owned());
    //                 if self.is_initial(&not_clause, initial_solver) || !solver.failed(&tag_lit) {
    //                     clause.push(removed_literal);
    //                     let last_index = clause.len() - 1;
    //                     clause.swap(i, last_index);
    //                     i += 1;
    //                 }
    //             }
    //             let c = Clause::new(clause.to_owned());
    //             Ok(c)
    //         }
    //     }
    // }

    // fn get_subset_of_clause_that_is_inductive_relative_to_solver<T: IncrementalSatSolver>(
    //     &self,
    //     solver: &mut T,
    //     clause: Clause,
    //     initial_solver: &mut T,
    // ) -> Option<Clause> {
    //     let cube = !clause;
    //     let mut literals: Vec<Literal> = cube.iter().copied().collect();

    //     while !literals.is_empty() {
    //         let cube = Cube::new_when_sorted(literals.to_owned());
    //         if self.is_initial(&cube, initial_solver) {
    //             break;
    //         }

    //         let clause = !cube;
    //         let r = self.is_clause_inductive_relative_to_solver(solver, clause, initial_solver);

    //         match r {
    //             Ok(c) => {
    //                 return Some(c);
    //             }
    //             Err(is_initial) => {
    //                 if let Some(counter_example_to_induction) = is_initial {
    //                     let literals_to_delete = (!counter_example_to_induction).unpack();
    //                     // this can be done faster using vector "X\Y"
    //                     literals.retain(|l| !literals_to_delete.contains(l));
    //                 } else {
    //                     // cube is not inductive
    //                     return None;
    //                 }
    //             }
    //         }
    //     }

    //     // couldn't find any sub part of the cube that is inductive
    //     None
    // }

    // ********************************************************************************************
    // API
    // ********************************************************************************************

    // / Returns a subset of the clause that is inductive relative to the solver.
    // / For example, if we have the clauses:
    // / ```text
    // / (a \/ b \/ c)
    // / (d \/ e \/ f)
    // / (g \/ h \/ i)
    // / ```
    // /
    // / Then, the result could be:
    // / ```text
    // / (a \/ b)
    // / (d \/ e)
    // / ```
    // / That is if (a \/ b) ^ (d \/ e) ^ solver -> (a' \/ b') ^ (d' \/ e')
    // pub fn get_inductive_subset_relative_to_cnf_including_subset_of_clauses<
    //     T: IncrementalSatSolver,
    //     R: Rng,
    // >(
    //     &mut self,
    //     initial_solver: &mut T,
    //     mut clauses: Vec<Clause>,
    //     cnf: &CNF,
    //     rng: &mut R,
    //     verbose: bool,
    // ) -> Vec<Clause> {
    //     clauses.sort_unstable();
    //     clauses.dedup();
    //     clauses.retain(|c| !self.is_initial(&!c.to_owned(), initial_solver));

    //     for i in 1.. {
    //         if verbose {
    //             println!("F{i} = {}", CNF::new(clauses.to_vec()));
    //         }
    //         let mut literals_in_clauses: Vec<Literal> = clauses
    //             .iter()
    //             .flat_map(|c| c.iter())
    //             // .map(|l| l.get_variable())
    //             .copied()
    //             .collect();
    //         literals_in_clauses.sort_unstable();
    //         literals_in_clauses.dedup();

    //         let mut variables_in_clauses: Vec<Variable> = literals_in_clauses
    //             .iter()
    //             .map(|l| l.get_variable())
    //             .collect();
    //         variables_in_clauses.sort_unstable();
    //         variables_in_clauses.dedup();

    //         // make new solver
    //         let mut solver: T = IncrementalSolverUtils::new_solver(cnf, rng.gen());
    //         for c in clauses.iter() {
    //             IncrementalSolverUtils::add_clause_to_solver(&mut solver, c);
    //         }

    //         // find counter examples to induction
    //         let mut counter_examples_to_induction = Vec::new();
    //         for (i, clause) in clauses.iter().enumerate() {
    //             let mut not_clause_tag = !clause.to_owned();
    //             self.add_tags_to_cube(&mut not_clause_tag, 1);
    //             let r = solver.solve(not_clause_tag.iter(), iter::empty());

    //             match r {
    //                 IncrementalSatResponse::Sat => {
    //                     let cti = self.extract_variables_from_solver(
    //                         &mut solver,
    //                         variables_in_clauses.iter().copied(),
    //                     );
    //                     let mut cti = cti.unpack().unpack();
    //                     cti.retain(|l| literals_in_clauses.contains(l));
    //                     counter_examples_to_induction.push((i, Cube::new_when_sorted(cti)));
    //                 }
    //                 IncrementalSatResponse::UnSat => {}
    //             }
    //         }

    //         if counter_examples_to_induction.is_empty() {
    //             return clauses;
    //         } else {
    //             for (_, cti) in counter_examples_to_induction {
    //                 if self.is_initial(&cti, initial_solver) {
    //                     return vec![];
    //                 }
    //                 clauses.push(!cti);
    //             }
    //         }
    //     }

    //     unreachable!()
    // }

    /// Returns a subset of the clause that is inductive relative to the solver.
    /// For example, if we have the clauses:
    /// ```text
    /// (a \/ b \/ c)
    /// (d \/ e \/ f)
    /// (g \/ h \/ i)
    /// ```
    ///
    /// Then, the result could be:
    /// ```text
    /// (a \/ b \/ c)
    /// (d \/ e \/ f)
    /// ```
    /// That is if (a \/ b \/ c) ^ (d \/ e \/ f) ^ solver -> (a' \/ b' \/ c') ^ (d' \/ e' \/ f')
    pub fn get_inductive_subset_relative_to_cnf<T: IncrementalSatSolver>(
        &self,
        mut clauses: Vec<Clause>,
        cnf: &CNF,
        verbose: bool,
    ) -> Vec<Clause> {
        clauses.sort_unstable();
        clauses.dedup();
        if verbose {
            println!("F0 = {}", clauses.len());
        }
        let mut i = 0;
        loop {
            i += 1;
            let new_clauses = self.get_proved_subset_relative_to_set_and_cnf::<T>(&clauses, cnf);
            if verbose {
                println!("F{i} = {}", new_clauses.len());
            }
            if new_clauses.len() == clauses.len() {
                debug_assert_eq!(new_clauses, clauses);
                if verbose {
                    println!("F{i} = F_inf");
                }
                return new_clauses;
            } else if new_clauses.is_empty() {
                return new_clauses;
            } else {
                clauses = new_clauses;
            }
        }
    }

    pub fn get_proved_subset_relative_to_set_and_cnf<T: IncrementalSatSolver>(
        &self,
        clauses: &[Clause],
        cnf: &CNF,
    ) -> Vec<Clause> {
        let mut solver: T = IncrementalSolverUtils::new_solver(cnf, 1234567890);
        for c in clauses.iter() {
            IncrementalSolverUtils::add_clause_to_solver(&mut solver, c);
        }

        let mut result = Vec::with_capacity(clauses.len());

        for clause in clauses.iter() {
            let can_be_proved_by_clauses = {
                let mut not_clause = !clause.to_owned();
                self.add_tags_to_cube(&mut not_clause, 1);
                let r = solver.solve(not_clause.iter().copied(), iter::empty());
                match r {
                    SatResult::Sat => false,
                    SatResult::UnSat => true,
                }
            };

            if can_be_proved_by_clauses {
                result.push(clause.to_owned());
            }
        }

        result
    }
}
