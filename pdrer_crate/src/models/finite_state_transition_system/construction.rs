// ************************************************************************************************
// use
// ************************************************************************************************

// use std::collections::FxHashSet;

use super::{FiniteStateTransitionSystem, FiniteStateTransitionSystemError};
use crate::formulas::{Clause, Cube, CNF};
use crate::formulas::{Literal, Variable};
use crate::models::circuit::node_types::{
    CircuitAnd, CircuitGenericGate, CircuitLatch, CircuitNode, CircuitNodeType,
};
use crate::models::circuit::Circuit;
use crate::models::truth_table::cnf::CNFCache;
use crate::models::{
    CircuitSimulator, Signal, SortedVecOfLiterals, TernaryValue, TruthTable, UniqueSortedHashMap,
    UniqueSortedVec, Wire,
};
use std::fmt;
// use std::process::Command;

// ************************************************************************************************
// impl
// ************************************************************************************************

// fn get_memory_usage() -> usize {
//     // This command will work on both Linux and macOS.
//     let output = Command::new("ps")
//         .args(["-o", "rss=", "-p", &std::process::id().to_string()])
//         .output()
//         .unwrap();

//     let memory_kb = String::from_utf8_lossy(&output.stdout)
//         .trim()
//         .parse::<usize>()
//         .unwrap();

//     memory_kb / 1024 // Convert from KB to MB
// }

impl FiniteStateTransitionSystem {
    // ********************************************************************************************
    // helper functions
    // ********************************************************************************************

    fn check_if_problem_is_trivial(
        circuit: &Circuit,
        assume_output_is_bad: bool,
    ) -> Result<(), FiniteStateTransitionSystemError> {
        // check if problem is trivial
        if circuit.get_highest_signal().wire(false).is_constant() {
            let constraint_wires = circuit.get_invariant_constraint_wires();
            if constraint_wires.iter().any(|w| w.is_constant_zero()) {
                return Err(FiniteStateTransitionSystemError::ConstraintWireIsConstantZero);
            }
            let bad_wires = Self::get_bad_wires(circuit, assume_output_is_bad);
            if bad_wires.iter().any(|w| w.is_constant_one()) {
                return Err(FiniteStateTransitionSystemError::BadWireIsConstantOne);
            }
            return Err(FiniteStateTransitionSystemError::EmptyCircuit);
        }
        Ok(())
    }

    // ********************************************************************************************
    // describer functions
    // ********************************************************************************************

    fn describe_and_gate_literal(&self, signal: Signal, inputs: &[Wire]) -> Vec<Clause> {
        // get variable numbers
        let lhs = self.convert_wire_to_literal(&signal.wire(false));
        debug_assert!(!(lhs.is_negated()));

        // check if this and gate is connected to a constant.
        if inputs.iter().any(|w| w.is_constant()) {
            // we can simplify the formula.
            if inputs.iter().any(|w| w == &Wire::new(0)) {
                // one of the wires is zero, so the output is zero.
                return vec![Clause::from_ordered_set(vec![!lhs])];
            } else if inputs.iter().all(|w| w == &Wire::new(1)) {
                // all of the wires are one, so the output is one.
                return vec![Clause::from_ordered_set(vec![lhs])];
            }
        }
        // no input is zero, and not all inputs are one.

        // are there any two inputs that are negations of each other
        if inputs.iter().any(|w1| inputs.iter().any(|w2| *w1 == !*w2)) {
            // the output is zero
            return vec![Clause::from_ordered_set(vec![!lhs])];
        }

        // debug_assert!(inputs.iter().all(|w| !w.is_constant()));
        let inputs_iter = inputs.iter().filter(|w| !w.is_constant());
        let mut clauses = Vec::with_capacity(inputs.len() + 1);

        // create clauses
        // lhs = rhs0 ^ rhs1 ^ rhs2 <=>
        // (lhs -> rhs0 ^ rhs1 ^ rhs2) ^ (lhs <- rhs0 ^ rhs1 ^ rhs2) <=>
        // (!lhs \/ (rhs0 ^ rhs1 ^ rhs2)) ^ (lhs \/ !(rhs0 ^ rhs1 ^ rhs2)) <=>
        // ((!lhs \/ rhs0) ^ (!lhs \/ rhs1)) ^ (!lhs \/ rhs2)) ^ (lhs \/ !rhs0 \/ !rhs1 \/ !rhs2)

        let mut clause = Vec::with_capacity(inputs.len() + 1);
        let mut last_variable: Option<Variable> = None;
        for (i, input) in inputs_iter.enumerate() {
            let rhs = self.convert_wire_to_literal(input);
            clauses.push(Clause::from_ordered_set(vec![rhs, !lhs]));
            clause.push(!rhs);

            // swap last 2 to remain sorted if needed
            let rhs_var = rhs.variable();
            match last_variable {
                None => {}
                Some(last_v) => {
                    if last_v == rhs_var {
                        clause.swap(i - 1, i);
                    }
                }
            }
            last_variable = Some(rhs_var);
        }
        clause.push(lhs);
        clauses.push(Clause::from_sequence(clause));
        clauses
    }

