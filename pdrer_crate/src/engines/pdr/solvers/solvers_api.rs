use std::{cell::RefCell, rc::Rc};

use fxhash::FxHashSet;

use crate::{
    engines::pdr::shared_objects::SharedObjects,
    formulas::{Clause, Cube, Literal, Variable, CNF},
    models::{Definition, FiniteStateTransitionSystem},
    solvers::sat::incremental::SatResult,
};

use super::{PropertyDirectedReachabilitySolver, SolverHolder, Solvers};

// ************************************************************************************************
// Transition extraction function
// ************************************************************************************************

pub fn build_transition(
    fin_state: &FiniteStateTransitionSystem,
    assume_constraints_in_second_cycle: bool,
    simplify: bool,
) -> CNF {
    let mut transition = fin_state.get_transition_connector().to_owned();
    transition.append(fin_state.get_transition_on_internals().to_owned());
    transition.append(fin_state.get_invariant_constraints_connector().to_owned());
    transition.append(
        fin_state
            .get_invariant_constraints_on_internals()
            .to_owned()
            .to_cnf(),
    );
    transition.append(fin_state.get_property_connector().to_owned());

    if assume_constraints_in_second_cycle {
        transition.append({
            let mut a = fin_state.get_invariant_constraints_connector().to_owned();
            fin_state.add_tags_to_relation(&mut a, 1);
            a
        });
    }

    if simplify {
        let t = transition.unpack().unpack();

        let mut frozen = Vec::new();
        let a = fin_state.get_input_variables().iter().copied();
        let b = fin_state.get_state_variables().iter().copied();
        let c = fin_state
            .get_property_on_internals()
            .iter()
            .map(|l| l.variable());
        let d = fin_state
            .get_invariant_constraints_on_internals()
            .iter()
            .map(|l| l.variable());
        let e = fin_state
            .get_state_variables()
            .iter()
            .copied()
            .map(|mut v| {
                fin_state.add_tags_to_variable(&mut v, 1);
                v
            });
        let ec = Cube::new_true();
        let f = if assume_constraints_in_second_cycle {
            fin_state.get_invariant_constraints_on_internals()
        } else {
            &ec
        }
        .iter()
        .map(|l| l.variable())
        .map(|mut v| {
            fin_state.add_tags_to_variable(&mut v, 1);
            v
        });
        for v in a.chain(b).chain(c).chain(d).chain(e).chain(f) {
            frozen.push(v);
        }

        let (x, _) = CNF::simplify_using_cadical(23456, &t, &frozen, 5);
        transition = CNF::from_sequence(x);
    }

    transition
}

// ************************************************************************************************
// impl
// ************************************************************************************************

impl<T: PropertyDirectedReachabilitySolver> Solvers<T> {
    // ********************************************************************************************
    // helper functions
    // ********************************************************************************************

    /// Insert a clause into the solver as is or reversed
    fn insert_clause_into_solver<F: Fn(Variable) -> Variable>(
        solver: &mut T,
        var_map: F,
        clause: &Clause,
        activations: Vec<Literal>,
        rev: bool,
    ) {
        if rev {
            solver.add_clause(
                activations.into_iter().chain(
                    clause
                        .iter()
                        .rev()
                        .copied()
                        .map(|l| var_map(l.variable()).literal(l.is_negated())),
                ),
            );
        } else {
            solver.add_clause(
                activations.into_iter().chain(
                    clause
                        .iter()
                        .copied()
                        .map(|l| var_map(l.variable()).literal(l.is_negated())),
                ),
            );
        }
    }

    fn insert_extension_transition_clauses_into_solver<F: Fn(Variable) -> Variable>(
        solver: &mut T,
        var_map: F,
        clauses: &[Clause],
        s: &SharedObjects,
    ) {
        for mut c in clauses.iter().cloned() {
            Self::insert_clause_into_solver(
                solver,
                &var_map,
                &c,
                vec![],
                s.parameters.insert_extension_variable_definitions_reversed,
            );
            s.fin_state.borrow().add_tags_to_clause(&mut c, 1);
            Self::insert_clause_into_solver(
                solver,
                &var_map,
                &c,
                vec![],
                s.parameters.insert_extension_variable_definitions_reversed,
            );
        }
    }

