// ************************************************************************************************
// use
// ************************************************************************************************

use super::Frame;
use crate::engines::pdr::delta_element::DeltaElement;
use crate::formulas::Clause;
use crate::models::{UniqueSortedVec, Utils};
use crate::solvers::dd::DecisionDiagramManager;

// use rand::Rng;

// ************************************************************************************************
// impl
// ************************************************************************************************

impl<D: DecisionDiagramManager> Frame<D> {
    // ********************************************************************************************
    // helper functions
    // ********************************************************************************************

    // ********************************************************************************************
    // construction API
    // ********************************************************************************************

    pub fn new() -> Self {
        Self {
            delta: Vec::new(),
            hash: 0,
            // s,
            propagated_fractions: Default::default(),
        }
    }

    // ********************************************************************************************
    // unique  clauses API
    // ********************************************************************************************

    /// Gets the clauses of the frame
    pub fn get_delta_clauses(&self) -> Vec<&Clause> {
        self.delta.iter().map(|de| de.clause()).collect()
    }

    pub fn get_delta_clauses_cloned(&self) -> Vec<Clause> {
        self.delta.iter().map(|de| de.clause().clone()).collect()
    }

    pub fn get_delta(&self) -> &Vec<DeltaElement<D>> {
        &self.delta
    }

    /// get unique clause by index
    pub fn get_delta_at(&self, i: usize) -> &DeltaElement<D> {
        &self.delta[i]
    }

    pub fn get_delta_at_mut(&mut self, i: usize) -> &mut DeltaElement<D> {
        &mut self.delta[i]
    }

    /// add unique clause, this is private because it should only be,
    /// called directly from outside the frame, this is because other
    /// frames should remove this clause as unique and add it as non unique.
    pub fn push_to_delta_and_increment_hash(&mut self, de: DeltaElement<D>) {
        self.push_to_delta(de);
        self.increment_hash();
    }

    /// This function is used to push a clause to the delta of the frame.
    /// DO NOT USE THIS FUNCTION TO ADD A CLAUSE TO THE FRAME, USE `push_to_delta_and_increment_hash` INSTEAD.
    pub fn push_to_delta(&mut self, clause: DeltaElement<D>) {
        self.delta.push(clause);
    }

    pub fn increment_hash(&mut self) {
        self.hash += 1;
    }

    /// remove unique clause by index
    pub fn remove_from_delta_at(&mut self, i: usize) -> DeltaElement<D> {
        self.delta.remove(i)
    }

    pub fn remove_multiple_from_delta(&mut self, indexes: &UniqueSortedVec<usize>) {
        Utils::remove_indexes(&mut self.delta, indexes);
    }

    // pub fn clear_unique_clauses(&mut self) {
    //     self.unique_clauses.clear();
    // }

    /// number of unique clauses in frame (size of delta)
    pub fn len(&self) -> usize {
        self.delta.len()
    }

    /// does frame contain a unique clause
    pub fn is_empty(&self) -> bool {
        self.delta.is_empty()
    }

    /// A hash of the frame that is used to check if the frame has changed.
    /// This hash is guaranteed to rise if the frame was changed in a way that
    /// would change the result of any SAT query.
    pub fn get_hash(&self) -> usize {
        self.hash
    }

    pub fn was_fraction_already_propagated(&self, c: &Clause) -> bool {
        self.propagated_fractions
            .iter()
            .any(|x| x.peek().peek().is_subset_of(c.peek().peek()))
    }

    pub fn mark_fraction_as_propagated(&mut self, c: Clause) {
        self.propagated_fractions.push(c);
    }
}

impl<D: DecisionDiagramManager> Default for Frame<D> {
    fn default() -> Self {
        Self::new()
    }
}