    fn describe_generic_gate(
        &self,
        signal: Signal,
        truth_table: &TruthTable,
        cache: &mut CNFCache,
    ) -> Vec<Clause> {
        truth_table.calculate_cnf_with_cache(|w| self.convert_wire_to_literal(w), signal, cache)
    }

    // ********************************************************************************************
    // handle ground
    // ********************************************************************************************

    fn handle_ground(&mut self, _: Signal) {
        // get variable
        // let variable = Self::convert_signal_to_variable(signal);
        // update hash map
        // self.variable_to_state_variables_in_its_cone
        //     .insert(variable, UniqueSortedVec::new());
    }

    // ********************************************************************************************
    // handle input
    // ********************************************************************************************

    fn handle_input(&mut self, signal: Signal) {
        let variable = self.convert_signal_to_variable(signal);
        self.input_variables.push(variable);
    }

    // ********************************************************************************************
    // handle latch
    // ********************************************************************************************

    fn handle_latch(
        &mut self,
        signal: Signal,
        latch: &CircuitLatch,
        initial_vector: &mut Vec<Literal>,
        transition_on_internals: &mut Vec<Clause>,
    ) {
        // get variable
        let variable = self.convert_signal_to_variable(signal);
        // update range
        // add variable to latch vars
        self.state_variables.push(variable);
        // update hash map for latches in cone
        // self.variable_to_state_variables_in_its_cone
        //     .insert(variable, UniqueSortedVec::from_ordered_set(vec![variable]));
        // update hash map for input
        let input_variable = self.convert_signal_to_variable(latch.input.signal());
        self.state_variable_to_its_internal_signal_variable
            .insert(variable, (input_variable, latch.input.is_negated()));
        // add to initial vector
        match latch.initial {
            TernaryValue::True => initial_vector.push(variable.literal(false)),
            TernaryValue::False => initial_vector.push(variable.literal(true)),
            TernaryValue::X => {}
        }
        // add to connection
        let mut latch_lit_after = variable.literal(false);
        self.add_tags_to_literal(&mut latch_lit_after, 1);

        if latch.input == Wire::new(0) {
            transition_on_internals.push(Clause::from_ordered_set(vec![!latch_lit_after]));
        } else if latch.input == Wire::new(1) {
            transition_on_internals.push(Clause::from_ordered_set(vec![latch_lit_after]));
        } else {
            let latch_input_lit = self.convert_wire_to_literal(&latch.input);
            let c1 = Clause::from_ordered_set(vec![latch_input_lit, !latch_lit_after]);
            let c2 = Clause::from_ordered_set(vec![!latch_input_lit, latch_lit_after]);
            if c1 < c2 {
                transition_on_internals.push(c1);
                transition_on_internals.push(c2);
            } else {
                transition_on_internals.push(c2);
                transition_on_internals.push(c1);
            }
        }
    }

    // ********************************************************************************************
    // handle gate
    // ********************************************************************************************

