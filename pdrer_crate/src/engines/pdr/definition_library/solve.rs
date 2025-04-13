// ************************************************************************************************
// use
// ************************************************************************************************

use std::iter;

use super::DefinitionLibrary;
use crate::{
    engines::pdr::{delta_element::DeltaElement, PropertyDirectedReachabilitySolver},
    formulas::{Clause, Cube, Literal},
    function,
    models::{time_stats::function_timer::FunctionTimer, TernaryValue},
    solvers::{
        dd::{DDError, DecisionDiagramManager},
        sat::incremental::SatResult,
    },
};

// ************************************************************************************************
// impl
// ************************************************************************************************

impl<T: PropertyDirectedReachabilitySolver, D: DecisionDiagramManager> DefinitionLibrary<T, D> {
    // ********************************************************************************************
    // SAT
    // ********************************************************************************************

    fn sat_solve_is_clause_a_tautology(&mut self, clause: &Clause) -> bool {
        let _timer = FunctionTimer::start(function!(), self.s.time_stats.clone());

        let r = self
            .solver
            .solve((!clause.to_owned()).iter().copied(), iter::empty());
        self.s.pdr_stats.borrow_mut().note_ev_sat_call(&r);
        r == SatResult::UnSat
    }

    fn sat_solve_is_clause_a_contradiction(&mut self, clause: &Clause) -> bool {
        let _timer = FunctionTimer::start(function!(), self.s.time_stats.clone());

        let r = self
            .solver
            .solve((!clause.to_owned()).iter().copied(), iter::empty());
        self.s.pdr_stats.borrow_mut().note_ev_sat_call(&r);
        r == SatResult::UnSat
    }

    pub fn sat_solve_implies(&mut self, a: &Clause, b: &Clause) -> bool {
        let _timer = FunctionTimer::start(function!(), self.s.time_stats.clone());
        debug_assert!(!a.is_empty());
        let b = !b.to_owned();
        let r = self.solver.solve(b.iter().copied(), a.iter().copied());
        self.s.pdr_stats.borrow_mut().note_ev_sat_call(&r);
        r == SatResult::UnSat
    }

    // ********************************************************************************************
    // BDD
    // ********************************************************************************************

    /// Converts a clause to a BDD, not all variables in the clause will be in the cone of
    /// influence of some extension variable, thus, these variables do not exist in the BDD.
    ///
    /// Example input `(a \/ b \/ c)`, x = a ^ b
    /// Example Output `(a \/ b)` as a BDD
    fn clause_to_bdd(&mut self, clause: &Clause) -> Result<D::DecisionDiagram, DDError> {
        let _timer = FunctionTimer::start(function!(), self.s.time_stats.clone());

        let mut bdd = self.manager_ref.bot()?;
        for l in clause.iter() {
            let v = l.variable();
            if !self.var_to_bdd.contains_key(&v) {
                return Err(DDError::OutOfMemory);
            }

            let x = self.var_to_bdd.get(&v).unwrap();
            if l.is_negated() {
                let not_x = self.manager_ref.apply_not(x)?;
                bdd = self.manager_ref.apply_or(&bdd, &not_x)?;
            } else {
                bdd = self.manager_ref.apply_or(&bdd, x)?;
            }
        }

        Ok(bdd)
    }

    fn cube_to_bdd(&mut self, cube: &Cube) -> Result<D::DecisionDiagram, DDError> {
        let _timer = FunctionTimer::start(function!(), self.s.time_stats.clone());

        let mut bdd = self.manager_ref.top()?;
        for l in cube.iter() {
            let v = l.variable();
            if !self.var_to_bdd.contains_key(&v) {
                return Err(DDError::OutOfMemory);
            }

            let x = self.var_to_bdd.get(&v).unwrap();
            if l.is_negated() {
                let not_x = self.manager_ref.apply_not(x)?;
                bdd = self.manager_ref.apply_and(&bdd, &not_x)?;
            } else {
                bdd = self.manager_ref.apply_and(&bdd, x)?;
            }
        }

        Ok(bdd)
    }

    pub fn ternary_propagation_using_bdds(&mut self, def: usize, cube: &Cube) -> TernaryValue {
        let v = self.definitions[def].variable;
        let cube_bdd = self.cube_to_bdd(cube).unwrap();
        let var_bdd = self.var_to_bdd.get(&v).unwrap();
        let tri_bdd = self.manager_ref.apply_and(var_bdd, &cube_bdd).unwrap();
        if self.manager_ref.is_contradiction(&tri_bdd).unwrap() {
            TernaryValue::False
        } else if self.manager_ref.is_tautology(&tri_bdd).unwrap() {
            TernaryValue::True
        } else {
            TernaryValue::X
        }
    }

