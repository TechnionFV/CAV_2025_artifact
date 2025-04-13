// ************************************************************************************************
// use
// ************************************************************************************************

use super::{Signal, UniqueSortedHashMap, UniqueSortedVec, Wire};

// ************************************************************************************************
// enum
// ************************************************************************************************

#[derive(Clone)]
pub enum SignalTransformation {
    SignalReorder(UniqueSortedHashMap<Signal, Signal>),
    SignalsRemovedBecauseTheyAreNotUsed(UniqueSortedVec<Signal>),
    SignalsRemovedBecauseOfEquivalentWires(UniqueSortedHashMap<Signal, Wire>),
}

// ************************************************************************************************
// struct
// ************************************************************************************************

#[derive(Clone)]
/// Mapping from old signals to new wires, can be used to check which signals were removed and
/// what the new signals are.
///
/// This is used to track signal changes when manipulating a circuit.
pub struct SignalTracker {
    transformations: Vec<SignalTransformation>,
}

// ************************************************************************************************
// rust submodule declaration, they get searched in their respective file  names
// ************************************************************************************************

pub mod api;
pub mod construction;

// ************************************************************************************************
// re-exports of structs in these modules to simplify paths for other imports
// ************************************************************************************************
