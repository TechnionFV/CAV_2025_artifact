// ************************************************************************************************
// use
// ************************************************************************************************

use fxhash::FxHashMap;

use crate::{
    formulas::{Literal, Variable},
    models::{
        and_inverter_graph::AndGate, definition::DefinitionFunction, AndInverterGraph, Signal,
        SignalTracker, TernaryValue, UniqueSortedVec, Wire,
    },
};

use super::Proof;
// ************************************************************************************************
// types
// ************************************************************************************************

type Map = FxHashMap<Variable, Wire>;

// ************************************************************************************************
// impl
// ************************************************************************************************

impl Proof {
    // ********************************************************************************************
    // impl
    // ********************************************************************************************

    fn get_variable_to_wire_map<F: Fn(Signal) -> Variable>(
        tracker: &SignalTracker,
        highest_signal: Signal,
        s2v: F,
    ) -> Map {
        let mut map: Map = Default::default();
        for i in 1..=highest_signal.number() {
            let signal = Signal::new(i);
            match tracker.get(signal) {
                Some(final_signal) => {
                    if final_signal.is_constant() {
                        continue;
                    }
                    let v = s2v(final_signal);
                    map.insert(v, signal.wire(false));
                }
                None => {
                    // signal was removed and so no variable would use this signal
                }
            }
        }
        map
    }

    fn get_identical_wires(tracker: &SignalTracker, highest_signal: Signal) -> Vec<(Wire, Wire)> {
        // let mut map = Map::new();
        let mut v = Vec::new();
        for i in 1..=highest_signal.number() {
            let signal = Signal::new(i);
            if let Some(w) = tracker.find_equivalent_if_removed(signal) {
                v.push((signal.wire(false), w));
            }
        }
        v
    }

    fn literal_to_wire(map: &Map, literal: Literal) -> Wire {
        let is_negated = literal.is_negated();
        let w = *map.get(&literal.variable()).unwrap();
        if is_negated {
            !w
        } else {
            w
        }
    }

    fn literals_to_wires(map: &Map, literals: &[Literal]) -> Vec<Wire> {
        literals
            .iter()
            .map(|l| Self::literal_to_wire(map, *l))
            .collect()
    }

    // ********************************************************************************************
    // Gate definitions
    // ********************************************************************************************

    fn get_free_signal(and_gates: &[AndGate]) -> Signal {
        Signal::new(and_gates.last().unwrap().out.number() + 1)
    }

    fn define_and_of_2_wires(a: Wire, b: Wire, and_gates: &mut Vec<AndGate>) -> Wire {
        let out = Self::get_free_signal(and_gates);
        and_gates.push(AndGate {
            in0: a,
            in1: b,
            out,
        });
        out.wire(false)
    }

    fn define_or_of_2_wires(a: Wire, b: Wire, and_gates: &mut Vec<AndGate>) -> Wire {
        let out = Self::get_free_signal(and_gates);
        and_gates.push(AndGate {
            in0: !a,
            in1: !b,
            out,
        });
        out.wire(true)
    }

    fn define_xor_of_2_wires(a: Wire, b: Wire, and_gates: &mut Vec<AndGate>) -> Wire {
        let x = Self::get_free_signal(and_gates);
        and_gates.push(AndGate {
            in0: a,
            in1: b,
            out: x,
        });
        let y = Self::get_free_signal(and_gates);
        and_gates.push(AndGate {
            in0: !a,
            in1: !b,
            out: y,
        });
        let out = Self::get_free_signal(and_gates);
        and_gates.push(AndGate {
            in0: x.wire(true),
            in1: y.wire(true),
            out,
        });
        out.wire(false)
    }

    fn define_signal_as_bitwise_operation_on_wires<F>(
        f: F,
        wires: Vec<Wire>,
        and_gates: &mut Vec<AndGate>,
    ) -> Wire
    where
        F: Fn(Wire, Wire, &mut Vec<AndGate>) -> Wire,
    {
        // assert!(wires.len() > 1);
        let mut result = wires[0];
        for w in wires.iter().skip(1) {
            result = f(result, *w, and_gates);
        }
        result
    }

    // ********************************************************************************************
    // impl
    // ********************************************************************************************

