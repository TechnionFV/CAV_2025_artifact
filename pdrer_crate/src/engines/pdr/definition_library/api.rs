// ************************************************************************************************
// use
// ************************************************************************************************

use std::fmt::Formatter;

use super::{Definition, DefinitionLibrary};
use crate::{
    engines::pdr::{shared_objects::SharedObjects, PropertyDirectedReachabilitySolver},
    formulas::{Clause, Variable},
    function,
    models::{
        definition::DefinitionFunction, time_stats::function_timer::FunctionTimer, PrettyTable,
        SortedVecOfLiterals, UniqueSortedVec,
    },
    solvers::dd::DecisionDiagramManager,
};

use fxhash::FxHashMap;
use quick_cache::unsync::Cache;

// ************************************************************************************************
// impl
// ************************************************************************************************

const CACHE_SIZE: usize = 1_000_000;

// ************************************************************************************************
// impl
// ************************************************************************************************

impl<T: PropertyDirectedReachabilitySolver, D: DecisionDiagramManager> DefinitionLibrary<T, D> {
    // ********************************************************************************************
    // helper functions
    // ********************************************************************************************

    pub fn position(
        &self,
        function: DefinitionFunction,
        inputs: &SortedVecOfLiterals,
    ) -> Option<(usize, bool)> {
        for (i, d) in self.definitions.iter().enumerate() {
            if d.function != function {
                continue;
            }
            match d.function {
                DefinitionFunction::And => {
                    if &d.inputs == inputs {
                        return Some((i, false));
                    }
                }
                DefinitionFunction::Xor => {
                    if &d.inputs == inputs {
                        return Some((i, false));
                    }
                    if d.inputs.len() == inputs.len()
                        && d.inputs
                            .iter()
                            .zip(inputs.iter())
                            .all(|(a, b)| a.variable() == b.variable())
                    {
                        if d.inputs
                            .iter()
                            .zip(inputs.iter())
                            .filter(|(a, b)| **a == !**b)
                            .count()
                            % 2
                            == 0
                        {
                            return Some((i, false));
                        } else {
                            return Some((i, true));
                        }
                    }
                }
            }
        }
        None
    }

    fn is_function_still_undefined(
        &self,
        function: DefinitionFunction,
        inputs: &SortedVecOfLiterals,
    ) -> bool {
        self.position(function, inputs).is_none()
    }

    fn save_definition_in_sat_solver(&mut self, d: &Definition) {
        if true {
            for c in d.to_cnf() {
                self.solver.add_clause(c.iter().copied());
            }
        }
    }

    /// Try and save a definition of a new variable using BDDs
    fn save_definition_in_bdd(&mut self, d: &Definition) -> Option<()> {
        let _timer = FunctionTimer::start(function!(), self.s.time_stats.clone());

        debug_assert!(!self.var_to_bdd.contains_key(&d.variable));
        debug_assert!(!d.inputs.is_empty());

        let mut d_bdd = match &d.function {
            DefinitionFunction::And => self.manager_ref.top(),
            DefinitionFunction::Xor => self.manager_ref.bot(),
        }
        .ok()?;

        for x in d.inputs.iter() {
            let a = self.var_to_bdd.get(&x.variable())?;
            let a = if x.is_negated() {
                self.manager_ref.apply_not(a).ok()?
            } else {
                a.to_owned()
            };

            d_bdd = match d.function {
                DefinitionFunction::And => self.manager_ref.apply_and(&d_bdd, &a).ok()?,
                DefinitionFunction::Xor => self.manager_ref.apply_xor(&d_bdd, &a).ok()?,
            };
        }

        self.var_to_bdd.insert(d.variable, d_bdd);
        Some(())
    }