    fn handle_gate(&mut self, signal: Signal, node: &CircuitNode, cache: &mut CNFCache) {
        // get variable
        // let variable = self.convert_signal_to_variable(*signal);
        // update range

        // call appropriate handler
        match &node.node_type {
            CircuitNodeType::And(a) => self.handle_and_gate(signal, a),
            CircuitNodeType::GenericGate(g) => self.handle_generic_gate(signal, g, cache),
            _ => unreachable!(),
        }
    }

    fn handle_and_gate(&mut self, signal: Signal, a: &CircuitAnd) {
        // add to description
        let variable = self.convert_signal_to_variable(signal);
        let mut description = self.describe_and_gate_literal(signal, a.inputs.peek());
        description.sort_unstable();
        self.variable_definitions.insert(variable, description);
    }

    fn handle_generic_gate(
        &mut self,
        signal: Signal,
        g: &CircuitGenericGate,
        cache: &mut CNFCache,
    ) {
        // add to description
        let variable = self.convert_signal_to_variable(signal);
        let mut description = self.describe_generic_gate(signal, &g.truth_table, cache);
        description.sort_unstable();
        self.variable_definitions.insert(variable, description);
    }

    // ********************************************************************************************
    // single pass
    // ********************************************************************************************

    fn perform_single_pass_on_circuit(&mut self, circuit: &Circuit, _assume_output_is_bad: bool) {
        // a hash table that defines each gate variable
        let mut initial_vector: Vec<Literal> =
            Vec::with_capacity(circuit.get_latch_signals().len());
        let mut transition_on_internals: Vec<Clause> =
            Vec::with_capacity(circuit.get_latch_signals().len() * 2);
        let mut cnf_cache = CNFCache::default();

        // println!("Memory usage 4.2.1: {} MB", get_memory_usage());

        for signal in circuit.iter_sorted() {
            let node: &CircuitNode = circuit.get_node(&signal).unwrap();
            match &node.node_type {
                CircuitNodeType::ConstantZero => self.handle_ground(signal),
                CircuitNodeType::Input => self.handle_input(signal),
                CircuitNodeType::Latch(l) => {
                    self.handle_latch(signal, l, &mut initial_vector, &mut transition_on_internals)
                }
                CircuitNodeType::And(a) => self.handle_and_gate(signal, a),
                CircuitNodeType::GenericGate(_) => self.handle_gate(signal, node, &mut cnf_cache),
            }
        }

        // println!("Memory usage 4.2.3: {} MB", get_memory_usage());

        self.initial_states = Cube::from_ordered_set(initial_vector);
        self.transition_on_internals =
            CNF::from_ordered_set(UniqueSortedVec::from_ordered_set(transition_on_internals))
    }

    // ********************************************************************************************
    // post processing
    // ********************************************************************************************

    fn get_bad_wires(circuit: &Circuit, assume_output_is_bad: bool) -> UniqueSortedVec<Wire> {
        let mut important_wires = circuit.get_bad_wires().to_owned();
        if assume_output_is_bad {
            important_wires = important_wires.merge(circuit.get_output_wires());
        }
        important_wires
    }

    fn create_or_of_wires<F>(
        &self,
        wires: &UniqueSortedVec<Wire>,
        constant_handler: F,
        error_if_variables_not_unique: FiniteStateTransitionSystemError,
    ) -> Result<Clause, FiniteStateTransitionSystemError>
    where
        F: Fn(&Wire) -> Option<FiniteStateTransitionSystemError>,
    {
        // split to 2 cases, depending on if empty or not.
        if !wires.is_empty() {
            let mut unsafe_literals = Vec::new();
            for wire in wires.iter() {
                if wire.is_constant() {
                    let r = constant_handler(wire);
                    if let Some(e) = r {
                        return Err(e);
                    }
                } else {
                    let b_lit = self.convert_wire_to_literal(wire);
                    unsafe_literals.push(b_lit);
                }
            }
            if !SortedVecOfLiterals::are_variables_sorted_and_unique(&unsafe_literals) {
                return Err(error_if_variables_not_unique);
            }
            Ok(Clause::from_ordered_set(unsafe_literals))
        } else {
            // the empty clause is un-sat when turned into cnf
            let result = Clause::from_ordered_set(vec![]);
            Ok(result)
        }
    }

