// ************************************************************************************************
// use
// ************************************************************************************************

use std::iter;

use crate::{
    formulas::{Clause, Cube, Variable, CNF},
    models::{Circuit, Counterexample, UniqueSortedVec, Utils},
    solvers::sat::incremental::{IncrementalSatSolver, IncrementalSolverUtils, SatResult},
};

use super::{Definition, FiniteStateTransitionSystem, Proof, ProofResult};

// ************************************************************************************************
// impl
// ************************************************************************************************

impl FiniteStateTransitionSystem {
    // ********************************************************************************************
    // helper functions
    // ********************************************************************************************

    // ********************************************************************************************
    // API
    // ********************************************************************************************

    pub fn check(&self, _circuit: &Circuit) -> Result<(), String> {
        // sorted

        // ensure initial_latch_cube,
        Ok(())
    }

    // ********************************************************************************************
    // check CNF
    // ********************************************************************************************

    pub fn definitions_to_cnf(definitions: &[Definition]) -> CNF {
        let mut cnf = vec![];
        for d in definitions {
            cnf.extend(d.to_cnf());
        }
        CNF::from_ordered_set(UniqueSortedVec::from_ordered_set(cnf))
    }

    /// Returns true if all initial states satisfy the cnf
    pub fn do_all_initial_states_satisfy_cnf<T: IncrementalSatSolver>(
        &self,
        definitions: &[Definition],
        cnf: &CNF,
    ) -> bool {
        let mut init = self.construct_initial_cnf(true);
        init.append(Self::definitions_to_cnf(definitions));
        Utils::does_a_imply_b::<T>(&init, cnf).unwrap()
    }

    pub fn is_cnf_semi_inductive<T: IncrementalSatSolver>(&self, cnf: &CNF) -> bool {
        self.is_cnf_semi_inductive_with_definitions::<T>(&[], cnf)
    }

    pub fn is_cnf_semi_inductive_with_definitions<T: IncrementalSatSolver>(
        &self,
        definitions: &[Definition],
        cnf: &CNF,
    ) -> bool {
        let transition_and_inv = {
            let mut tmp: CNF = self.construct_transition_cnf(true, true, true, true);
            tmp.append(cnf.to_owned());
            let mut d_cnf = Self::definitions_to_cnf(definitions);
            tmp.append(d_cnf.to_owned());
            self.add_tags_to_relation(&mut d_cnf, 1);
            tmp.append(d_cnf);
            tmp
        };

        let cnf_tag = {
            let mut inv_clone = cnf.to_owned();
            self.add_tags_to_relation(&mut inv_clone, 1);
            inv_clone
        };

        Utils::does_a_imply_b::<T>(&transition_and_inv, &cnf_tag).unwrap_or(true)
    }

    pub fn does_cnf_guarantee_safety<T: IncrementalSatSolver>(
        &self,
        definitions: &[Definition],
        cnf: &CNF,
    ) -> bool {
        let mut bad = self.construct_property_cnf(true, true);
        bad.append(Self::definitions_to_cnf(definitions));
        !Utils::is_a_and_b_satisfiable::<T>(cnf, &bad)
    }

    /// Returns true if all initial states violate constraints
    pub fn do_all_initial_states_violate_constraints<T: IncrementalSatSolver>(&self) -> bool {
        let init = self.construct_initial_cnf(true);
        let mut solver = IncrementalSolverUtils::new_solver::<T>(&init, 0);

        // check if there are no initial states that satisfy the safety property
        SatResult::UnSat == solver.solve(iter::empty(), iter::empty())
    }

    // /// check if the transition relation with constraints is satisfiable
    // pub fn do_all_successors_violate_constraints<T: IncrementalSatSolver>(&self) -> bool {
    //     let transition = self.construct_transition_cnf(true, true, true);
    //     let mut solver = IncrementalSolverUtils::new_solver::<T>(&transition, 0);

    //     // check if there are no initial states that satisfy the safety property
    //     IncrementalSatResponse::UnSat == solver.solve(iter::empty(), iter::empty())
    // }

    pub fn is_there_an_initial_state_that_satisfies_all_constraints_and_violates_the_property<
        T: IncrementalSatSolver,
    >(
        &self,
    ) -> Option<Counterexample> {
        let init = self.construct_initial_cnf(true);
        let mut solver = IncrementalSolverUtils::new_solver::<T>(&init, 0);
        let p = self.property_on_internals.to_owned();
        let not_p: Clause = !p;
        solver.add_clause(not_p.iter().copied());

        if SatResult::Sat == solver.solve(iter::empty(), iter::empty()) {
            let input = self.extract_input_from_solver(|l| solver.val(l));
            let state = self.extract_state_from_solver(|l| solver.val(l));
            return Some(Counterexample {
                initial_cube: state,
                inputs: vec![input],
            });
        }

        None
    }

