//! Performs structural hashing
//!
//! // ************************************************************************************************
// use
// ************************************************************************************************

use fxhash::FxHashMap;

use crate::models::{
    circuit::{
        node_types::{CircuitAnd, CircuitLatch, CircuitNodeType},
        CircuitSimplifier,
    },
    signal_tracker::SignalTransformation,
    Circuit, Signal, TernaryValue, UniqueSortedHashMap, UniqueSortedVec, Wire,
};

// ************************************************************************************************
// simplifier
// ************************************************************************************************

pub struct CircuitStructuralHashing {}

// ************************************************************************************************
// impl
// ************************************************************************************************

impl CircuitStructuralHashing {
    // ********************************************************************************************
    // helper functions
    // ********************************************************************************************

    fn get_hash_of_wire(w: &Wire, node_to_hash: &UniqueSortedHashMap<Signal, Wire>) -> Wire {
        let rep = node_to_hash.get(&w.signal()).unwrap();
        if w.is_negated() {
            !*rep
        } else {
            *rep
        }
    }

    fn translate_wires(
        wires: &UniqueSortedVec<Wire>,
        node_to_hash: &UniqueSortedHashMap<Signal, Wire>,
    ) -> UniqueSortedVec<Wire> {
        let mut result = Vec::with_capacity(wires.len());
        for w in wires.iter() {
            result.push(Self::get_hash_of_wire(w, node_to_hash));
        }
        UniqueSortedVec::from_sequence(result)
    }

    /// This function should return the hash of the and gate.
    fn hash_and_gate(
        signal: Signal,
        a: &CircuitAnd,
        node_to_hash: &UniqueSortedHashMap<Signal, Wire>,
        and_inputs_to_node: &mut FxHashMap<UniqueSortedVec<Wire>, Signal>,
    ) -> Wire {
        // first translate the wire inputs to what they will be after translation
        let mut inputs = Self::translate_wires(&a.inputs, node_to_hash).unpack();

        debug_assert!(!inputs.is_empty());

        // remove ones
        inputs.retain(|w| !w.is_constant_one());

        // if all inputs were ones, return one
        if inputs.is_empty() {
            return Wire::CONSTANT_ONE;
        }

        // check if something is zero
        if inputs.iter().any(|w| w.is_constant_zero()) {
            return Wire::CONSTANT_ZERO;
        }

        // check for two negating inputs (this relies on x, !x being next to each other)
        if inputs.windows(2).any(|w| w[0] == !w[1]) {
            return Wire::CONSTANT_ZERO;
        }

        // now hash the and gate such that it and gates that should be equal will hash to the same value.
        if inputs.len() == 1 {
            let input = inputs[0];
            return Self::get_hash_of_wire(&input, node_to_hash);
        }

        // check if there was an and gate with the same inputs
        let inputs = UniqueSortedVec::from_ordered_set(inputs);
        if and_inputs_to_node.contains_key(&inputs) {
            let node = and_inputs_to_node.get(&inputs).unwrap();
            return *node_to_hash.get(node).unwrap();
        }

        // there is no and gate with the same inputs
        and_inputs_to_node.insert(inputs, signal);
        signal.wire(false)
    }

    fn hash_latch(
        signal: Signal,
        a: &CircuitLatch,
        node_to_hash: &UniqueSortedHashMap<Signal, Wire>,
        latch_details_to_node: &mut FxHashMap<CircuitLatch, Signal>,
    ) -> Wire {
        // uninitialized latches must always hash to their own wires
        if a.initial == TernaryValue::X {
            return signal.wire(false);
        }

        // check if there is a latch with the same input and initial value
        if latch_details_to_node.contains_key(a) {
            let latch_signal = latch_details_to_node.get(a).unwrap();
            return *node_to_hash.get(latch_signal).unwrap();
        }

        // there is no and gate with the same inputs
        latch_details_to_node.insert(a.to_owned(), signal);
        signal.wire(false)
    }