    fn save_definition(&mut self, d: Definition) -> usize {
        debug_assert!(self.is_function_still_undefined(d.function, &d.inputs));
        self.definitions.push(d.to_owned());
        debug_assert!(!self.is_function_still_undefined(d.function, &d.inputs));
        self.save_definition_in_sat_solver(&d);
        self.save_definition_in_bdd(&d);

        debug_assert!(!self.extended_var_to_state_coi.contains_key(&d.variable));
        debug_assert!(!self.extended_var_to_coi.contains_key(&d.variable));

        let mut cone = self.build_coi(d.inputs.iter().map(|l| l.variable()));
        cone.insert(d.variable);
        self.extended_var_to_coi.insert(d.variable, cone.clone());
        cone.retain(|v| !self.is_extension_variable(*v));
        self.extended_var_to_state_coi.insert(d.variable, cone);

        self.definitions.len() - 1
    }

    fn check_that_variable_has_not_been_used_before(&self, v: Variable) -> bool {
        let mut v_tag = v;
        self.s
            .fin_state
            .borrow()
            .add_tags_to_variable(&mut v_tag, 1);

        let check_used_variable = |used_variable: Variable| {
            // let used_variable = ;
            let mut used_variable_tag = used_variable;
            self.s
                .fin_state
                .borrow()
                .add_tags_to_variable(&mut used_variable_tag, 1);
            debug_assert!(used_variable < v);
            debug_assert!(used_variable_tag < v_tag);
            debug_assert_ne!(used_variable, v_tag);
            debug_assert_ne!(used_variable_tag, v);
        };

        for d in self.definitions.iter() {
            check_used_variable(d.variable);
        }

        for v in self.s.fin_state.borrow().get_input_variables().iter() {
            check_used_variable(*v);
        }

        for v in self.s.fin_state.borrow().get_state_variables().iter() {
            check_used_variable(*v);
        }

        // let used_var = self.s.fin_state.borrow().get_max_variable();
        // check_used_variable(used_var);

        true
    }

    // ********************************************************************************************
    // API
    // ********************************************************************************************

    pub fn new(s: SharedObjects, max_variable_in_transition: Variable) -> Self {
        // let f = ;
        let state_vars = s.fin_state.borrow().get_state_variables().to_owned();
        // Avoid allocating too much memory for small cases
        let max_memory_in_mb = if state_vars.len() <= 10 {
            1
        } else if state_vars.len() <= 10000 {
            state_vars.len() / 10
        } else {
            1000
        };
        let mut manager_ref = D::new(state_vars.len(), 1, max_memory_in_mb);
        let mut var_to_bdd: FxHashMap<Variable, D::DecisionDiagram> = FxHashMap::default();

        for i in 0..state_vars.len() {
            let dd = manager_ref.ithvar(i).unwrap();
            let v = state_vars.peek()[i];
            var_to_bdd.insert(v, dd);
        }

        let mut base = max_variable_in_transition.number() as usize + 1;

        if let Some(mut min_base) = s.fin_state.borrow().get_state_variables().max().copied() {
            s.fin_state.borrow().add_tags_to_variable(&mut min_base, 1);
            base = std::cmp::max(base, min_base.number() as usize + 1);
        }

        Self {
            definitions: vec![],
            var_to_bdd,
            extended_var_to_state_coi: Default::default(),
            extended_var_to_coi: Default::default(),
            base,
            free_variable: 0,
            implies_cache: Cache::new(CACHE_SIZE),
            solver: T::new(12345678910111213141),
            manager_ref,
            clause_cache: Cache::new(CACHE_SIZE),
            s,
        }
    }

    pub fn len(&self) -> usize {
        self.definitions.len()
    }

    pub fn is_empty(&self) -> bool {
        self.definitions.is_empty()
    }

    pub fn iter(&self) -> impl DoubleEndedIterator<Item = &Definition> + ExactSizeIterator {
        self.definitions.iter()
    }

    pub fn at(&self, i: usize) -> &Definition {
        &self.definitions[i]
    }

    pub fn get_definitions(&self) -> &Vec<Definition> {
        &self.definitions
    }

