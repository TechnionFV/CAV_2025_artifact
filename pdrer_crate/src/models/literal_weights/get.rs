// ************************************************************************************************
// use
// ************************************************************************************************

use super::LiteralWeights;
use crate::{
    formulas::{Cube, Literal},
    models::unique_sorted_hash_map::UniqueSortedHash,
};

// ************************************************************************************************
// impl
// ************************************************************************************************

impl LiteralWeights {
    /// Returns a vector of literals sorted by their weights.
    /// From the lowest weight to the highest.
    pub fn get_cube_literals_sorted_by_weights(&self, cube: &Cube) -> Vec<Literal> {
        let mut literals: Vec<Literal> = cube.iter().copied().collect();
        self.sort_literals_by_weights_fast(&mut literals);
        literals
    }

    /// Orders the given vector by weights
    pub fn sort_literals_by_weights_fast(&self, literals: &mut [Literal]) {
        // sort literals
        // Note: this sort is stable because when using internal signals we want to lower indexes first.
        literals.sort_by_key(|l| self.get_weight(l));
    }

    pub fn sort_literals_by_literal_negation_weights_fast(&self, literals: &mut [Literal]) {
        // sort literals
        // Note: this sort is stable because when using internal signals we want to lower indexes first.
        literals.sort_by_key(|l| self.get_weight(&!l.to_owned()));
    }

    pub fn get_weight(&self, literal: &Literal) -> usize {
        let index = literal.hash();
        self.literal_weights[index]
    }
}