    fn check_var_map(var_map: &[Variable], activations: &[Variable]) -> bool {
        assert!(activations.iter().all(|v| v.number() != 0));
        assert!(activations.windows(2).all(|w| w[0] < w[1]));

        let mut x = FxHashSet::with_capacity_and_hasher(var_map.len(), Default::default());
        for v in var_map.iter().filter(|v| v.number() != 0) {
            if x.contains(v) {
                // no two variables can map to the same variable in the SAT solver
                return false;
            }
            if activations.contains(v) {
                // nothing can map to an activation variable in the SAT solver
                return false;
            }
            x.insert(*v);
        }

        true
    }

    fn update_var_map(
        var_map: &mut Vec<Variable>,
        activations: &[Variable],
        from: Variable,
        to: Variable,
    ) {
        let index = from.number() as usize;
        if index >= var_map.len() {
            var_map.resize(index + 1, Variable::new(0));
        }
        var_map[index] = to;

        debug_assert!(Self::check_var_map(var_map, activations));
    }

    fn recursively_update_var_map(var_map: &mut Vec<Variable>, from: Variable, to: Variable) {
        let index = from.number() as usize;
        if index >= var_map.len() {
            var_map.resize(index + 1, Variable::new(0));
        }

        for v in var_map.iter_mut().filter(|v| v.number() != 0) {
            if to <= *v {
                v.bump(1);
            }
        }
        var_map[index] = to;
    }

    fn m(var_map: &[Variable], v: Variable) -> Variable {
        let r = var_map[v.number() as usize];
        debug_assert!(r.number() != 0);
        r
    }

    fn ml(var_map: &[Variable], l: Literal) -> Literal {
        Self::m(var_map, l.variable()).literal(l.is_negated())
    }

