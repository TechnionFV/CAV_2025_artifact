// ************************************************************************************************
// use
// ************************************************************************************************

use crate::models::{
    circuit::{
        node_types::{CircuitGenericGate, CircuitNodeType},
        CircuitSimplifier,
    },
    signal_tracker::SignalTransformation,
    Circuit, Signal, TruthTable, UniqueSortedVec,
};

// ************************************************************************************************
// simplifier
// ************************************************************************************************

pub struct CircuitGenericPatternDetector {
    verbose: bool,
}

// ************************************************************************************************
// impl
// ************************************************************************************************

impl CircuitGenericPatternDetector {
    // ********************************************************************************************
    // helper functions
    // ********************************************************************************************

    fn detect_xor_pattern_if_exists(
        circuit: &Circuit,
        signal: &Signal,
        aggregator: &mut Vec<(Signal, TruthTable)>,
    ) -> bool {
        let parent_node = circuit.nodes.get(signal).unwrap().to_owned();

        // check if node is an and gate
        let inputs = match &parent_node.node_type {
            CircuitNodeType::And(a) => &a.inputs,
            _ => {
                return false;
            }
        };

        if inputs.len() != 2 {
            return false;
        }

        let a = inputs.peek()[0];
        let b = inputs.peek()[1];

        let a_node = circuit.nodes.get(&a.signal()).unwrap();
        let b_node = circuit.nodes.get(&b.signal()).unwrap();
        if !matches!(a_node.node_type, CircuitNodeType::And(_))
            || !matches!(b_node.node_type, CircuitNodeType::And(_))
        {
            return false;
        }

        let a_inputs = match &a_node.node_type {
            CircuitNodeType::And(x) => &x.inputs,
            _ => {
                return false;
            }
        };

        let b_inputs = match &b_node.node_type {
            CircuitNodeType::And(x) => &x.inputs,
            _ => {
                return false;
            }
        };

        if a_inputs.len() != 2 || b_inputs.len() != 2 {
            return false;
        }

        let aa = a_inputs.peek()[0];
        let ab = a_inputs.peek()[1];
        let ba = b_inputs.peek()[0];
        let bb = b_inputs.peek()[1];

        let mut seen = vec![aa.signal(), ab.signal(), ba.signal(), bb.signal()];
        seen.sort_unstable();
        seen.dedup();

        if seen.len() > 3 {
            // no pattern to detect
            return false;
        }

        let mut tt_aa = TruthTable::new_identity_truth_table(&aa.signal());
        if aa.is_negated() {
            tt_aa.negate();
        }

        let mut tt_ab = TruthTable::new_identity_truth_table(&ab.signal());
        if ab.is_negated() {
            tt_ab.negate();
        }

        let mut tt_ba = TruthTable::new_identity_truth_table(&ba.signal());
        if ba.is_negated() {
            tt_ba.negate();
        }

        let mut tt_bb = TruthTable::new_identity_truth_table(&bb.signal());
        if bb.is_negated() {
            tt_bb.negate();
        }

        let mut tt_a = TruthTable::and(tt_aa, tt_ab);
        if a.is_negated() {
            tt_a.negate();
        }

        let mut tt_b = TruthTable::and(tt_ba, tt_bb);
        if b.is_negated() {
            tt_b.negate();
        }

        let tt = TruthTable::and(tt_a, tt_b);

        // if seen.len() <= 2 || (seen.len() == 3 && tt.is_ite()) {
        // } else {
        //     false
        // }

        aggregator.push((*signal, tt));
        true
    }

    // ********************************************************************************************
    // API
    // ********************************************************************************************

    pub fn new(verbose: bool) -> Self {
        Self { verbose }
    }

    pub fn detect_4_input_cuts(&mut self, circuit: &mut Circuit) -> SignalTransformation {
        // get signals in order
        let signals: Vec<Signal> = circuit.nodes.iter_sorted().collect();
        let iterations = signals.len();
        let one_percent_iterations = iterations / 100;
        let ten_percent_iterations = iterations / 10;
        let initial_number_of_nodes = circuit.nodes.len();

        let mut aggregator = vec![];

        // go over all nodes and call pattern matcher.
        for (i, signal) in signals.iter().copied().enumerate() {
            // print progress
            if self.verbose && (ten_percent_iterations > 0) && i % ten_percent_iterations == 0 {
                println!(
                    "DETECTING PATTERNS, PATTERNS MATCHED: {}, progress = {}%",
                    aggregator.len(),
                    if one_percent_iterations > 0 {
                        i / one_percent_iterations
                    } else {
                        0
                    }
                );
            }

            Self::detect_xor_pattern_if_exists(circuit, &signal, &mut aggregator);
        }

        for (signal, tt) in aggregator {
            // let inputs = tt.get_signals().iter().map(|x| x.wire(false)).collect();
            circuit.nodes.get_mut(&signal).unwrap().node_type =
                CircuitNodeType::GenericGate(CircuitGenericGate { truth_table: tt });
        }

        // fix levels
        // circuit.fix_levels_and_users();

        debug_assert!(circuit.check().is_ok());

        if self.verbose {
            println!(
                "DONE! number of nodes before: {}, number of nodes after = {} ({}% reduction)",
                initial_number_of_nodes,
                circuit.nodes.len(),
                100.0
                    * (((initial_number_of_nodes - circuit.nodes.len()) as f32)
                        / (initial_number_of_nodes as f32)),
            );
        }

        SignalTransformation::SignalsRemovedBecauseTheyAreNotUsed(UniqueSortedVec::from_sequence(
            vec![],
        ))
    }
}

impl CircuitSimplifier for CircuitGenericPatternDetector {
    fn simplify(&mut self, circuit: &mut Circuit) -> SignalTransformation {
        self.detect_4_input_cuts(circuit)
    }

    fn title(&self) -> String {
        "Detect 4 input cuts".to_string()
    }
}
