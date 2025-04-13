// ************************************************************************************************
// use
// ************************************************************************************************

use crate::models::{AndInverterGraph, Signal, Wire};

// ************************************************************************************************
// impl
// ************************************************************************************************

impl AndInverterGraph {
    /// Function that gets a vector describing the input nodes in the system.
    /// The output is a vector containing number representing input literals.
    ///
    /// # Arguments
    ///
    /// * `&self` - the AndInverterGraph desired.
    ///
    /// # Examples
    /// ```
    /// use rust_formal_verification::models::{Signal, AndInverterGraph, Wire, TernaryValue};
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
    /// assert_eq!(Vec::<Signal>::new(), aig.get_input_signals());
    /// ```
    pub fn get_input_signals(&self) -> Vec<Signal> {
        let from = 1;
        let to = from + self.number_of_inputs;
        (from..to).map(Signal::new).collect()
    }

    /// Function that gets a vector describing the bad nodes in the system.
    /// The output is a vector containing usize numbers, these are the literals
    /// that are bad.
    ///
    /// # Arguments
    ///
    /// * `&self` - the AndInverterGraph desired.
    ///
    /// # Examples
    /// ```
    /// use rust_formal_verification::models::{Signal, AndInverterGraph, Wire, TernaryValue};
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
    /// assert_eq!(Vec::<Wire>::new(), aig.get_bad_wires());
    /// ```
    pub fn get_bad_wires(&self) -> Vec<Wire> {
        self.bad.clone()
    }

    /// Function that gets a vector describing the constraints nodes in the system.
    /// The output is a vector containing usize numbers, these are the literals
    /// that are constraint.
    ///
    /// # Arguments
    ///
    /// * `&self` - the AndInverterGraph desired.
    ///
    /// # Examples
    /// ```
    /// use rust_formal_verification::models::{Signal, AndInverterGraph, Wire, TernaryValue};
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
    /// assert_eq!(Vec::<Wire>::new(), aig.get_constraints_wires());
    /// ```
    pub fn get_constraints_wires(&self) -> Vec<Wire> {
        self.constraints.clone()
    }

    /// Function that gets a vector describing the output nodes in the system.
    /// The output is a vector containing usize numbers, these are the literals
    /// that are outputs of the AIG.
    ///
    /// # Arguments
    ///
    /// * `&self` - the AndInverterGraph desired.
    ///
    /// # Examples
    /// ```
    /// use rust_formal_verification::models::{Signal, AndInverterGraph, Wire, TernaryValue};
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
    /// let mut expected_result = Vec::<Wire>::new();
    /// expected_result.push(Wire::new(10));
    /// assert_eq!(expected_result, aig.get_output_wires());
    /// ```
    pub fn get_output_wires(&self) -> Vec<Wire> {
        self.outputs.clone()
    }

    /// Function that gets the maximum variable number used in the AIG.
    ///
    /// # Arguments
    ///
    /// * `&self` - the AndInverterGraph desired.
    ///
    /// # Examples
    /// ```
    /// use rust_formal_verification::models::{Signal, AndInverterGraph, Wire, TernaryValue};
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
    /// assert_eq!(Wire::new(10), aig.get_highest_non_negated_wire());
    /// ```
    pub fn get_highest_non_negated_wire(&self) -> Wire {
        Wire::new(self.maximum_variable_index << 1)
    }

    /// Get the comments of the AIG.
    /// The comments are a string that can be used to describe the AIG.
    ///
    /// # Arguments
    ///
    /// * `&self` - the AndInverterGraph desired.
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_formal_verification::models::{Signal, AndInverterGraph, Wire, TernaryValue};
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
    ///     String::from("This is a comment."),
    /// )
    /// .unwrap();
    /// assert_eq!(&"This is a comment.".to_string(), aig.get_comments());
    /// ```
    pub fn get_comments(&self) -> &String {
        &self.comments
    }
}
