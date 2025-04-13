// ************************************************************************************************
// use
// ************************************************************************************************

use super::Frames;
use crate::engines::pdr::definition_library::DefinitionLibrary;
use crate::engines::pdr::PropertyDirectedReachabilitySolver;
// use crate::formulas::Clause;
use crate::models::definition::Definition;
use crate::solvers::dd::DecisionDiagramManager;

// ************************************************************************************************
// impl
// ************************************************************************************************

impl<T: PropertyDirectedReachabilitySolver, D: DecisionDiagramManager> Frames<T, D> {
    // ********************************************************************************************
    // helper functions
    // ********************************************************************************************

    // ********************************************************************************************
    // API to only be using in frames
    // ********************************************************************************************

    // pub(super) fn add_all_definitions_to_frame(&mut self, k: usize) {
    //     debug_assert!(self.frames[k].get_delta_clauses().is_empty());
    //     for d in self.definition_library.iter() {
    //         self.frames[k].add_new_definition(d);
    //     }
    // }

    // pub(super) fn add_definition_to_all_frames(frames: &mut [Frame<D>], d: &Definition) {
    //     for frame in frames.iter_mut() {
    //         frame.add_new_definition(d);
    //     }
    // }

    // ********************************************************************************************
    // Definition API
    // ********************************************************************************************

    pub fn get_definitions(&self) -> &Vec<Definition> {
        self.definition_library.get_definitions()
    }

    pub fn get_definitions_lib(&self) -> &DefinitionLibrary<T, D> {
        &self.definition_library
    }
}