    fn get_description_of_signals(&self, signals: &[Signal]) -> CNF {
        let mut clauses = Vec::with_capacity(signals.len() * 6);

        let mut max_var_before_internals = self.state_variables.max().copied();
        if max_var_before_internals.is_none() {
            max_var_before_internals = self.input_variables.max().copied();
        }
        if max_var_before_internals.is_none() {
            max_var_before_internals = Some(Variable::new(0));
        }
        let max_var_before_internals = max_var_before_internals.unwrap();
        let max_signal_before_internals = self.convert_variable_to_signal(max_var_before_internals);

        for signal in signals {
            if *signal <= max_signal_before_internals {
                continue;
            }

            let variable = self.convert_signal_to_variable(*signal);
            // skip latches, inputs and ground

            let description_of_signal = self.variable_definitions.get(&variable).unwrap();
            clauses.append(&mut description_of_signal.to_vec())
        }
        CNF::from_ordered_set(UniqueSortedVec::from_ordered_set(clauses))
    }

    fn get_cone_of_transition(&self, circuit: &Circuit) -> Vec<Signal> {
        let signals_that_feed_into_latches = self
            .state_variable_to_its_internal_signal_variable
            .iter_items()
            .map(|x| &x.0)
            .map(|v| self.convert_variable_to_signal(*v));
        circuit
            .get_cone_of_influence(signals_that_feed_into_latches)
            .unpack()
    }

    fn get_state_variables_in_cone_of_variables(
        &self,
        variables: &[Variable],
        circuit: &Circuit,
    ) -> UniqueSortedVec<Variable> {
        let signals = variables
            .iter()
            .map(|v| self.convert_variable_to_signal(*v));

        let mut state_vars = Vec::with_capacity(circuit.get_latch_signals().len());
        let call_back = |s: Signal| {
            if let Some(x) = circuit.get_latch_signals().max() {
                if s <= *x && !s.is_constant() {
                    if let Some(y) = circuit.get_input_signals().max() {
                        if *y < s {
                            let var = self.convert_signal_to_variable(s);
                            state_vars.push(var);
                        }
                    } else {
                        let var = self.convert_signal_to_variable(s);
                        state_vars.push(var);
                    }
                }
            }
        };
        circuit.get_cone_of_influence_custom(signals, call_back);
        state_vars.shrink_to_fit();
        state_vars.reverse();
        UniqueSortedVec::from_ordered_set(state_vars)
    }

