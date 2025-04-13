// ************************************************************************************************
// use
// ************************************************************************************************

use super::delta_element::DeltaElement;
use crate::{formulas::Clause, solvers::dd::DecisionDiagramManager};

// ************************************************************************************************
// struct
// ************************************************************************************************

#[derive(Debug)]
pub struct Frame<D: DecisionDiagramManager> {
    /// The unique clauses in this frame, these are unique because they are not present in later
    /// frames.
    delta: Vec<DeltaElement<D>>,
    /// A number that is guaranteed to be incremented when the frame changes in a way that would change
    /// the result of any SAT query on state variables.
    hash: usize,
    /// The fractions that were found to already have been propagated to the next frame.
    propagated_fractions: Vec<Clause>,
    // /// shared objects across pdr implementation
    // s: SharedObjects,
}

// ************************************************************************************************
// rust submodule declaration, they get searched in their respective file  names
// ************************************************************************************************

pub mod basic;

// ************************************************************************************************
// re-exports of structs in these modules to simplify paths for other imports
// ************************************************************************************************
