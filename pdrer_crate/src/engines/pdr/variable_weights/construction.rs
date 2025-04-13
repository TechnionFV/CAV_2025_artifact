// ************************************************************************************************
// use
// ************************************************************************************************

use super::{VariableWeight, VariableWeights, INITIAL_WEIGHT};
use crate::{
    formulas::Variable,
    models::{unique_sorted_hash_map::UniqueSortedHash, FiniteStateTransitionSystem},
};

// ************************************************************************************************
// impl
// ************************************************************************************************

impl VariableWeights {
    pub fn new(fin_state: &FiniteStateTransitionSystem, decay: VariableWeight) -> Self {
        let length_of_weights: usize = fin_state
            .get_state_variables()
            .max()
            .unwrap_or(&Variable::new(0))
            .hash()
            + 1;
        let literal_weights = vec![INITIAL_WEIGHT; length_of_weights];

        assert!(0.0 < decay);
        assert!(decay < 1.0);

        Self {
            weight_per_variable: literal_weights,
            decay,
            one_minus_decay: 1.0 - decay,
        }
    }
}
