// ************************************************************************************************
// use
// ************************************************************************************************

use super::{
    node_types::{CircuitGenericGate, CircuitNode, CircuitNodeType},
    Circuit, CircuitError,
};
use crate::models::{
    circuit::node_types::{CircuitAnd, CircuitLatch},
    circuit_builder::CircuitBuilder,
    AndInverterGraph, Signal, UniqueSortedHashMap, UniqueSortedVec, Wire,
};
use std::cmp::max;

// ************************************************************************************************
// impl
// ************************************************************************************************

impl Circuit {
    // ********************************************************************************************
    // helper functions
    // ********************************************************************************************

    fn check_less_than(
        a: Option<&Signal>,
        b: Option<&Signal>,
        e: CircuitError,
    ) -> Result<(), CircuitError> {
        match (a, b) {
            (None, None) => Ok(()),
            (None, Some(_)) => Ok(()),
            (Some(_), None) => Ok(()),
            (Some(x), Some(y)) => {
                if x < y {
                    Ok(())
                } else {
                    Err(e)
                }
            }
        }
    }

    // ********************************************************************************************
    // API
    // ********************************************************************************************

    /// Function takes AIG and creates a circuit that represent it.
    pub fn from_aig(aig: &AndInverterGraph) -> Self {
        let mut builder = CircuitBuilder::new();
        builder.add_ground();
        for i in aig.get_input_signals() {
            builder.add_input(i);
        }
        for l in aig.get_latch_information() {
            builder.add_latch(l.output, l.input, l.initial);
        }
        for and in aig.get_all_and_gates() {
            let v = UniqueSortedVec::from_sequence(vec![and.in0, and.in1]);
            builder.add_and_gate(and.out, v).unwrap()
        }
        for o in aig.get_output_wires() {
            builder.mark_as_output(o);
        }
        for b in aig.get_bad_wires() {
            builder.mark_as_bad(b);
        }
        for c in aig.get_constraints_wires() {
            builder.mark_as_invariant_constraint(c);
        }
        let (c, m) = builder.build().unwrap();
        // check that the mapping is the identity function
        debug_assert!(m.iter_pairs().all(|(a, b)| &a == b));
        // check that the circuit and AIG are the same
        debug_assert_eq!(&aig.get_input_signals(), c.get_input_signals().peek());
        debug_assert_eq!(
            &aig.get_latch_information()
                .into_iter()
                .map(|l| l.output)
                .collect::<Vec<Signal>>(),
            c.get_latch_signals().peek()
        );
        debug_assert_eq!(
            &aig.get_all_and_gates()
                .into_iter()
                .map(|a| a.out)
                .collect::<Vec<Signal>>(),
            c.get_gate_signals().peek()
        );
        debug_assert_eq!(
            &{
                let mut a = aig.get_output_wires();
                a.sort();
                a.dedup();
                a
            },
            c.get_output_wires().peek()
        );
        debug_assert_eq!(
            &{
                let mut a = aig.get_bad_wires();
                a.sort();
                a.dedup();
                a
            },
            c.get_bad_wires().peek()
        );
        debug_assert_eq!(
            &{
                let mut a = aig.get_constraints_wires();
                a.sort();
                a.dedup();
                a
            },
            c.get_invariant_constraint_wires().peek()
        );
        c
    }