    /// Takes a clause and adds to it all the definitions that are applicable.
    ///
    ///
    /// 1. `x = (!a /\ !b)`
    /// 2. `y = (x /\ !c)`
    ///
    /// the clause `(a \/ b \/ c)` will become `(a \/ b \/ c \/ !x)` which will become `(a \/ b \/ c \/ !x \/ !y)`
    pub fn ternary_propagation(&self, clause: Clause) -> Option<Clause> {
        let mut cube = !clause;
        for d in self.definitions.iter() {
            if let Some(l) = d.get_ternary_result_of_definition_on_cube(&cube) {
                if cube.contains(&!l) {
                    return None;
                }
                cube.insert(l);
            }
        }
        Some(!cube)
    }

    /// Takes a clause and converts it to a canonical form, meaning that if a definition can be added, it will be added.
    /// For example, if:
    /// 1. `x = (!a /\ !b)`
    /// 2. `y = (x /\ !c)`
    ///
    /// the clause `(a \/ b \/ c)` will become `(a \/ b \/ c \/ !x)` which will become `(a \/ b \/ c \/ !x \/ !y)`
    pub fn make_clause_canonical(&self, mut clause: Clause) -> Clause {
        clause = self.backwards(clause);
        // println!("Backwards: {}", clause);
        // clause = self.forward(clause).unwrap();
        // println!("Forward: {}", clause);
        clause
    }

    /// Takes a clause and converts it to the most definition using form,
    /// meaning that if a set of clauses is already define, it will be replaced.
    /// For example, if:
    /// 1. `x = (!a /\ !b)`
    /// 2. `y = (x /\ !c)`
    ///
    /// the clause `(a \/ b \/ c)` will become `(a \/ b \/ c \/ !x)` which will become `(a \/ b \/ c \/ !x \/ !y)` which will become `(!y)`
    /// Function may return None if during forward it was found that the clause is a tautology.
    pub fn forward(&self, mut clause: Clause) -> Option<Clause> {
        let mut to_remove = Vec::new();
        for d in self.definitions.iter() {
            clause = d.forward(clause, &mut to_remove)?;
        }

        let mut clause = clause.unpack().unpack();
        to_remove.sort_unstable();
        to_remove.dedup();
        clause = clause.subtract(&UniqueSortedVec::from_ordered_set(to_remove));
        Some(Clause::from_ordered_set(clause.unpack()))
    }

    pub fn backwards(&self, mut clause: Clause) -> Clause {
        for d in self.iter().rev() {
            clause = d.backwards(clause);
        }
        clause
    }

    // pub fn does_cube_contain_redundancy(&self, cube: &Cube) -> bool {
    //     for d in self.definitions.iter() {
    //         if cube.contains_variable(&d.variable) {
    //             if let Some(l) = d.get_ternary_result_of_definition_on_cube(cube) {
    //                 assert!(cube.contains(&l));
    //                 return true;
    //             }
    //         }
    //     }
    //     false
    // }

    // pub fn apply_definition_to_clauses(d: &Definition, clauses: &mut [Clause]) {
    //     for c in clauses.iter_mut() {
    //         // let mut new_literals = vec![];
    //         if d.function == DefinitionFunction::And && d.inputs.iter().all(|l| c.contains(&!*l)) {
    //             println!("Clause before applying definition: {}", c);
    //             for l in d.inputs.iter() {
    //                 // if c.contains(&!*l) {
    //                 let r = c.remove(&!*l);
    //                 debug_assert!(r);
    //                 // }
    //             }
    //             c.insert(d.variable.literal(true));
    //             println!("Clause before applying definition: {}", c);
    //             // *c = Clause::from_ordered_set(new_literals);
    //         }
    //     }
    // }

    /// Definition of a new variable,
    pub fn get_free_variable(&mut self) -> Variable {
        let x = self.free_variable;
        self.free_variable += 1;
        let v = self.index_to_var_number(x);
        let r = Variable::new(v as u32);
        debug_assert!(self.check_that_variable_has_not_been_used_before(r));
        r
    }

