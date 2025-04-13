// ************************************************************************************************
// use
// ************************************************************************************************

use super::UniqueSortedVec;
use crate::models::Utils;

// ************************************************************************************************
// impl
// ************************************************************************************************

impl<V: Ord> UniqueSortedVec<V> {
    pub fn new() -> Self {
        Self { usv: Vec::new() }
    }

    pub fn from_ordered_set(usv: Vec<V>) -> Self {
        debug_assert!(Utils::is_sorted_and_unique(&usv));
        Self { usv }
    }

    pub fn from_sequence(mut usv: Vec<V>) -> Self {
        usv.sort_unstable();
        usv.dedup();
        Self { usv }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            usv: Vec::with_capacity(capacity),
        }
    }
}
