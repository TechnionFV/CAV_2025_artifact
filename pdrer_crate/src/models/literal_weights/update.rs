// ************************************************************************************************
// use
// ************************************************************************************************

use super::LiteralWeights;
use crate::{formulas::Literal, models::unique_sorted_hash_map::UniqueSortedHash};

// ************************************************************************************************
// impl
// ************************************************************************************************

impl LiteralWeights {
    // ********************************************************************************************
    // helper functions - using weights to know which literals are most important
    // ********************************************************************************************

    pub fn update_weights_on_add<'a, I>(&mut self, literals: I)
    where
        I: IntoIterator<Item = &'a Literal>,
    {
        for l in literals {
            let index = l.hash();

            self.literal_weights[index] += self.amount_to_add_when_literal_appears
                + self.amount_to_subtract_when_literal_does_not_appear;
        }

        for x in self.literal_weights.iter_mut() {
            let r = (*x).checked_sub(self.amount_to_subtract_when_literal_does_not_appear);
            if let Some(y) = r {
                *x = y;
            } else {
                *x = 0;
            }
        }
    }

    pub fn clear(&mut self) {
        self.literal_weights.fill(0);
    }
}