    pub fn index_to_var_number(&self, index: usize) -> usize {
        let n = self.s.fin_state.borrow().get_max_variable().number() as usize;
        // let base = self.base.get() as usize + 1;
        let pos = index % n;
        let i = index / n;
        self.base + (2 * n * i) + pos
    }

    pub fn is_extension_variable(&self, v: Variable) -> bool {
        let n = self.s.fin_state.borrow().get_max_variable();
        v > n
    }

    pub fn add_definition(
        &mut self,
        function: DefinitionFunction,
        inputs: SortedVecOfLiterals,
    ) -> Result<(usize, bool), bool> {
        let _timer = FunctionTimer::start(function!(), self.s.time_stats.clone());

        match self.position(function, &inputs) {
            Some((i, should_negate)) => Ok((i, should_negate)),
            None => {
                let v = self.get_free_variable();
                // self.s.weights.borrow_mut().introduce_new_variable(v);
                let d = Definition {
                    variable: v,
                    function,
                    inputs,
                };
                {
                    let l = Clause::from_ordered_set(vec![d.variable.literal(false)]);
                    debug_assert!(!self.solve_is_clause_a_contradiction(&l));
                    // if self.solve_is_clause_a_contradiction(&l) {
                    //     unreachable!();
                    //     self.free_variable -= 1;
                    //     return Err(false);
                    // }
                    debug_assert!(!self.solve_is_clause_a_tautology(&l));
                    // if self.solve_is_clause_a_tautology(&l) {
                    //     unreachable!();
                    //     self.free_variable -= 1;
                    //     return Err(true);
                    // }
                }
                let x = self.save_definition(d);
                Ok((x, false))
            }
        }
    }

    // pub fn get_state_coi_of_variable(&self, v: Variable) -> &UniqueSortedVec<Variable> {
    //     if let Some(x) = self.extended_var_to_state_coi.get(&v) {
    //         return x;
    //     }
    //     unreachable!()
    // }

    pub fn get_coi_of_variable(&self, v: Variable) -> &UniqueSortedVec<Variable> {
        if let Some(x) = self.extended_var_to_coi.get(&v) {
            return x;
        }
        unreachable!()
    }

    pub fn build_coi<I>(&self, v: I) -> UniqueSortedVec<Variable>
    where
        I: IntoIterator<Item = Variable>,
    {
        let mut coi = UniqueSortedVec::new();
        for v in v {
            match self.extended_var_to_coi.get(&v) {
                Some(cone) => {
                    debug_assert!(cone.contains(&v));
                    coi.extend(cone.iter().copied());
                }
                None => {
                    coi.insert(v);
                }
            }
            coi.insert(v);
        }
        coi
    }

    // pub fn get_definition(
    //     &self,
    //     l1: Literal,
    //     l2: Literal,
    //     f: DefinitionFunction,
    // ) -> Option<Definition> {
    //     assert!(l1 < l2);
    //     let i = self.position(l1, l2, f)?;
    //     Some(self.definitions[i])
    // }
}

// ************************************************************************************************
// fmt
// ************************************************************************************************

impl<T: PropertyDirectedReachabilitySolver, D: DecisionDiagramManager> std::fmt::Display
    for DefinitionLibrary<T, D>
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut table = PrettyTable::new(vec![
            "#".to_string(),
            "Variable".to_string(),
            "Function".to_string(),
            "Inputs".to_string(),
        ]);

        for (i, d) in self.definitions.iter().enumerate() {
            table
                .add_row(vec![
                    i.to_string(),
                    d.variable.to_string(),
                    d.function.to_string(),
                    d.inputs.peek().to_string(),
                ])
                .unwrap();
        }
        write!(f, "{}", table)
    }
}

impl<T: PropertyDirectedReachabilitySolver, D: DecisionDiagramManager> std::fmt::Debug
    for DefinitionLibrary<T, D>
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

// ********************************************************************************************
// tests
// ********************************************************************************************
