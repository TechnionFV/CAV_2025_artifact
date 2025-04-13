//! object for reading .aig files, turning them into .aag files, and scanning them in different
//! ways.

// ************************************************************************************************
// use
// ************************************************************************************************

use crate::models::and_inverter_graph::aig_node::AIGNode;
use crate::models::Signal;
use crate::models::Wire;

// ************************************************************************************************
// struct
// ************************************************************************************************

/// Struct that describes memory layout of the AIG.
///
/// implementations of many additional features can be found in sub-modules.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AndInverterGraph {
    maximum_variable_index: u32,
    number_of_inputs: u32,
    number_of_latches: u32,
    number_of_outputs: u32,
    number_of_and_gates: u32,
    number_of_bad_state_constraints: u32,
    number_of_invariant_constraints: u32,
    number_of_justice_constraints: u32,
    number_of_fairness_constraints: u32,

    nodes: UniqueSortedHashMap<Signal, AIGNode>, /* [0..maxvar] */

    // these contain wires.
    outputs: Vec<Wire>,
    bad: Vec<Wire>,
    constraints: Vec<Wire>,

    // comments
    comments: String,

    // symbols
    input_symbols: Vec<(u32, String)>,
    latch_symbols: Vec<(u32, String)>,
    output_symbols: Vec<(u32, String)>,
    bad_symbols: Vec<(u32, String)>,
    constraint_symbols: Vec<(u32, String)>,
}

// ************************************************************************************************
// rust submodule declaration, they get searched in their respective file  names
// ************************************************************************************************

mod aig_node;
pub mod check;
pub mod construction;
pub mod conversion;
pub mod get_and;
pub mod get_latches;
pub mod get_wires;
pub mod random;

// ************************************************************************************************
// re-exports of structs in these modules to simplify paths for other imports
// ************************************************************************************************

pub use get_and::AndGate;
pub use get_latches::Latch;

use super::UniqueSortedHashMap;
