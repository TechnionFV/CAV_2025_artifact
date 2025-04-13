// ************************************************************************************************
// use
// ************************************************************************************************

use crate::solvers::dd::DecisionDiagramManager;

use super::{
    definition_library::DefinitionLibrary, frame::Frame, shared_objects::SharedObjects,
    solvers::Solvers, PropertyDirectedReachabilitySolver,
};

// ************************************************************************************************
// struct
// ************************************************************************************************

#[derive(Debug)]
pub struct Frames<T: PropertyDirectedReachabilitySolver, D: DecisionDiagramManager> {
    /// The frames, first is INIT, last is F_INF
    frames: Vec<Frame<D>>,
    /// Solver manager
    solvers: Solvers<T>,
    /// The hash of the sat solver after the last time clauses from this frame were propagated
    frame_hash_after_last_propagate: Vec<usize>,
    /// The current depth of the algorithm
    depth: usize,
    /// the number of calls to the function that detects extension variables
    extension_variables_counter: usize,
    // lowest_frame_that_was_updated_since_last_condense: usize,
    lowest_frame_that_was_updated_since_last_propagate: usize,
    // length_of_f_inf_after_the_last_time_it_was_generalized: usize,
    definition_library: DefinitionLibrary<T, D>,

    s: SharedObjects,
}

// ************************************************************************************************
// rust submodule declaration, they get searched in their respective file  names
// ************************************************************************************************

pub mod basic;
pub mod check;
pub mod complex;
pub mod definitions;
pub mod extension_variables;
pub mod generalize;
pub mod generalize_with_ctg;
pub mod insert;
pub mod propagate;
pub mod propagate_f_inf;
pub mod sat_calls;

// ************************************************************************************************
// re-exports of structs in these modules to simplify paths for other imports
// ************************************************************************************************