    fn post_processing(
        &mut self,
        circuit: &Circuit,
        assume_output_is_bad: bool,
    ) -> Result<(), FiniteStateTransitionSystemError> {
        // get bad and constraint wires
        let bad_wires = Self::get_bad_wires(circuit, assume_output_is_bad);
        let invariant_wires = circuit.get_invariant_constraint_wires();

        // wires that must not change when performing ternary simulation and dropping literals.
        // self.safety_and_constraints_simulation_signals = UniqueSortedVec::from_unsorted(
        //     bad_wires
        //         .iter()
        //         .chain(invariant_wires.iter())
        //         .map(|w| w.get_signal())
        //         .collect(),
        // );

        // make constraint on internal signals FIRST because of errors if both 1 and 2 happen

        self.invariant_constraints_on_internals = self
            .create_or_of_wires(
                invariant_wires,
                |w| {
                    if w.is_constant_zero() {
                        // 1
                        Some(FiniteStateTransitionSystemError::ConstraintWireIsConstantZero)
                    } else {
                        None
                    }
                },
                FiniteStateTransitionSystemError::ConstraintWiresIncludeWireAndItsNegation,
            )
            .map(|c| Cube::from_ordered_set(c.unpack().unpack().unpack()))?;

        // make property on internal signals
        self.property_on_internals = self
            .create_or_of_wires(
                &bad_wires,
                |w| {
                    if w.is_constant_one() {
                        // 2
                        Some(FiniteStateTransitionSystemError::BadWireIsConstantOne)
                    } else {
                        None
                    }
                },
                FiniteStateTransitionSystemError::BadWiresIncludeWireAndItsNegation,
            )
            .map(|c| !c)?;

        // make state to safety translation
        let cone_of_property = circuit
            .get_cone_of_influence(bad_wires.iter().map(|w| w.signal()))
            .unpack();
        self.property_connector = self.get_description_of_signals(&cone_of_property);

        // make state to invariant translation
        let mut cone_of_invariant = circuit
            .get_cone_of_influence(invariant_wires.iter().map(|w| w.signal()))
            .unpack();
        self.invariant_constraints_connector = self.get_description_of_signals(&cone_of_invariant);

        // make transition
        let mut cone_of_transition = self.get_cone_of_transition(circuit);
        self.transition_connector = self.get_description_of_signals(&cone_of_transition);

        self.state_variables_in_cone_of_safety = self.get_state_variables_in_cone_of_variables(
            &(self
                .property_on_internals
                .iter()
                .map(|l| l.variable())
                .collect::<Vec<_>>()),
            circuit,
        );

        self.state_variables_in_cone_of_invariant_constraint = self
            .get_state_variables_in_cone_of_variables(
                &(self
                    .invariant_constraints_on_internals
                    .iter()
                    .map(|l| l.variable())
                    .collect::<Vec<_>>()),
                circuit,
            );

        self.state_variables_in_cone_of_safety_and_invariant_constraint = UniqueSortedVec::merge(
            &self.state_variables_in_cone_of_safety,
            &self.state_variables_in_cone_of_invariant_constraint,
        );

        // signals to implicate on must be in the CNFs
        self.signals_to_implicate_on = {
            let mut a = cone_of_property.to_owned();
            a.append(&mut cone_of_invariant);
            a.append(&mut cone_of_transition);
            UniqueSortedVec::from_sequence(a)
        };

        for (state_var, (internal_var, _)) in self
            .state_variable_to_its_internal_signal_variable
            .iter_pairs()
        {
            let value = self.get_state_variables_in_cone_of_variables(&[*internal_var], circuit);
            self.state_variable_to_state_variables_in_its_cone
                .insert(state_var, value);
        }

        Ok(())
    }

    // ********************************************************************************************
    // circuit api functions
    // ********************************************************************************************

    pub fn new(
        circuit: &Circuit,
        assume_output_is_bad: bool,
    ) -> Result<Self, FiniteStateTransitionSystemError> {
        // perform some checks first
        let max_wire_in_circuit = circuit.get_highest_signal().wire(false);
        if max_wire_in_circuit >= Wire::new(u32::MAX) {
            return Err(FiniteStateTransitionSystemError::MaxWireTooHigh);
        }
        Self::check_if_problem_is_trivial(circuit, assume_output_is_bad)?;

        let signal_to_variable = |s: Signal| -> Variable { Variable::new(s.number()) };
        let variable_to_signal = |v: Variable| -> Signal { Signal::new(v.number()) };

        // println!("Memory usage 4.1: {} MB", get_memory_usage());

        let highest_latch_signal = circuit
            .get_latch_signals()
            .max()
            .copied()
            .unwrap_or(Signal::GROUND);
        let highest_latch_variable = signal_to_variable(highest_latch_signal);

        let max_variable = signal_to_variable(max_wire_in_circuit.signal());

        let mut result = Self {
            signal_to_variable: Box::new(signal_to_variable),
            variable_to_signal: Box::new(variable_to_signal),

            // initial_states:
            initial_states: Cube::from_sequence(vec![]),

            // connecter between state literals and desired internal signal literals
            transition_connector: CNF::new(),
            property_connector: CNF::new(),
            invariant_constraints_connector: CNF::new(),

            // desired properties
            invariant_constraints_on_internals: Cube::from_sequence(vec![]),
            property_on_internals: Cube::from_sequence(vec![]),
            transition_on_internals: CNF::new(),

            // some meta data
            max_variable,
            variable_definitions: UniqueSortedHashMap::new(max_variable),

            // all states in inputs
            input_variables: UniqueSortedVec::with_capacity(circuit.get_input_signals().len()),
            state_variables: UniqueSortedVec::with_capacity(circuit.get_latch_signals().len()),

            // for extraction
            state_variables_in_cone_of_safety: UniqueSortedVec::new(),
            state_variables_in_cone_of_invariant_constraint: UniqueSortedVec::new(),
            state_variables_in_cone_of_safety_and_invariant_constraint: UniqueSortedVec::new(),
            state_variable_to_its_internal_signal_variable: UniqueSortedHashMap::new(
                highest_latch_variable,
            ),
            state_variable_to_state_variables_in_its_cone: UniqueSortedHashMap::new(max_variable),

            // tri simulation
            circuit: CircuitSimulator::new(circuit),
            // safety_and_constraints_simulation_signals: UniqueSortedVec::new(),
            signals_to_implicate_on: UniqueSortedVec::new(),
        };

        // println!("Memory usage 4.2: {} MB", get_memory_usage());

        result.perform_single_pass_on_circuit(circuit, assume_output_is_bad);

        // println!("Memory usage 4.3: {} MB", get_memory_usage());

        // Consume circuit

        result.post_processing(circuit, assume_output_is_bad)?;

        // println!("Memory usage 4.4: {} MB", get_memory_usage());

        // // make vectors
        // result.make_variable_vectors(assume_output_is_bad);

        // // make formulas
        // result.make_formulas(assume_output_is_bad)?;

        // result.make_hash_sets();

        // result.make_cones();

        // result.make_simulation_variables();

        debug_assert!(result.check(circuit).is_ok());

        // create object
        Ok(result)
    }

