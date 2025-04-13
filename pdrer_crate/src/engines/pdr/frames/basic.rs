// ************************************************************************************************
// use
// ************************************************************************************************

use super::{super::frame::Frame, Frames};
use crate::{
    engines::pdr::{
        definition_library::DefinitionLibrary, shared_objects::SharedObjects, solvers::Solvers,
        PropertyDirectedReachabilitySolver,
    },
    solvers::dd::DecisionDiagramManager,
};

// ************************************************************************************************
// impl
// ************************************************************************************************

/// Basic operations on frames, by design we disallow any mutable references to frames
/// to avoid any potential issues where a frame is modified in a way that is not expected.
impl<T: PropertyDirectedReachabilitySolver, D: DecisionDiagramManager> Frames<T, D> {
    // ********************************************************************************************
    // helper functions
    // ********************************************************************************************

    // ********************************************************************************************
    // API
    // ********************************************************************************************

    pub fn new(s: SharedObjects) -> Self {
        let solvers = Solvers::new(s.clone());
        let dr = DefinitionLibrary::new(s.clone(), solvers.max_variable_in_transition());
        let initial_frame = Frame::new();
        let inf_frame = Frame::new();
        let frame_hash_after_last_propagate = vec![initial_frame.get_hash(), inf_frame.get_hash()];
        Self {
            frames: vec![initial_frame, inf_frame],
            solvers,
            frame_hash_after_last_propagate,
            depth: 0,
            extension_variables_counter: 0,
            // lowest_frame_that_was_updated_since_last_condense: 1,
            lowest_frame_that_was_updated_since_last_propagate: 1,
            definition_library: dr,

            s,
        }
    }

    pub fn depth(&self) -> usize {
        self.depth
    }

    pub fn len(&self) -> usize {
        self.frames.len()
    }

    pub fn is_empty(&self) -> bool {
        self.frames.is_empty()
    }

    pub fn at(&self, i: usize) -> &Frame<D> {
        &self.frames[i]
    }

    pub fn iter(&self) -> impl DoubleEndedIterator<Item = &Frame<D>> + ExactSizeIterator {
        self.frames.iter()
    }

    pub(super) fn new_frame(&mut self) {
        debug_assert!(!self.frames.is_empty());
        // let is_first_frame = self.frames.len() == 1;
        let frame = Frame::new();

        let k = self.frames.len() - 1;
        self.frames.insert(k, frame);
        self.frame_hash_after_last_propagate
            .insert(k, self.frames[k].get_hash());
        self.solvers.new_solver(self.get_deltas());
        // self.add_all_definitions_to_frame(k);
    }

    pub fn increase_depth(&mut self) {
        self.depth += 1;
        if self.depth > (self.frames.len() - 2) {
            self.new_frame();
        }
    }

    /// This function gets the inf frame, which is the last frame in the list of frames.
    /// This frame is guaranteed to be inductive.
    pub fn get_f_inf(&self) -> &Frame<D> {
        self.frames.last().unwrap()
    }
}
