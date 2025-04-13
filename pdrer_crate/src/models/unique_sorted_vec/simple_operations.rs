// ************************************************************************************************
// use
// ************************************************************************************************

use std::cmp::min;

use crate::models::Utils;

use super::UniqueSortedVec;

// ************************************************************************************************
// impl
// ************************************************************************************************

impl<V: Ord> UniqueSortedVec<V> {
    pub fn push(&mut self, value: V) {
        debug_assert!(
            if let Some(last) = self.usv.last() {
                last < &value
            } else {
                true
            },
            "Are equal: {}",
            self.usv.last() == Some(&value)
        );
        self.usv.push(value);
    }

    /// Adds a value to the vector.
    ///
    /// Returns whether the value was newly inserted. That is:
    ///
    /// - If the vector did not previously contain this value, `true` is returned.
    /// - If the vector already contained this value, `false` is returned,
    ///   and the vector is not modified: original value is not replaced,
    ///   and the value passed as argument is dropped.
    ///
    pub fn insert(&mut self, value: V) -> bool {
        let index = self.usv.binary_search(&value);
        match index {
            Ok(_) => false,
            Err(i) => {
                self.usv.insert(i, value);
                debug_assert!({
                    let k0 = i.saturating_sub(1);
                    let k1 = min(
                        self.usv.len() - 1,
                        i.checked_add(1).unwrap_or(self.usv.len() - 1),
                    );
                    Utils::is_sorted_and_unique(&self.usv[k0..=k1])
                });
                true
            }
        }
    }

    /// Removes a value from the vector. Returns whether the value was
    /// present in the vector.
    pub fn remove(&mut self, value: &V) -> bool {
        let index = self.find(value);
        if let Some(i) = index {
            self.usv.remove(i);
            true
        } else {
            false
        }
    }

    pub fn remove_index(&mut self, index: usize) -> V {
        self.usv.remove(index)
    }

    pub fn find(&self, value: &V) -> Option<usize> {
        self.usv.binary_search(value).ok()
        // self.usv.binary_search(value)
    }

    pub fn at(&self, index: usize) -> &V {
        &self.usv[index]
    }

    pub fn len(&self) -> usize {
        self.usv.len()
    }

    pub fn is_empty(&self) -> bool {
        self.usv.is_empty()
    }

    pub fn iter(&self) -> impl DoubleEndedIterator<Item = &V> + ExactSizeIterator {
        self.usv.iter()
    }

    pub fn max(&self) -> Option<&V> {
        self.usv.last()
    }

    pub fn min(&self) -> Option<&V> {
        self.usv.first()
    }

    pub fn contains(&self, value: &V) -> bool {
        self.usv.binary_search(value).is_ok()
    }

    pub fn perform_operation_on_each_value<F>(&mut self, f: F)
    where
        F: FnMut(&mut V),
    {
        self.usv.iter_mut().for_each(f);
        debug_assert!(Utils::is_sorted_and_unique(&self.usv));
    }

    pub fn perform_disordering_operation_on_each_value<F>(&mut self, f: F)
    where
        F: FnMut(&mut V),
    {
        self.usv.iter_mut().for_each(f);
        self.usv.sort_unstable();
        self.usv.dedup();
    }

    pub fn unpack(self) -> Vec<V> {
        self.usv
    }

    pub fn peek(&self) -> &Vec<V> {
        &self.usv
    }

    /// This function is problematic as it can break the invariant that elements are sorted and unique.
    pub fn peek_mut(&mut self) -> &mut Vec<V> {
        &mut self.usv
    }

    pub fn clear(&mut self) {
        self.usv.clear();
    }

    pub fn extend<I: IntoIterator<Item = V>>(&mut self, other: I) {
        self.usv.extend(other);
        self.usv.sort_unstable();
        self.usv.dedup();
    }

    pub fn retain<F>(&mut self, f: F)
    where
        F: FnMut(&V) -> bool,
    {
        self.usv.retain(f);
    }

    pub fn position(&self, value: &V) -> Option<usize> {
        self.usv.binary_search(value).ok()
    }

    pub fn shrink_to_fit(&mut self) {
        self.usv.shrink_to_fit();
    }
}
