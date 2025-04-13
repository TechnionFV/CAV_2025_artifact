// ************************************************************************************************
// use
// ************************************************************************************************

use crate::models::and_inverter_graph::aig_node::AIGNodeType;
use crate::models::and_inverter_graph::AndInverterGraph;
use crate::models::Signal;
use crate::models::Utils;
use crate::models::Wire;

// ************************************************************************************************
// impl
// ************************************************************************************************

impl AndInverterGraph {
    // ********************************************************************************************
    // helper functions
    // ********************************************************************************************

    fn check_node(
        &self,
        expected_signal: &Signal,
        _expected_type: AIGNodeType,
        // expected_level: Option<usize>,
        signal_number_span: (u32, u32),
    ) -> Result<(), String> {
        let node = self.nodes.get(expected_signal).unwrap();
        Utils::ensure(
            matches!(node.get_type(), _expected_type),
            "Wrong node type.",
        )?;
        // Utils::ensure(node.get_wire().signal() == *expected_signal, "Wrong wire.")?;
        Utils::ensure(
            signal_number_span.0 <= expected_signal.number(),
            "Wrong signal number.",
        )?;
        Utils::ensure(
            expected_signal.number() <= signal_number_span.1,
            "Wrong signal number.",
        )?;
        Ok(())
    }

    // ********************************************************************************************
    // aig creator
    // ********************************************************************************************

    pub(super) fn check(&self) -> Result<(), String> {
        // basic check
        Utils::ensure(
            self.maximum_variable_index
                == (self.number_of_inputs + self.number_of_latches + self.number_of_and_gates),
            "The number of variables does not add up.",
        )?;
        Utils::ensure(
            self.number_of_fairness_constraints == 0,
            "Fairness is currently unsupported.",
        )?;
        Utils::ensure(
            self.number_of_justice_constraints == 0,
            "Justice is currently unsupported.",
        )?;
        Utils::ensure(
            self.nodes.len() as u32 == self.maximum_variable_index + 1,
            "The number of nodes does not add up.",
        )?;

        // check that constant
        self.check_node(
            &Signal::new(0),
            AIGNodeType::ConstantZero,
            // Some(0),
            (0, 0),
        )?;

        // inputs
        for input_index in &self.get_input_signals() {
            self.check_node(
                input_index,
                AIGNodeType::Input,
                // Some(0),
                (1, self.number_of_inputs),
            )?;
        }

        // latches
        for latch_index in &self.get_latch_information() {
            let latch_index = &latch_index.output.wire(false).signal();
            self.check_node(
                latch_index,
                AIGNodeType::Latch {
                    input: Wire::MAX,
                    reset: Wire::MAX,
                },
                // Some(0),
                (
                    1 + self.number_of_inputs,
                    self.number_of_inputs + self.number_of_latches,
                ),
            )?;

            let node = self.nodes.get(latch_index).unwrap();

            // check reset
            let reset = node.get_latch_reset();
            Utils::ensure(
                reset == Wire::new(0) || reset == Wire::new(1) || reset == latch_index.wire(false),
                "Wrong reset wire.",
            )?;

            // check input to latch
            let input = node.get_latch_input();
            Utils::ensure(
                self.nodes.contains_key(&input.signal()),
                "Latch input is not a node.",
            )?;
        }
        // ands
        for and_index in &self.get_all_and_gates() {
            let and_index = &and_index.out;
            self.check_node(
                and_index,
                AIGNodeType::And {
                    input0: Wire::MAX,
                    input1: Wire::MAX,
                },
                // None,
                (
                    1 + self.number_of_inputs + self.number_of_latches,
                    self.maximum_variable_index,
                ),
            )?;

            let node = self.nodes.get(and_index).unwrap();
            let wire = and_index.wire(false);
            // let mut expected_level = 0;

            // check both inputs
            let (in0, in1) = (node.get_and_rhs0(), node.get_and_rhs1());
            for inp in &[in0, in1] {
                Utils::ensure(
                    self.nodes.contains_key(&inp.signal()),
                    "Input is not a node.",
                )?;
            }
            Utils::ensure(wire > in0 && in0 >= in1, "Wrong input order.")?;
        }

        // check output, constraint and bad wires
        for (expected_len, list) in [
            (self.number_of_outputs, self.outputs.to_owned()),
            (self.number_of_bad_state_constraints, self.bad.to_owned()),
            (
                self.number_of_invariant_constraints,
                self.constraints.to_owned(),
            ),
        ] {
            Utils::ensure(expected_len == list.len() as u32, "Wrong number of wires.")?;
            for x in list.iter() {
                Utils::ensure(self.nodes.contains_key(&x.signal()), "Not a node.")?;
            }
        }

        Ok(())
    }
}
