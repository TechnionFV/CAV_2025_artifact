// ************************************************************************************************
// use
// ************************************************************************************************

use std::{cell::RefCell, rc::Rc};

// choose which ternary_value_vector implementation to use
use ternary_value_vector_1::TernaryValueVector;

use super::{Circuit, Signal, TimeStats, TruthTable, UniqueSortedHashMap, UniqueSortedVec, Wire};

// ************************************************************************************************
// types
// ************************************************************************************************

#[derive(Clone, Debug)]
enum SimulationGate {
    And(UniqueSortedVec<Wire>),
    Generic(TruthTable),
}

// ************************************************************************************************
// constants
// ************************************************************************************************

// in order to use less memory, we cram the simulation state in a Vec<usize> where each usize
// represents multiple elements. This constant defines how many elements are in a usize.
// const ELEMENT_SIZE_IN_BYTES: u32 = Element::BITS / 8;

// ************************************************************************************************
// struct
// ************************************************************************************************

#[derive(Clone, Debug)]
pub struct CircuitSimulator {
    // The state of the simulation, this makes it a lot more efficient.
    simulation_state: TernaryValueVector,
    cleared_simulation_state: TernaryValueVector,
    gates: Vec<Option<SimulationGate>>,
    _internal_users: UniqueSortedHashMap<Signal, UniqueSortedVec<Signal>>,
    first_gate: Option<Signal>,
    max_delta_between_node_and_its_users: usize,

    /// time statistics
    time_stats: Rc<RefCell<TimeStats>>,
}

// ************************************************************************************************
// rust submodule declaration, they get searched in their respective file  names
// ************************************************************************************************

pub mod construction;
pub mod simulation;
pub mod ternary_value_vector_1;
pub mod ternary_value_vector_2;

// ************************************************************************************************
// re-exports of structs in these modules to simplify paths for other imports
// ************************************************************************************************
