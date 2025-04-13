//! Creating and manipulating circuits efficiently.
//! This is because circuits have very specific layouts, which don't allow for dynamic creation
//! efficiently. For example, latches must be after inputs, and gates must be after latches.
//! Thus adding a latch after a gate would require renumbering all the signals.
//! So to add 5 latches after 5 gates, we would have to renumber 5+5+5=15 signals.
//! This is why the `CircuitBuilder` is used to create or manipulate circuits.

// ************************************************************************************************
// use
// ************************************************************************************************

use fxhash::FxHashMap;

use super::{
    circuit::node_types::CircuitNodeType, Circuit, Signal, TernaryValue, TruthTable,
    UniqueSortedVec, Wire,
};

// ************************************************************************************************
// struct
// ************************************************************************************************

#[derive(Clone)]
/// Object that enables the creation of circuits efficiently.
/// This is because circuits have very specific layouts, which don't allow for dynamic creating.
pub struct CircuitBuilder {
    signals: FxHashMap<Signal, CircuitNodeType>,
    max_signal: Signal,
    outputs: Vec<Wire>,
    invariant_constraints: Vec<Wire>,
    bad: Vec<Wire>,
}

// ************************************************************************************************
// rust submodule declaration, they get searched in their respective file  names
// ************************************************************************************************

pub mod build;
pub mod construction;
pub mod operations;

// ************************************************************************************************
// re-exports of structs in these modules to simplify paths for other imports
// ************************************************************************************************
