// ************************************************************************************************
// use
// ************************************************************************************************

use super::{VariableWeight, VariableWeights};
use crate::{
    formulas::{Cube, Literal, Variable},
    models::unique_sorted_hash_map::UniqueSortedHash,
};

// ************************************************************************************************
// impl
// ************************************************************************************************

impl VariableWeights {
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
        literals.sort_unstable_by(|a, b| {
            let w_a = self.get_weight(&a.variable());
            let w_b = self.get_weight(&b.variable());
            w_a.total_cmp(&w_b).then(a.cmp(b))
        });
    }

    // pub fn sort_literals_by_step(&self, literals: &mut [Literal]) {
    //     // sort literals
    //     // Note: this sort is stable because when using internal signals we want to lower indexes first.
    //     literals.sort_by_key(|l| self.get_weight(&l.variable()) > 0);
    // }

    pub fn get_weight(&self, variable: &Variable) -> VariableWeight {
        let index = variable.hash();
        self.weight_per_variable[index]
    }
}
