//! object for representing a circuit in memory, this is more expressive than the AIGER.
//! For example XOR here is represented as a single node, while in AIGER it is represented as 3.

// ************************************************************************************************
// use
// ************************************************************************************************

// use crate::models::and_inverter_graph::aig_node::AIGNode;
use self::node_types::CircuitNode;
use super::signal_tracker::SignalTransformation;
use super::UniqueSortedHashMap;
use super::UniqueSortedVec;
use crate::models::Signal;
use crate::models::Wire;

// ************************************************************************************************
// struct
// ************************************************************************************************

#[derive(Clone, Debug)]
pub enum CircuitError {
    OutputWireDoesNotExist,
    BadWireDoesNotExist,
    ConstraintWireDoesNotExist,
    InputSignalTooSmall,
    LatchSignalTooSmall,
    GateSignalTooSmall,
    InputAndLatchSignalsIntersect,
    LatchAndGateSignalIntersect,
    InputAndGateSignalIntersect,
    InputToLatchDoesNotExist,
    InputToAndGateDoesNotExist,
    InputToGenericGateDoesNotExist,
    // TruthTableSignalsDoNotMatchGateInputs,
    AndGateWithNoInputs,
    GenericGateWithNoInput,
    AndGateWithInputGreaterOrEqualToIt,
    GenericGateWithInputGreaterOrEqualToIt,
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Struct that describes memory layout of the Circuit
pub struct Circuit {
    greatest_signal: Signal,

    nodes: UniqueSortedHashMap<Signal, CircuitNode>, /* [0..maxvar] */

    // these contain indexes that are in nodes that have these nodes.
    inputs: UniqueSortedVec<Signal>,
    latches: UniqueSortedVec<Signal>,
    gates: UniqueSortedVec<Signal>,

    // these contain wires.
    outputs: UniqueSortedVec<Wire>,
    bad: UniqueSortedVec<Wire>,
    constraints: UniqueSortedVec<Wire>,

    // important signals
    important_signals: UniqueSortedVec<Signal>,
}

// ************************************************************************************************
// trait
// ************************************************************************************************

/// Trait that circuit simplification techniques should implement
pub trait CircuitSimplifier {
    fn simplify(&mut self, circuit: &mut Circuit) -> SignalTransformation;
    fn title(&self) -> String;
}

// ************************************************************************************************
// rust submodule declaration, they get searched in their respective file  names
// ************************************************************************************************

pub mod check;
pub mod construction;
pub mod cut_enumeration;
pub mod dot;
pub mod fixes;
pub mod get_nodes;
pub mod get_wires;
pub mod graph;
pub mod node_types;
pub mod simplifiers;
pub mod technology_map_area_flow;
pub mod technology_map_min_popularity;
pub mod technology_map_minimizing_depth;
pub mod technology_map_using_levels;
pub mod un_constraint;

// ************************************************************************************************
// re-exports of structs in these modules to simplify paths for other imports
// ************************************************************************************************
