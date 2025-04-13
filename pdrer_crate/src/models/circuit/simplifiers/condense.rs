// ************************************************************************************************
// use
// ************************************************************************************************

use crate::models::{
    circuit::{node_types::CircuitNodeType, CircuitSimplifier},
    signal_tracker::SignalTransformation,
    Circuit, Signal, TruthTable, UniqueSortedHashMap, Wire,
};

// ************************************************************************************************
// simplifier
// ************************************************************************************************

pub struct CircuitCondenser {}

type M = UniqueSortedHashMap<Signal, Signal>;

// ************************************************************************************************
// impl
// ************************************************************************************************

impl CircuitCondenser {
    // ********************************************************************************************
    // helper functions
    // ********************************************************************************************

    fn get_old_signal_to_new_signal_mapping(circuit: &Circuit) -> M {
        let mut i = if circuit.nodes.contains_key(&Signal::GROUND) {
            Signal::GROUND
        } else {
            Signal::new(1)
        };
        let mut old_signal_to_new_signal =
            UniqueSortedHashMap::new(circuit.nodes.max_key().unwrap());

        for old_signal in circuit.nodes.iter_sorted() {
            old_signal_to_new_signal.insert(old_signal, i);
            i = Signal::new(i.number() + 1);
        }

        old_signal_to_new_signal
    }

    fn map_signal(signal: &Signal, mapping: &M) -> Signal {
        *mapping.get(signal).unwrap()
    }

    fn map_wire(wire: &Wire, mapping: &M) -> Wire {
        let s = Self::map_signal(&wire.signal(), mapping);
        s.wire(wire.is_negated())
    }

    // ********************************************************************************************
    // API
    // ********************************************************************************************

    pub fn new() -> Self {
        Self {}
    }

    pub fn condense(circuit: &mut Circuit) -> SignalTransformation {
        // get signals in order
        let m = Self::get_old_signal_to_new_signal_mapping(circuit);

        // println!("Mapping:\n{}", m);

        circuit.greatest_signal = Self::map_signal(&circuit.greatest_signal, &m);

        for wires in [
            &mut circuit.outputs,
            &mut circuit.constraints,
            &mut circuit.bad,
        ] {
            wires.perform_operation_on_each_value(|w| *w = Self::map_wire(w, &m));
        }

        for signals in [
            &mut circuit.inputs,
            &mut circuit.latches,
            &mut circuit.gates,
            &mut circuit.important_signals,
        ] {
            signals.perform_operation_on_each_value(|s| *s = Self::map_signal(s, &m));
        }

        let circuit_signals: Vec<Signal> = circuit.nodes.iter_sorted().collect();
        for old_signal in circuit_signals {
            let mut tmp_node = circuit.nodes.remove(&old_signal).unwrap();
            // for signals in [&mut tmp_node.users, &mut tmp_node.internal_users] {
            //     signals.perform_operation_on_each_value(|s| *s = Self::map_signal(s, &m));
            // }
            match &mut tmp_node.node_type {
                CircuitNodeType::ConstantZero => {}
                CircuitNodeType::Input => {}
                CircuitNodeType::Latch(l) => {
                    l.input = Self::map_wire(&l.input, &m);
                }
                CircuitNodeType::And(a) => {
                    // println!("a.inputs before = {:?}", a.inputs.peek());
                    a.inputs
                        .perform_operation_on_each_value(|w| *w = Self::map_wire(w, &m));
                    // println!("a.inputs after = {:?}", a.inputs.peek());
                }
                CircuitNodeType::GenericGate(g) => {
                    // g.inputs
                    //     .perform_operation_on_each_value(|w| *w = Self::map_wire(w, &m));
                    // println!("g.inputs before = {:?}", g.truth_table.get_signals().peek());
                    g.truth_table =
                        TruthTable::new_truth_table_with_signals_renamed(&g.truth_table, |s| {
                            Self::map_signal(s, &m)
                        });
                    // println!("g.inputs after = {:?}", g.truth_table.get_signals().peek());
                }
            }

            let new_signal = Self::map_signal(&old_signal, &m);
            circuit.nodes.insert(new_signal, tmp_node);
        }
        circuit.nodes.resize_with(circuit.greatest_signal);

        SignalTransformation::SignalReorder(m)
    }
}

impl CircuitSimplifier for CircuitCondenser {
    fn title(&self) -> String {
        "Condense circuit".to_string()
    }

    fn simplify(&mut self, circuit: &mut Circuit) -> SignalTransformation {
        Self::condense(circuit)
    }
}
