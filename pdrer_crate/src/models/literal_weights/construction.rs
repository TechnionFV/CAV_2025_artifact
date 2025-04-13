// ************************************************************************************************
// use
// ************************************************************************************************

use super::LiteralWeights;
use crate::models::FiniteStateTransitionSystem;

// ************************************************************************************************
// impl
// ************************************************************************************************

impl LiteralWeights {
    pub fn new(
        fin_state: &FiniteStateTransitionSystem,
        amount_to_add_when_literal_appears: usize,
        amount_to_subtract_when_literal_does_not_appear: usize,
    ) -> Self {
        // the array should contain all literal numbers
        let length_of_weights: usize = (fin_state.get_max_variable().number() as usize) << 1;
        let literal_weights = vec![0; length_of_weights + 2];

        // let state_variable_range = fin_state.get_state_variable_range();
        // let input_variable_range = fin_state.get_input_variable_range();

        Self {
            literal_weights,
            // state_variable_range,
            // input_variable_range,
            amount_to_add_when_literal_appears,
            amount_to_subtract_when_literal_does_not_appear,
        }
    }
}
