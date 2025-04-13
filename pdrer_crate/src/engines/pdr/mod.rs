// ************************************************************************************************
// use
// ************************************************************************************************

use crate::solvers::dd::DecisionDiagramManager;

// ************************************************************************************************
// search results
// ************************************************************************************************

pub type Weights = VariableWeights;

// ************************************************************************************************
// struct
// ************************************************************************************************

#[derive(Debug)]
pub enum PropertyDirectedReachabilityError {
    ConstraintsNotSupported,
}

#[derive(Debug)]
pub enum PropertyDirectedReachabilityProofError {
    MaxDepthReached,
    TimeOutReached,
}

// ************************************************************************************************
// struct
// ************************************************************************************************
pub struct PropertyDirectedReachability<
    T: PropertyDirectedReachabilitySolver,
    D: DecisionDiagramManager,
> {
    /// The frames we hold at the moment starting from 0 up to the frame infinity
    pub frames: Frames<T, D>,
    /// proof obligations (cubes that need to be proven as unreachable in a certain frame)
    pub proof_obligations: ProofObligations,
    /// objects that are shared throughout the PDR algorithm
    pub s: SharedObjects,
}

// ************************************************************************************************
// rust submodule declaration, they get searched in their respective file  names
// ************************************************************************************************

pub mod block_cube;
pub mod construction;
pub mod definition_library;
pub mod delta_element;
pub mod frame;
pub mod frames;
pub mod parameters;
pub mod pdr_stats;
pub mod proof_obligations;
pub mod prove;
pub mod shared_objects;
pub mod solvers;
pub mod utils;
pub mod variable_weights;

// ************************************************************************************************
// re-exports of structs in these modules to simplify paths for other imports
// ************************************************************************************************

use self::frames::Frames;
pub use parameters::PropertyDirectedReachabilityParameters;
use proof_obligations::ProofObligations;
use shared_objects::SharedObjects;
pub use solvers::PropertyDirectedReachabilitySolver;
pub use variable_weights::VariableWeights;
