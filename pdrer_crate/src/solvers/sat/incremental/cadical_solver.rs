// ************************************************************************************************
// use
// ************************************************************************************************

// use std::{env, fs};

use crate::formulas::{Clause, Literal, Variable};
use cadical_sys::{CaDiCal, Status};

use super::{IncrementalSatSolver, SatResult};

// ************************************************************************************************
// struct
// ************************************************************************************************

// #[derive(Default, Clone, Copy)]
pub struct CaDiCalSolver {
    solver: CaDiCal,
}

// ************************************************************************************************
// impl SplrSolver
// ************************************************************************************************

impl CaDiCalSolver {
    // ********************************************************************************************
    // helper functions
    // ********************************************************************************************

    fn transform_one(l: Literal) -> i32 {
        l.get_dimacs_number()
    }

    fn transform<I>(clause: I) -> impl IntoIterator<Item = i32>
    where
        I: IntoIterator<Item = Literal>,
    {
        clause.into_iter().map(Self::transform_one)
    }

    // ********************************************************************************************
    // API functions
    // ********************************************************************************************

    pub fn new(seed: u64) -> Self {
        // println!("CaDiCal seed = {seed}");
        let actual_seed = (seed >> 38) as i32;
        // println!("CaDiCal actual seed = {actual_seed}");
        let mut solver = CaDiCal::default();
        let r = solver.set("seed".to_string(), actual_seed);
        assert!(r);
        Self { solver }
    }

    #[inline]
    pub fn add_clause<I>(&mut self, clause: I)
    where
        I: IntoIterator<Item = Literal>,
    {
        // print!("Adding clause:");
        for lit in Self::transform(clause) {
            debug_assert!(lit != 0 && lit != i32::MIN);
            // print!("{} ", lit);
            self.solver.add(lit);
        }
        // println!();
        self.solver.add(0);
    }

    #[inline]
    pub fn solve<I, U>(&mut self, assumptions: I, constraint_clause: U) -> SatResult
    where
        I: IntoIterator<Item = Literal>,
        U: IntoIterator<Item = Literal>,
    {
        // add all the assumptions
        for lit in Self::transform(assumptions) {
            debug_assert!(lit != 0 && lit != i32::MIN);
            self.solver.assume(lit);
        }

        // add the assumed clause and then finalize if needed.
        let mut iterations = 0;
        for lit in Self::transform(constraint_clause) {
            iterations += 1;
            debug_assert!(lit != 0 && lit != i32::MIN);
            self.solver.constrain(lit);
        }
        // finalize the clause if needed
        if iterations > 0 {
            self.solver.constrain(0);
        }

        // call the solve function
        match self.solver.solve() {
            Status::SATISFIABLE => SatResult::Sat,
            Status::UNSATISFIABLE => SatResult::UnSat,
            Status::UNKNOWN => unreachable!(),
        }
    }

    #[inline]
    pub fn val(&mut self, lit: Literal) -> Option<bool> {
        let lit = Self::transform_one(lit);
        let val = self.solver.val(lit);
        if val == lit {
            Some(true)
        } else if val == -lit {
            Some(false)
        } else {
            None
        }
    }

    #[inline]
    pub fn failed(&mut self, lit: Literal) -> bool {
        self.solver.failed(Self::transform_one(lit))
    }

    #[inline]
    pub fn constraint_failed(&mut self) -> bool {
        self.solver.constraint_failed()
    }

    #[inline]
    pub fn freeze(&mut self, v: Variable) {
        self.solver.freeze(v.number().try_into().unwrap());
    }

    #[inline]
    pub fn frozen(&mut self, v: Variable) -> bool {
        self.solver.frozen(v.number().try_into().unwrap())
    }

    #[inline]
    pub fn melt(&mut self, v: Variable) {
        self.solver.melt(v.number().try_into().unwrap())
    }

    #[inline]
    pub fn optimize(&mut self, val: i32) {
        self.solver.optimize(val)
    }

    #[inline]
    pub fn simplify(&mut self, rounds: i32) -> Option<SatResult> {
        match self.solver.simplify(rounds) {
            Status::SATISFIABLE => Some(SatResult::Sat),
            Status::UNSATISFIABLE => Some(SatResult::UnSat),
            Status::UNKNOWN => None,
        }
    }

    #[inline]
    pub fn get_clauses(&mut self) -> Vec<Clause> {
        struct CI {
            clause: Vec<Vec<i32>>,
        }

        impl cadical_sys::ClauseIterator for CI {
            fn clause(&mut self, clause: &[i32]) -> bool {
                self.clause.push(clause.to_vec());
                true
            }
        }

        let mut ci = CI { clause: Vec::new() };
        self.solver.traverse_clauses(&mut ci);

        ci.clause
            .into_iter()
            .map(|c| {
                let mut c: Vec<Literal> = c.into_iter().map(Literal::from_dimacs_number).collect();
                c.sort_unstable();
                // println!("c = {:?}", c);
                Clause::from_ordered_set(c)
            })
            .collect()
    }

    #[inline]
    pub fn reserve_variables(&mut self, max_var: Variable) {
        self.solver.reserve(max_var.number().try_into().unwrap())
    }

    // pub fn get_cnf(&mut self) -> CNF {
    //     // get tmp dir path
    //     let tmp_dir = env::temp_dir();

    //     // create tmp file path
    //     let random_string = format!("{:x}", rand::random::<u128>());
    //     let path = tmp_dir.join(format!("rust_formal_verification__tmp_{random_string}.cnf"));
    //     let path_string = path.to_str().unwrap();

    //     // write cnf to tmp file
    //     // println!("Writing CNF to file: {}", path_string);
    //     self.solver.write_dimacs(path.as_path()).unwrap();

    //     // read file to string
    //     let dimacs = fs::read_to_string(path_string)
    //         .unwrap_or_else(|_| panic!("Unable to read the '.cnf' file {path_string}"));

    //     // remove tmp file
    //     fs::remove_file(path_string).unwrap();

    //     // parse string to cnf
    //     CNF::from_dimacs(&dimacs).unwrap()
    // }

    // pub fn freeze(&mut self, lit: &Literal) {
    //     self.solver.freeze(Self::transform_one(lit));
    // }
}

// ************************************************************************************************
// impl trait
// ************************************************************************************************

impl IncrementalSatSolver for CaDiCalSolver {
    fn new(seed: u64) -> Self {
        Self::new(seed)
    }

    fn add_clause<I>(&mut self, clause: I)
    where
        I: IntoIterator<Item = Literal>,
    {
        self.add_clause(clause)
    }

    fn solve<I, U>(&mut self, assumptions: I, constraint_clause: U) -> SatResult
    where
        I: IntoIterator<Item = Literal>,
        U: IntoIterator<Item = Literal>,
    {
        self.solve(assumptions, constraint_clause)
    }

    fn val(&mut self, lit: Literal) -> Option<bool> {
        self.val(lit)
    }

    fn failed(&mut self, lit: Literal) -> bool {
        self.failed(lit)
    }

    fn constraint_failed(&mut self) -> bool {
        self.constraint_failed()
    }

    fn simplify(&mut self) -> Option<SatResult> {
        self.simplify(3)
    }
}

// ************************************************************************************************
// impl Default trait
// ************************************************************************************************

impl Default for CaDiCalSolver {
    fn default() -> Self {
        Self::new(1234567891011121314)
    }
}