    /// Create circuit using the following inputs
    pub fn new(
        inputs: UniqueSortedVec<Signal>,
        mut latch_details: Vec<(Signal, CircuitLatch)>,
        mut and_details: Vec<(Signal, CircuitAnd)>,
        mut generic_details: Vec<(Signal, CircuitGenericGate)>,
        outputs: UniqueSortedVec<Wire>,
        bad: UniqueSortedVec<Wire>,
        constraints: UniqueSortedVec<Wire>,
    ) -> Result<Self, CircuitError> {
        latch_details.sort_unstable_by_key(|x| x.0);
        and_details.sort_unstable_by_key(|x| x.0);
        generic_details.sort_unstable_by_key(|x| x.0);

        let latches =
            UniqueSortedVec::from_ordered_set(latch_details.iter().map(|x| x.0).collect());
        let ands = UniqueSortedVec::from_ordered_set(and_details.iter().map(|x| x.0).collect());
        let generics =
            UniqueSortedVec::from_ordered_set(generic_details.iter().map(|x| x.0).collect());
        let gates = ands.merge(&generics);

        {
            // check inputs then
            let i1 = inputs.min();
            Self::check_less_than(Some(&Signal::GROUND), i1, CircuitError::InputSignalTooSmall)?;
            let i2 = inputs.max();

            let l1 = latches.min();
            Self::check_less_than(Some(&Signal::GROUND), l1, CircuitError::LatchSignalTooSmall)?;
            Self::check_less_than(i2, l1, CircuitError::InputAndLatchSignalsIntersect)?;
            let l2 = latches.max();

            let g1 = gates.min();
            Self::check_less_than(Some(&Signal::GROUND), g1, CircuitError::GateSignalTooSmall)?;
            Self::check_less_than(i2, g1, CircuitError::InputAndGateSignalIntersect)?;
            Self::check_less_than(l2, g1, CircuitError::LatchAndGateSignalIntersect)?;
        }

        let greatest_signal = {
            let a = inputs.max().unwrap_or(&Signal::GROUND);
            let b = latches.max().unwrap_or(&Signal::GROUND);
            let c = gates.max().unwrap_or(&Signal::GROUND);
            *max(max(a, b), c)
        };

        let mut result = Self {
            greatest_signal,
            nodes: UniqueSortedHashMap::new(greatest_signal),
            inputs,
            latches,
            gates,
            outputs,
            bad,
            constraints,
            important_signals: UniqueSortedVec::new(),
        };

        // add ground
        let mut current_max_signal_without_gates = Signal::GROUND;
        result.add_ground_if_possible();

        // add inputs
        for i in result.inputs.iter() {
            debug_assert!(current_max_signal_without_gates < *i);
            current_max_signal_without_gates = max(current_max_signal_without_gates, *i);
            result.nodes.insert(
                *i,
                CircuitNode {
                    node_type: CircuitNodeType::Input,
                },
            );
        }

        // add latches
        for (s, l) in latch_details.into_iter() {
            debug_assert!(current_max_signal_without_gates < s);
            current_max_signal_without_gates = max(current_max_signal_without_gates, s);
            result.nodes.insert(
                s,
                CircuitNode {
                    node_type: CircuitNodeType::Latch(l),
                },
            );
        }

        // add and gates
        for (s, a) in and_details.into_iter() {
            debug_assert!(current_max_signal_without_gates < s);

            // check inputs
            if a.inputs.is_empty() {
                return Err(CircuitError::AndGateWithNoInputs);
            }
            if a.inputs.iter().map(|w| w.signal()).max().unwrap() >= s {
                return Err(CircuitError::AndGateWithInputGreaterOrEqualToIt);
            }

            for w in a.inputs.iter() {
                let s = w.signal();
                if (!result.nodes.contains_key(&s)) && (!generics.contains(&s)) {
                    return Err(CircuitError::InputToAndGateDoesNotExist);
                }
            }
            result.nodes.insert(
                s,
                CircuitNode {
                    node_type: CircuitNodeType::And(a),
                },
            );
        }

        // add generic gates
        for (s, g) in generic_details.into_iter() {
            debug_assert!(current_max_signal_without_gates < s);

            if g.truth_table.get_signals().is_empty() {
                return Err(CircuitError::GenericGateWithNoInput);
            }
            if *g.truth_table.get_signals().iter().max().unwrap() >= s {
                return Err(CircuitError::GenericGateWithInputGreaterOrEqualToIt);
            }

            for s in g.truth_table.get_signals().iter() {
                if !result.nodes.contains_key(s) {
                    return Err(CircuitError::InputToGenericGateDoesNotExist);
                }
            }
            // if &signals != g.truth_table.get_signals().peek() {
            //     return Err(CircuitError::TruthTableSignalsDoNotMatchGateInputs);
            // }
            result.nodes.insert(
                s,
                CircuitNode {
                    node_type: CircuitNodeType::GenericGate(g),
                },
            );
        }

        // check latch inputs
        let latches = result.latches.to_owned();
        for s in latches.iter() {
            let l = result.nodes.get(s).unwrap();
            let l = match &l.node_type {
                CircuitNodeType::Latch(l) => l,
                _ => unreachable!(),
            };
            if !result.nodes.contains_key(&l.input.signal()) {
                return Err(CircuitError::InputToLatchDoesNotExist);
            }
        }

        // check outputs
        for o in result.outputs.iter() {
            if !result.nodes.contains_key(&o.signal()) {
                return Err(CircuitError::OutputWireDoesNotExist);
            }
        }

        // check bad
        for b in result.bad.iter() {
            if !result.nodes.contains_key(&b.signal()) {
                return Err(CircuitError::BadWireDoesNotExist);
            }
        }

        // check constraints
        for c in result.constraints.iter() {
            if !result.nodes.contains_key(&c.signal()) {
                return Err(CircuitError::ConstraintWireDoesNotExist);
            }
        }

        // fix levels and users
        // result.fix_levels_and_users();
        result.important_signals = result.recalculate_important_signals();
        result.remove_ground_if_possible();
        debug_assert!(result.check().is_ok());
        Ok(result)
    }
}
