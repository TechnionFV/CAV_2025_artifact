// ************************************************************************************************
// use
// ************************************************************************************************

use super::{VariableWeight, VariableWeights, INITIAL_WEIGHT};
use crate::{formulas::Literal, models::unique_sorted_hash_map::UniqueSortedHash};

// ************************************************************************************************
// impl
// ************************************************************************************************

impl VariableWeights {
    fn decay(weight: &mut VariableWeight, decay: VariableWeight) {
        *weight = ((*weight) * decay) as VariableWeight;
    }

    // ********************************************************************************************
    // helper functions - using weights to know which literals are most important
    // ********************************************************************************************

    pub fn update_weights_on_add<'a, I>(&mut self, literals: I)
    where
        I: IntoIterator<Item = &'a Literal>,
    {
        let mut cx_iter = self.weight_per_variable.iter_mut().enumerate();
        let mut cy_iter = literals.into_iter().map(|l| l.variable()).map(|v| v.hash());
        let mut cx = cx_iter.next();
        let mut cy = cy_iter.next();
        loop {
            match (cx, &cy) {
                (None, None) => {
                    break;
                }
                (None, Some(_)) => {
                    // variable does not exist
                    unreachable!();
                }
                (Some((_, weight)), None) => {
                    Self::decay(weight, self.decay);
                    cx = cx_iter.next();
                }
                (Some((x, weight)), Some(y)) => match x.cmp(y) {
                    std::cmp::Ordering::Less => {
                        Self::decay(weight, self.decay);
                        cx = cx_iter.next();
                    }
                    std::cmp::Ordering::Equal => {
                        Self::decay(weight, self.decay);
                        (*weight) += self.one_minus_decay;
                        cx = cx_iter.next();
                        cy = cy_iter.next();
                    }
                    std::cmp::Ordering::Greater => {
                        // variable does not exist
                        unreachable!();
                        // cy = cy_iter.next();
                    }
                },
            }
        }
    }

    pub fn clear(&mut self) {
        self.weight_per_variable.fill(INITIAL_WEIGHT);
    }
}
