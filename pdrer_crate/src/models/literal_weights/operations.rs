// ************************************************************************************************
// use
// ************************************************************************************************

use super::LiteralWeights;

// ************************************************************************************************
// printing
// ************************************************************************************************

impl LiteralWeights {
    // ********************************************************************************************
    // helper functions
    // ********************************************************************************************

    // ********************************************************************************************
    // API
    // ********************************************************************************************

    /// subtract the weights of b from self.
    /// In other words perform the calculation self - b
    pub fn subtract(&mut self, b: &LiteralWeights) {
        // debug_assert!(self.state_variable_range == b.state_variable_range);
        // debug_assert!(self.input_variable_range == b.input_variable_range);
        debug_assert!(self.literal_weights.len() == b.literal_weights.len());

        for (i, weight) in b.literal_weights.iter().enumerate() {
            self.literal_weights[i] = self.literal_weights[i].saturating_sub(*weight);
        }
    }

    pub fn max_normalize(&mut self, multiplier: usize) {
        let max: usize = *self.literal_weights.iter().max().unwrap();
        if max == 0 {
            return;
        }
        for weight in self.literal_weights.iter_mut() {
            *weight = (*weight * multiplier) / max;
        }
    }
}