    /// Checks if the problem that self represent is trivial.
    /// A problem is considered trivial if one of the following holds:
    /// 1. There exists an initial state that satisfies all the constraints, and violates the safety property.
    /// 2. All initial states violate one of the invariant constraints.
    pub fn is_trivial<T: IncrementalSatSolver>(&self) -> Option<ProofResult> {
        if self.do_all_initial_states_violate_constraints::<T>() {
            return Some(Ok(Proof {
                all_initial_states_violate_constraints: true,
                invariant: CNF::from_sequence(vec![]),
                definitions: vec![],
            }));
        }

        if let Some(ctx) = self.is_there_an_initial_state_that_satisfies_all_constraints_and_violates_the_property::<T>() {
            return Some(Err(ctx));
        }

        // if self.do_all_successors_violate_constraints::<T>() {
        //     return Some(Ok(Proof {
        //         invariant: CNF::new(vec![]),
        //     }));
        // }

        None
    }

    /// A definition `new_variable = f(x, y, z)` if for any assignment to `x, y, z` the CNF of the definition is satisfied.
    /// Meaning that there does not exist an assigmnet to `x, y, z` that (regardless of the value of `new_variable`) the CNF is not satisfied.
    /// This function checks that all the definitions are valid.
    ///
    /// If a proof contains a definition that is not valid, the proof can sneak in assumptions to the model and so
    /// cheking the proof must gurantee that the defitions are valid.
    pub fn are_definitions_valid<T: IncrementalSatSolver>(
        &self,
        mut definitions: Vec<Definition>,
    ) -> bool {
        definitions.sort_by_key(|d| d.variable);
        let mut min_var = self.max_variable;
        self.add_tags_to_variable(&mut min_var, 1);
        let mut vars = vec![];
        let mut var_tags = vec![];
        for d in definitions {
            if !d.is_valid() {
                return false;
            }
            let is_too_small = d.variable <= min_var;
            let is_already_used = vars.contains(&d.variable);
            let is_already_used_by_tag = var_tags.contains(&d.variable);
            if is_too_small || is_already_used || is_already_used_by_tag {
                return false;
            }
            vars.push(d.variable);
            let mut v = d.variable;
            self.add_tags_to_variable(&mut v, 1);
            var_tags.push(v);
        }
        true
    }

    /// checks if invariant proves property.
    pub fn check_proof<T: IncrementalSatSolver>(&self, proof: &Proof) -> Result<(), String> {
        if self.do_all_initial_states_violate_constraints::<T>() {
            return Ok(());
        }

        // inv_candidate.append(proof.definitions.to_owned());

        Utils::ensure(self.are_definitions_valid::<T>(proof.definitions.to_owned()), "Some defition is not valid. Meaning that there exists an assignment to the inputs of the definition that does not satisfy the definition. In other words the defentnion is not an expression that defines the new variable but it also adds some constrain on the input.")?;

        // check INIT -> inv_candidate
        Utils::ensure(
            self.do_all_initial_states_satisfy_cnf::<T>(&proof.definitions, &proof.invariant),
            "Invariant does not cover all of init.",
        )?;

        Utils::ensure(
            self.is_cnf_semi_inductive_with_definitions::<T>(&proof.definitions, &proof.invariant),
            "Invariant doesn't cover all of the reachable states (invariant is not inductive)",
        )?;

        Utils::ensure(
            self.does_cnf_guarantee_safety::<T>(&proof.definitions, &proof.invariant),
            "Invariant isn't always safe. (invariant does not guarantee the safety property)",
        )?;

        Ok(())
    }

    // ********************************************************************************************
    // API - Check counter example
    // ********************************************************************************************

    pub fn get_implications_of_state_and_input(&mut self, state: Cube, input: Cube) -> Cube {
        let mut lits = input.unpack().unpack().unpack();
        lits.append(&mut state.unpack().unpack().unpack());
        self.get_state_implications(&lits, false)
    }

    pub fn do_implications_violate_the_property(&self, implications: &Cube) -> bool {
        self.property_on_internals
            .iter()
            .any(|l| implications.contains(&!*l))
    }

    pub fn may_implications_violate_some_invariant_constraint(&self, implications: &Cube) -> bool {
        let intersection = self
            .invariant_constraints_on_internals
            .peek()
            .peek()
            .intersect(implications.peek().peek());
        intersection.len() < self.invariant_constraints_on_internals.len()
    }