    // represent a clause in BDDs, use caching to save result.
    pub fn clause_to_bdd_cached(&mut self, clause: &Clause) -> Result<D::DecisionDiagram, DDError> {
        let _timer = FunctionTimer::start(function!(), self.s.time_stats.clone());

        if self.clause_cache.get(clause).is_none() {
            let bdd = self.clause_to_bdd(clause)?;
            self.clause_cache.insert(clause.to_owned(), bdd);
        }

        Ok(self.clause_cache.get(clause).unwrap().to_owned())
    }

    /// A query literal is a literal that is in the cone of influence of some extension variable.
    /// This function checks if a literal is a query literal.
    /// This distinction is important because the BDD is only defined for query literals.
    fn is_in_coi_of_some_ev(&self, l: &Literal) -> bool {
        let _timer = FunctionTimer::start(function!(), self.s.time_stats.clone());

        self.var_to_bdd.contains_key(&l.variable())
    }

    fn bdd_solve_is_clause_a_tautology(&mut self, clause: &Clause) -> Result<bool, DDError> {
        let _timer = FunctionTimer::start(function!(), self.s.time_stats.clone());

        if clause.iter().any(|l| !self.is_in_coi_of_some_ev(l)) {
            return Ok(false);
        }

        let r = self.clause_to_bdd_cached(clause)?;
        let r = self.manager_ref.is_tautology(&r)?;
        Ok(r)
    }

    fn bdd_solve_is_clause_a_contradiction(&mut self, clause: &Clause) -> Result<bool, DDError> {
        let _timer = FunctionTimer::start(function!(), self.s.time_stats.clone());

        if clause.iter().any(|l| !self.is_in_coi_of_some_ev(l)) {
            return Ok(false);
        }

        let r = self.clause_to_bdd_cached(clause)?;
        let r = self.manager_ref.is_tautology(&r)?;
        Ok(r)
    }

    /// Checks if a -> b
    fn bdd_solve_implies(
        &mut self,
        a: &DeltaElement<D>,
        b: &DeltaElement<D>,
    ) -> Result<bool, DDError> {
        let _timer = FunctionTimer::start(function!(), self.s.time_stats.clone());

        let a = a.dd().as_ref().ok_or(DDError::OutOfMemory)?;
        let b = b.dd().as_ref().ok_or(DDError::OutOfMemory)?;
        let r = self.manager_ref.apply_imp(a, b)?;

        self.manager_ref.is_tautology(&r)
    }

    // ********************************************************************************************
    // API
    // ********************************************************************************************

    pub fn solve_is_clause_a_tautology(&mut self, clause: &Clause) -> bool {
        let x = self.bdd_solve_is_clause_a_tautology(clause);
        if let Ok(x) = x {
            debug_assert_eq!(self.sat_solve_is_clause_a_tautology(clause), x);
            return x;
        }

        self.sat_solve_is_clause_a_tautology(clause)
    }

    pub fn solve_is_clause_a_contradiction(&mut self, clause: &Clause) -> bool {
        let x = self.bdd_solve_is_clause_a_contradiction(clause);
        if let Ok(x) = x {
            debug_assert_eq!(self.sat_solve_is_clause_a_contradiction(clause), x);
            return x;
        }

        self.sat_solve_is_clause_a_contradiction(clause)
    }

    pub fn solve_implies(&mut self, a: &DeltaElement<D>, b: &DeltaElement<D>) -> bool {
        let ab = (a.clause().to_owned(), b.clause().to_owned());
        if let Some(x) = self.implies_cache.get(&ab) {
            self.s
                .pdr_stats
                .borrow_mut()
                .increment_generic_count("solve_implies cache hit");
            return x.to_owned();
        }

        let x = self.bdd_solve_implies(a, b);
        if let Ok(x) = x {
            debug_assert!(
                if self.sat_solve_implies(a.clause(), b.clause()) == x {
                    true
                } else {
                    println!("\na = {}\nb = {}\n{}", a.clause(), b.clause(), self);
                    let r = self.bdd_solve_implies(a, b);
                    println!("r = {:?}", r);
                    false
                },
                "\na = {}\nb = {}\n{}",
                a.clause(),
                b.clause(),
                self
            );
            self.s
                .pdr_stats
                .borrow_mut()
                .increment_generic_count("solve_implies solved with DDs.");
            self.implies_cache.insert(ab, x);
            return x;
        }

        let r = self.sat_solve_implies(a.clause(), b.clause());
        self.s
            .pdr_stats
            .borrow_mut()
            .increment_generic_count("solve_implies solved with SAT call.");
        self.implies_cache.insert(ab, r);
        r
    }

    pub fn solve_is_clause_valid(&mut self, clause: &Clause) -> bool {
        let _timer = FunctionTimer::start(function!(), self.s.time_stats.clone());

        if self.solve_is_clause_a_contradiction(clause) {
            return false;
        }
        if self.solve_is_clause_a_tautology(clause) {
            return false;
        }

        true
    }
}