    /// Two node are equivalent iff they always output the same value.
    /// This function detects some cases that guarantee this:
    /// 1. The two nodes accept the same inputs and have the same truth table.
    /// 2. One is an identity function of another.
    /// 3. And gate with negated inputs is equivalent to a Ground.
    /// 4.
    fn get_signal_to_representative(circuit: &Circuit) -> UniqueSortedHashMap<Signal, Wire> {
        // node hash takes a hash key and returns the signal that represents it.
        // let mut node_to_hash: FxHashMap<Signal, Wire> = FxHashMap::<HashKey, Signal>::new();
        // let mut past_hashes: FxHashMap<Signal, HashKey> = FxHashMap::<Signal, HashKey>::new();
        let mut and_inputs_to_node: FxHashMap<UniqueSortedVec<Wire>, Signal> = Default::default();
        let mut latch_details_to_node: FxHashMap<CircuitLatch, Signal> = Default::default();
        let mut node_to_hash: UniqueSortedHashMap<Signal, Wire> =
            UniqueSortedHashMap::new_like(&circuit.nodes);

        // go over all nodes and call pattern matcher.
        for (signal, node) in circuit.nodes.iter_pairs() {
            // let node = self.nodes.get(&signal).unwrap();

            match &node.node_type {
                CircuitNodeType::ConstantZero => {
                    node_to_hash.insert(signal, Wire::CONSTANT_ZERO);
                }
                CircuitNodeType::Input => {
                    node_to_hash.insert(signal, signal.wire(false));
                }
                CircuitNodeType::Latch(a) => {
                    let h = Self::hash_latch(signal, a, &node_to_hash, &mut latch_details_to_node);
                    node_to_hash.insert(signal, h);
                }
                CircuitNodeType::And(a) => {
                    let h = Self::hash_and_gate(signal, a, &node_to_hash, &mut and_inputs_to_node);
                    node_to_hash.insert(signal, h);
                }
                CircuitNodeType::GenericGate(_) => {
                    unreachable!("Structural hashing with generic gates is not implemented")
                }
            }
        }

        node_to_hash
    }

    fn rewrite_circuit_according_to_mapping(
        circuit: &mut Circuit,
        equivalence: &mut UniqueSortedHashMap<Signal, Wire>,
        mapping: &UniqueSortedHashMap<Signal, Wire>,
    ) {
        let signals = circuit.nodes.iter_sorted().collect::<Vec<_>>();
        for signal in signals {
            // delete node if it is not the representative of itself
            if mapping.get(&signal).unwrap() != &signal.wire(false) {
                circuit.nodes.remove(&signal);
                debug_assert!(circuit.gates.contains(&signal) || circuit.latches.contains(&signal));
                circuit.gates.remove(&signal);
                circuit.latches.remove(&signal);
                equivalence.insert(signal, *mapping.get(&signal).unwrap());
                continue;
            }

            // this node should remain and should its inputs should be updated
            let node = circuit.nodes.get_mut(&signal).unwrap();
            match &mut node.node_type {
                CircuitNodeType::ConstantZero => {}
                CircuitNodeType::Input => {}
                CircuitNodeType::Latch(a) => {
                    a.input = Self::get_hash_of_wire(&a.input, mapping);
                }
                // these nodes have inputs
                CircuitNodeType::And(a) => {
                    let mut inputs = Self::translate_wires(&a.inputs, mapping).unpack();
                    inputs.retain(|w| !w.is_constant_one());
                    debug_assert!(!inputs.is_empty());
                    a.inputs = UniqueSortedVec::from_ordered_set(inputs);
                }
                CircuitNodeType::GenericGate(_) => {
                    // let inputs = Self::translate_wires(&a.inputs, mapping);
                    // a.inputs = inputs;
                    todo!()
                }
            }
        }

        // if ground is needed, add it
        circuit.add_ground_if_possible();

        // fix outputs, bad and constraints
        circuit.outputs = Self::translate_wires(&circuit.outputs, mapping);
        circuit.bad = Self::translate_wires(&circuit.bad, mapping);
        circuit.constraints = Self::translate_wires(&circuit.constraints, mapping);

        // update greatest signal
        circuit.greatest_signal = circuit.nodes.max_key().unwrap();

        // fix important signals
        circuit.important_signals = circuit.recalculate_important_signals();

        // fix levels and users
        // circuit.fix_levels_and_users();

        circuit.remove_ground_if_possible();
    }

    // ********************************************************************************************
    // API
    // ********************************************************************************************

    pub fn new() -> Self {
        Self {}
    }

    pub fn structural_hash(circuit: &mut Circuit) -> SignalTransformation {
        // get signals in order
        // let signals: Vec<Signal> = .collect();
        // let mut signal_mapping = SignalMapping::new(self.get_highest_signal());
        // let mut signal_mapping = SignalTracker::new(self.greatest_signal);

        let mut equivalences = UniqueSortedHashMap::new_like(&circuit.nodes);
        loop {
            let before: Vec<Signal> = circuit.nodes.iter_sorted().collect();
            let mapping = Self::get_signal_to_representative(circuit);
            Self::rewrite_circuit_according_to_mapping(circuit, &mut equivalences, &mapping);
            debug_assert!(circuit.check().is_ok());
            let after: Vec<Signal> = circuit.nodes.iter_sorted().collect();
            if before == after {
                break;
            }
        }

        SignalTransformation::SignalsRemovedBecauseOfEquivalentWires(equivalences)
    }
}

impl CircuitSimplifier for CircuitStructuralHashing {
    fn simplify(&mut self, circuit: &mut Circuit) -> SignalTransformation {
        CircuitStructuralHashing::structural_hash(circuit)
    }

    fn title(&self) -> String {
        "Structural hashing".to_string()
    }
}