    // pub fn new_from_aig(
    //     aig: &AndInverterGraph,
    // ) -> Result<(Self, SignalTracker), FiniteStateTransitionSystemError> {
    //     // let circuit = Circuit::from_aig(sig);
    //     let assume_output_is_bad = aig.get_bad_wires().is_empty();
    //     let mut circuit = Circuit::from_aig(aig);

    //     // These simplification make the resulting CNF smaller
    //     // circuit.simplify_circuit_before_using_proof_engine()
    //     let mut map = SignalTracker::new();
    //     map.push(circuit.remove_unused_signals());
    //     map.push(circuit.structural_hash());
    //     map.push(circuit.remove_unused_signals());
    //     map.push(circuit.detect_generic_patterns());

    //     let mut r = Self::new(&circuit, assume_output_is_bad)?;

    //     // These simplification were found to harm the CNF size but are still advantageous for the proof engine
    //     circuit.merge_and_gates();

    //     r.circuit = CircuitSimulator::new(&circuit);

    //     debug_assert!(r.check(&circuit).is_ok());

    //     Ok((r, map))
    // }
}

// ************************************************************************************************
// printing errors
// ************************************************************************************************

impl fmt::Display for FiniteStateTransitionSystemError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            FiniteStateTransitionSystemError::EmptyCircuit => {
                write!(f, "Circuit is empty or contains only ground.")
            }
            FiniteStateTransitionSystemError::MaxWireTooHigh => {
                write!(f, "Circuit has wires numbers that are too high.")
            }
            FiniteStateTransitionSystemError::ConstraintWireIsConstantZero => {
                write!(f, "Constraint wire is constant zero, and thus the invariant constraints are never satisfied, meaning that the model is safe or un-sat")
            }
            FiniteStateTransitionSystemError::ConstraintWiresIncludeWireAndItsNegation => {
                write!(f, "Constraint wires include x and also !x, and thus the invariant constraints are never satisfied, meaning that the model is safe or un-sat")
            }
            FiniteStateTransitionSystemError::BadWireIsConstantOne => {
                write!(
                    f,
                    "Bad wire is constant one, and thus the property is always violated (probably unsafe or sat), this model could still be safe if there are constraints that are never satisfied."
                )
            }
            FiniteStateTransitionSystemError::BadWiresIncludeWireAndItsNegation => {
                write!(
                    f,
                    "Bad wires include x and also !x, and thus the property is always violated (probably unsafe or sat), this model could still be safe if there are constraints that are never satisfied."
                )
            }
        }
    }
}