    fn get_new_solver<'a, I>(
        s: &SharedObjects,
        transition: &CNF,
        initial: bool,
        ext: &[Clause],
        f_inf: I,
        var_map: &[Variable],
    ) -> T
    where
        I: IntoIterator<Item = &'a Clause>,
    {
        let mut solver = T::new(s.parameters.seed);

        // insert transition clauses
        for c in transition.iter() {
            Self::insert_clause_into_solver(
                &mut solver,
                |v| Self::m(var_map, v),
                c,
                vec![],
                s.parameters.insert_transition_clauses_reversed,
            );
        }

        Self::insert_extension_transition_clauses_into_solver(
            &mut solver,
            |v| Self::m(var_map, v),
            ext,
            s,
        );

        // insert initial state clauses
        if initial {
            for l in s.fin_state.borrow().get_initial_relation().iter() {
                Self::insert_clause_into_solver(
                    &mut solver,
                    |v| Self::m(var_map, v),
                    &Clause::from_ordered_set(vec![*l]),
                    vec![],
                    s.parameters.insert_transition_clauses_reversed,
                );
            }
        } else {
            for c in f_inf {
                Self::insert_clause_into_solver(
                    &mut solver,
                    |v| Self::m(var_map, v),
                    c,
                    vec![],
                    s.parameters.insert_frame_clauses_reversed,
                );
            }
        }

        solver
    }

    // ********************************************************************************************
    // helper functions when using single solver
    // ********************************************************************************************

    fn find_possible_activation_literal(
        var_map: &mut [Variable],
        activations: &[Variable],
    ) -> (bool, Variable) {
        debug_assert!(activations.iter().all(|v| v.number() != 0));
        debug_assert!(activations.windows(2).all(|w| w[0] < w[1]));
        let mut max = activations.last().copied().unwrap_or(Variable::new(0));
        max.bump(1);

        // does variable already exist?
        if var_map.contains(&max) {
            for v in var_map.iter_mut().filter(|v| v.number() != 0) {
                v.bump(2);
            }
            debug_assert!(!var_map.contains(&max));
            (true, max)
        } else {
            (false, max)
        }
    }

    fn get_activations_for_frame(frame: usize, activations: &[Variable]) -> Vec<Literal> {
        debug_assert!(frame <= activations.len());
        if frame == activations.len() {
            Vec::new()
        } else {
            vec![activations[frame].literal(false)]
        }
    }

    fn get_assumptions_from_frame(frame: usize, activations: &[Variable]) -> Vec<Literal> {
        debug_assert!(frame <= activations.len());
        let mut all_off: Vec<Literal> = activations.iter().map(|v| v.literal(false)).collect();
        if frame == 0 {
            all_off[0] = !all_off[0];
        } else {
            for i in all_off.iter_mut().skip(frame) {
                *i = !*i;
            }
        }
        all_off
    }

    // ********************************************************************************************
    // API
    // ********************************************************************************************

    fn get_initial_mapping(s: &SharedObjects, transition: &CNF) -> Vec<Variable> {
        let mut vars = transition.get_variables();
        vars = vars.merge(s.fin_state.borrow().get_input_variables());
        let mut state_vars = s.fin_state.borrow().get_state_variables().to_owned();
        vars = vars.merge(&state_vars);

        state_vars
            .perform_operation_on_each_value(|v| s.fin_state.borrow().add_tags_to_variable(v, 1));
        vars = vars.merge(&state_vars);

        let mut var_map = vec![Variable::new(0); vars.len()];
        for v in vars.iter().copied() {
            Self::update_var_map(&mut var_map, &[], v, v);
        }

        var_map
    }

    /// Initialize a solver instance
    pub fn new(s: SharedObjects) -> Self {
        let transition = build_transition(
            &s.fin_state.borrow(),
            s.parameters.assume_constraints_in_second_cycle,
            true,
        );

        let var_map = Self::get_initial_mapping(&s, &transition);

        let mut r = if s.parameters.use_only_one_solver {
            let mut var_map = var_map;
            let (_, act) = Self::find_possible_activation_literal(&mut var_map, &[]);
            let activations = vec![act];
            // debug_assert!(needs_reset);
            Self {
                s,
                transition,
                var_map,
                must_reset: false,
                ext: Vec::new(),
                h: SolverHolder::Shared(Rc::new(RefCell::new(T::new(0))), activations.to_owned()),
            }
        } else {
            Self {
                s,
                transition,
                var_map,
                must_reset: false,
                ext: Vec::new(),
                h: SolverHolder::Owned(vec![T::new(0), T::new(0)]),
            }
        };

        r.rest_solvers(vec![vec![], vec![]]);
        r
    }

    pub fn new_solver(&mut self, deltas: Vec<Vec<Clause>>) {
        debug_assert!(!self.must_reset);
        match &mut self.h {
            SolverHolder::Shared(_, activations) => {
                let (needs_reset, act) =
                    Self::find_possible_activation_literal(&mut self.var_map, activations);
                activations.push(act);
                debug_assert!(Self::check_var_map(&self.var_map, activations));
                if needs_reset {
                    self.rest_solvers(deltas);
                }
            }
            SolverHolder::Owned(s) => {
                let f = Self::get_new_solver(
                    &self.s,
                    &self.transition,
                    false,
                    &self.ext,
                    deltas.last().unwrap(),
                    &self.var_map,
                );
                s.insert(s.len() - 1, f);
            }
        }
    }

    // /// reserve variables in the solver
    // fn reserve_variables(&mut self, max_var: Variable);

    /// add clause to solver
    pub fn add_frame_clause(&mut self, frame: usize, clause: &Clause) {
        debug_assert!(!self.must_reset);
        match &mut self.h {
            SolverHolder::Shared(solver, act) => {
                let c = clause.to_owned();
                Self::insert_clause_into_solver(
                    &mut solver.borrow_mut(),
                    |v| Self::m(&self.var_map, v),
                    &c,
                    Self::get_activations_for_frame(frame, act),
                    self.s.parameters.insert_frame_clauses_reversed,
                );
            }
            SolverHolder::Owned(s) => Self::insert_clause_into_solver(
                &mut s[frame],
                |v| Self::m(&self.var_map, v),
                clause,
                vec![],
                self.s.parameters.insert_frame_clauses_reversed,
            ),
        }
    }

    pub fn add_new_definition(&mut self, def: &Definition) {
        debug_assert!(match &self.h {
            SolverHolder::Shared(_, act) => {
                Self::check_var_map(&self.var_map, act)
            }
            SolverHolder::Owned(_) => Self::check_var_map(&self.var_map, &[]),
        });

        let cnf = def.to_cnf();
        self.ext.extend(cnf.iter().cloned());
        let mut from = def.variable;

        debug_assert!(!def.inputs.is_empty());
        let mut to = def
            .inputs
            .iter()
            .map(|l| Self::m(&self.var_map, l.variable()))
            .max()
            .unwrap();
        to.bump(1);

        Self::recursively_update_var_map(&mut self.var_map, from, to);

        debug_assert!(match &self.h {
            SolverHolder::Shared(_, act) => {
                Self::check_var_map(&self.var_map, act)
            }
            SolverHolder::Owned(_) => Self::check_var_map(&self.var_map, &[]),
        });

        self.s.fin_state.borrow().add_tags_to_variable(&mut from, 1);

        let mut to = def
            .inputs
            .iter()
            .copied()
            .map(|mut l| {
                self.s.fin_state.borrow().add_tags_to_literal(&mut l, 1);
                l
            })
            .map(|l| Self::m(&self.var_map, l.variable()))
            .max()
            .unwrap();
        to.bump(1);

        Self::recursively_update_var_map(&mut self.var_map, from, to);

        debug_assert!(match &self.h {
            SolverHolder::Shared(_, act) => {
                Self::check_var_map(&self.var_map, act)
            }
            SolverHolder::Owned(_) => Self::check_var_map(&self.var_map, &[]),
        });

        self.must_reset = true;
    }

    pub fn rest_solvers(&mut self, mut deltas: Vec<Vec<Clause>>) {
        // debug_assert!(self.must_reset);
        match &mut self.h {
            SolverHolder::Shared(solver, activations) => {
                debug_assert!(activations.len() == deltas.len() - 1);
                debug_assert!(Self::check_var_map(&self.var_map, activations));

                // first add the initial state clauses to F0
                debug_assert!(deltas[0].is_empty());
                deltas[0].extend(
                    self.s
                        .fin_state
                        .borrow()
                        .get_initial_relation()
                        .to_cnf()
                        .unpack()
                        .unpack(),
                );

                // Create the solver where F_inf has no activation literals
                let new_solver = Self::get_new_solver(
                    &self.s,
                    &self.transition,
                    false,
                    &self.ext,
                    &[],
                    &self.var_map,
                );

                // Reset the solver
                *solver.borrow_mut() = new_solver;

                self.must_reset = false;

                // Insert all frames one clause at a time
                for (i, d) in deltas.into_iter().enumerate() {
                    for c in d {
                        self.add_frame_clause(i, &c);
                    }
                }
            }
            SolverHolder::Owned(vec) => {
                debug_assert!(Self::check_var_map(&self.var_map, &[]));
                for (i, solver) in vec.iter_mut().enumerate() {
                    *solver = Self::get_new_solver(
                        &self.s,
                        &self.transition,
                        i == 0,
                        &self.ext,
                        deltas[i..].iter().flat_map(|v| v.iter()),
                        &self.var_map,
                    );
                }
                self.must_reset = false;
            }
        }
        debug_assert!(!self.must_reset);
    }

    pub fn max_variable_in_transition(&self) -> Variable {
        self.transition.get_max_variable()
    }

    /// solver under the assumptions and constraint clause X ^ ass1 ^ ass2 ^ ... ^ (con1 \/ con2 \/ con3 \/...)
    pub fn solve<I, U>(&mut self, frame: usize, assumptions: I, constraint_clause: U) -> SatResult
    where
        I: IntoIterator<Item = Literal>,
        U: IntoIterator<Item = Literal>,
    {
        debug_assert!(!self.must_reset);
        let assumptions = assumptions.into_iter().map(|l| Self::ml(&self.var_map, l));
        let constraint_clause = constraint_clause
            .into_iter()
            .map(|l| Self::ml(&self.var_map, l));
        match &mut self.h {
            SolverHolder::Shared(solver, activations) => {
                let ass = Self::get_assumptions_from_frame(frame, activations);
                // println!("Ass = {:?}", ass);
                solver
                    .borrow_mut()
                    .solve(ass.into_iter().chain(assumptions), constraint_clause)
            }
            SolverHolder::Owned(s) => s[frame].solve(assumptions, constraint_clause),
        }
    }

    /// valid in the sat case, retrieves a variable's truth table
    /// The returned value is `None` if the formula is satisfied
    /// regardless of the value of the literal.
    #[inline]
    pub(super) fn val(
        h: &mut SolverHolder<T>,
        var_map: &[Variable],
        frame: usize,
        lit: Literal,
    ) -> Option<bool> {
        match h {
            SolverHolder::Shared(s, _) => s.borrow_mut().val(Self::ml(var_map, lit)),
            SolverHolder::Owned(s) => s[frame].val(Self::ml(var_map, lit)),
        }
    }

    /// valid in the un-sat case, checks for a failed assumption.
    /// Returns true if the literal was assumed in the last sat call and
    /// was important for concluding that the call was un sat.
    pub(super) fn failed(
        h: &mut SolverHolder<T>,
        var_map: &[Variable],
        frame: usize,
        lit: Literal,
    ) -> bool {
        match h {
            SolverHolder::Shared(s, _) => s.borrow_mut().failed(Self::ml(var_map, lit)),
            SolverHolder::Owned(s) => s[frame].failed(Self::ml(var_map, lit)),
        }
    }
}
