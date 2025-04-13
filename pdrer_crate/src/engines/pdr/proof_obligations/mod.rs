// ************************************************************************************************
// use
// ************************************************************************************************

use self::{queue::ProofObligationsQueue, trace_tree::TraceTree};
use crate::formulas::Cube;

// ************************************************************************************************
// struct
// ************************************************************************************************

#[derive(Debug, Clone)]
pub struct ProofObligation {
    pub cube: Cube,
    pub frame: usize,
    pub hash_when_added: usize,
}

pub struct ProofObligations {
    queue: ProofObligationsQueue,
    trace_tree: TraceTree,
}

// ************************************************************************************************
// rust submodule declaration, they get searched in their respective file  names
// ************************************************************************************************

pub mod api;
pub mod queue;
pub mod trace_tree;

// ************************************************************************************************
// re-exports of structs in these modules to simplify paths for other imports
// ************************************************************************************************