    pub fn get_next_state_from_implications(&self, implications: &Cube) -> Cube {
        let mut next_state = Vec::new();
        for (latch_var, (latch_input_var, is_negated)) in self
            .state_variable_to_its_internal_signal_variable
            .iter_pairs()
        {
            // if latch connected to ground
            if latch_input_var == &Variable::new(0) {
                next_state.push(latch_var.literal(!is_negated));
                continue;
            }

            // figure out what the input is
            let input_is_1 = implications
                .peek()
                .contains(&latch_input_var.literal(false));
            let input_is_0 = implications.peek().contains(&latch_input_var.literal(true));
            debug_assert!(!(input_is_0 && input_is_1));

            // push to next state
            if input_is_1 {
                next_state.push(latch_var.literal(*is_negated));
            } else if input_is_0 {
                next_state.push(latch_var.literal(!is_negated));
            }
            // if implications.contains(x)
        }
        Cube::from_sequence(next_state)
    }

    pub fn check_counter_example(
        &mut self,
        ctx: Counterexample,
        verbose: bool,
    ) -> Result<(), String> {
        const REQUIRE_COMPLETE_ASSIGNMENTS: bool = false;
        // check that the provided cube is actually a state
        for l in ctx.initial_cube.iter() {
            Utils::ensure(
                self.is_state_literal(l),
                "Counter example starts with a cube that contains non-state literals.",
            )?;
        }
        if REQUIRE_COMPLETE_ASSIGNMENTS {
            // check that initial state assigns all latches
            for state_var in self.state_variables.iter() {
                Utils::ensure(
                    ctx.initial_cube.peek().contains(&state_var.literal(false))
                        || ctx.initial_cube.peek().contains(&state_var.literal(true)),
                    "Counter example starts with a cube that does not assign all latches.",
                )?;
            }
        }
        // check that the initial cube is initial
        Utils::ensure(
            self.is_cube_satisfied_by_some_initial_state(&ctx.initial_cube)
                .unwrap(),
            "Counter example starts with a cube that is not initial.",
        )?;
        let mut current_state = ctx.initial_cube;
        let last_iter_index = ctx.inputs.len() - 1;
        for (i, input) in ctx.inputs.into_iter().enumerate() {
            if verbose {
                println!(
                    "Iteration: {} state = {}\t input = {}",
                    i,
                    Clause::from_ordered_set(current_state.peek().peek().peek().to_owned()),
                    Clause::from_ordered_set(input.peek().peek().peek().to_owned())
                );
            }
            // check that the provided cube is actually a state
            for l in input.iter() {
                Utils::ensure(
                    self.is_input_literal(l),
                    "Counter example contains input cube that contains non-input literals.",
                )?;
            }
            if REQUIRE_COMPLETE_ASSIGNMENTS {
                // check that the input cube assigns all input
                for input_var in self.input_variables.iter() {
                    Utils::ensure(
                        input.peek().contains(&input_var.literal(false))
                            || input.peek().contains(&input_var.literal(true)),
                        "Counter example contains input cube that does not assign all inputs.",
                    )?;
                }
            }
            // check that the current state is a state
            for l in current_state.iter() {
                debug_assert!(self.is_state_literal(l));
            }

            // perform simulation
            let implications = self.get_implications_of_state_and_input(current_state, input);

            if verbose {
                println!(
                    "Iteration: {} are constraints violated = {}\t is property violated = {}",
                    i,
                    self.may_implications_violate_some_invariant_constraint(&implications),
                    self.do_implications_violate_the_property(&implications)
                );
            }

            // all states must not violate the invariant constraint
            Utils::ensure(
                !self.may_implications_violate_some_invariant_constraint(&implications),
                "Some state in the counter example violates some invariant constraint.",
            )?;

            // in the last iteration we need to check that the implications lead to the violation of the safety property
            if last_iter_index == i {
                Utils::ensure(
                    self.do_implications_violate_the_property(&implications),
                    "Last state in trace is not unsafe.",
                )?;
                break;
            }

            // calculate expression for next state
            current_state = self.get_next_state_from_implications(&implications);
        }
        Ok(())
        // self.circuit.begin_simulation(initial_signals)
    }

    /// Check if the proof result is correct.
    pub fn check_proof_result<T: IncrementalSatSolver>(
        &mut self,
        proof_result: ProofResult,
    ) -> Result<(), String> {
        match proof_result {
            ProofResult::Ok(p) => self.check_proof::<T>(&p),
            ProofResult::Err(ctx) => self.check_counter_example(ctx, false),
        }
    }
}
