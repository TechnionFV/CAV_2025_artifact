// ************************************************************************************************
// use
// ************************************************************************************************

use crate::models::{ternary_value::TernaryValue, AndInverterGraph, Signal, Wire};

// ************************************************************************************************
// struct
// ************************************************************************************************

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Latch {
    pub input: Wire,
    pub initial: TernaryValue,
    pub output: Signal,
}

// ************************************************************************************************
// impl
// ************************************************************************************************

impl AndInverterGraph {
    // ********************************************************************************************
    // aig getting node info
    // ********************************************************************************************

    /// Function that gets a vector describing the latch nodes in the system.
    /// The output is a vector containing tuple with a length of 3,
    /// representing latch information :
    /// ```text
    /// // (latch output literal, latch input literal, latch initial value)
    /// //                   ___________
    /// //                  |           |
    /// // latch input ---> |   latch   | --> latch output
    /// //                  |___________|
    /// //                        ^
    /// //                        |
    /// //               latch initial value
    /// ```
    /// # Arguments
    ///
    /// * `&self` - the AndInverterGraph desired.
    ///
    /// # Examples
    /// ```
    /// use rust_formal_verification::models::{and_inverter_graph::{Latch}, TernaryValue, Signal, Wire, AndInverterGraph};
    ///
    /// let aig = AndInverterGraph::new(
    ///     Signal::new(5),
    ///     0,
    ///     &[
    ///         (Signal::new(5).wire(false), TernaryValue::False),
    ///         (Signal::new(1).wire(false), TernaryValue::False),
    ///         (Signal::new(2).wire(false), TernaryValue::False),
    ///     ],
    ///     vec![Signal::new(5).wire(false)],
    ///     vec![],
    ///     vec![],
    ///     &[
    ///         (Signal::new(2).wire(true), Signal::new(3).wire(true)),
    ///         (
    ///             Signal::new(1).wire(true),
    ///             Signal::new(4).wire(false),
    ///         ),
    ///     ],
    ///     String::new(),
    /// )
    /// .unwrap();
    /// assert_eq!(vec![
    ///     Latch { input: Wire::new(10), initial: TernaryValue::False, output: Signal::new(1) },
    ///     Latch { input: Wire::new(2), initial: TernaryValue::False, output: Signal::new(2) },
    ///     Latch { input: Wire::new(4), initial: TernaryValue::False, output: Signal::new(3) }
    /// ], aig.get_latch_information());
    /// ```
    pub fn get_latch_information(&self) -> Vec<Latch> {
        let mut result = Vec::with_capacity(self.number_of_latches as usize);
        let from = 1 + self.number_of_inputs;
        let to = from + self.number_of_latches;
        for latch_index in (from..to).map(Signal::new) {
            let latch = &self.nodes.get(&latch_index).unwrap();

            let latch_literal = latch_index.wire(false);
            let latch_input = latch.get_latch_input();
            let latch_reset = latch.get_latch_reset();

            debug_assert!(
                latch_reset == Wire::new(0)
                    || latch_reset == Wire::new(1)
                    || latch_reset == latch_literal
            );
            let r = if latch_reset == Wire::new(0) {
                TernaryValue::False
            } else if latch_reset == Wire::new(1) {
                TernaryValue::True
            } else {
                TernaryValue::X
            };

            let l = Latch {
                input: latch_input,
                initial: r,
                output: latch_literal.signal(),
            };
            result.push(l);
        }
        result
    }

    pub fn wires_that_feed_into_latches(&self) -> Vec<Wire> {
        self.get_latch_information()
            .iter()
            .map(|l| l.input)
            .collect()
    }
}