    /// Get the original AIGER, a proof that was produced with the associated circuit as well
    /// as a mapping between the AIG signals to the circuit signals, and encodes the proof into
    /// a witness than can then be checked by certifaiger.
    pub fn get_certifaiger_witness<F: Fn(Signal) -> Variable>(
        &self,
        tracker: &SignalTracker,
        aig: &AndInverterGraph,
        s2v: F,
    ) -> AndInverterGraph {
        let inputs = aig.get_input_signals();
        let latches: Vec<(Wire, TernaryValue)> = aig
            .get_latch_information()
            .iter()
            .map(|x| (x.input, x.initial))
            .collect();
        let mut output_wires = aig.get_output_wires();
        let mut bad_wires = aig.get_bad_wires();
        let constraint_wires = aig.get_constraints_wires();
        let mut and_gates = aig.get_all_and_gates();
        let comments = format!(
            "c\nWITNESS for some AIG file, this file should be checked by certifaiger.\nOriginal output wires: {}\nOriginal bad wires: {}\nOriginal constraint wires: {}",
            // aig.get_comments(),
            UniqueSortedVec::from_sequence(output_wires.to_owned()),
            UniqueSortedVec::from_sequence(bad_wires.to_owned()),
            UniqueSortedVec::from_sequence(constraint_wires.to_owned())
        );

        let mut variable_to_wire = Self::get_variable_to_wire_map(
            tracker,
            aig.get_highest_non_negated_wire().signal(),
            s2v,
        );

        if and_gates.is_empty() {
            // add this gate to make code easier to write
            and_gates.push(AndGate {
                in0: Wire::new(0),
                in1: Wire::new(0),
                out: Signal::new(aig.get_highest_non_negated_wire().signal().number() + 1),
            });
        }

        for d in self.definitions.iter() {
            let inputs = Self::literals_to_wires(&variable_to_wire, d.inputs.peek().peek());
            let w = match d.function {
                DefinitionFunction::And => Self::define_signal_as_bitwise_operation_on_wires(
                    Self::define_and_of_2_wires,
                    inputs,
                    &mut and_gates,
                ),
                DefinitionFunction::Xor => Self::define_signal_as_bitwise_operation_on_wires(
                    Self::define_xor_of_2_wires,
                    inputs,
                    &mut and_gates,
                ),
            };
            variable_to_wire.insert(d.variable, w);
        }

        let mut clause_wires = Vec::with_capacity(self.invariant.len());
        for clause in self.invariant.iter() {
            let inputs = Self::literals_to_wires(&variable_to_wire, clause.peek().peek().peek());
            let w = Self::define_signal_as_bitwise_operation_on_wires(
                Self::define_or_of_2_wires,
                inputs,
                &mut and_gates,
            );
            clause_wires.push(w);
        }
        for (a, b) in
            Self::get_identical_wires(tracker, aig.get_highest_non_negated_wire().signal())
        {
            let is_latch_signal =
                |s: Signal| aig.get_latch_information().iter().any(|x| x.output == s);
            if !is_latch_signal(a.signal()) || !is_latch_signal(b.signal()) {
                continue;
            }
            for clause in [vec![a, !b], vec![!a, b]] {
                let w = Self::define_signal_as_bitwise_operation_on_wires(
                    Self::define_or_of_2_wires,
                    clause,
                    &mut and_gates,
                );
                clause_wires.push(w);
            }
        }

        let invariant_wire = if self.all_initial_states_violate_constraints {
            Wire::CONSTANT_ZERO
        } else if !clause_wires.is_empty() {
            Self::define_signal_as_bitwise_operation_on_wires(
                Self::define_and_of_2_wires,
                clause_wires,
                &mut and_gates,
            )
        } else {
            Wire::CONSTANT_ONE
        };

        let constraints_wire = if !constraint_wires.is_empty() {
            Self::define_signal_as_bitwise_operation_on_wires(
                Self::define_and_of_2_wires,
                constraint_wires.clone(),
                &mut and_gates,
            )
        } else {
            Wire::CONSTANT_ONE
        };

        for wires in [&mut bad_wires, &mut output_wires] {
            if !wires.is_empty() {
                let mut and_between: Vec<Wire> = wires.iter().map(|x| !*x).collect();
                for w in [invariant_wire, constraints_wire].iter() {
                    and_between.push(*w);
                }
                and_between.sort_unstable();
                and_between.dedup();
                *wires = vec![!Self::define_signal_as_bitwise_operation_on_wires(
                    Self::define_and_of_2_wires,
                    and_between,
                    &mut and_gates,
                )];
            }
        }

        AndInverterGraph::new(
            Signal::new((inputs.len() + latches.len() + and_gates.len()) as u32),
            inputs.len() as u32,
            &latches,
            output_wires,
            bad_wires,
            constraint_wires,
            &and_gates
                .into_iter()
                .map(|a| (a.in0, a.in1))
                .collect::<Vec<_>>(),
            comments.clone(),
        )
        .unwrap()
    }
}
