// ************************************************************************************************
// use
// ************************************************************************************************

use std::{
    cmp::{min, Ordering},
    collections::BinaryHeap,
};

use super::UniqueSortedVec;

// ************************************************************************************************
// k_merge on vectors
// ************************************************************************************************

struct MergeCandidate<'a, T> {
    first: T,
    rest: &'a [T],
}

impl<'a, T: Copy> MergeCandidate<'a, T> {
    // Creates a new `MergeCandidate`, taking the last element from the reversed vector and keeping the rest.
    fn new(list: &'a [T]) -> Option<Self> {
        if list.is_empty() {
            return None;
        }

        Some(MergeCandidate {
            first: list[0],
            rest: &list[1..],
        })
    }

    fn next(&mut self) -> bool {
        if self.rest.is_empty() {
            return false;
        }

        self.first = self.rest[0];
        self.rest = &self.rest[1..];
        true
    }
}

impl<T: PartialEq> PartialEq for MergeCandidate<'_, T> {
    fn eq(&self, other: &Self) -> bool {
        self.first == other.first
    }
}

impl<T: Eq> Eq for MergeCandidate<'_, T> {}

impl<T: PartialOrd> PartialOrd for MergeCandidate<'_, T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        other.first.partial_cmp(&self.first) // Reverse for min-heap behavior
    }
}

impl<T: Ord> Ord for MergeCandidate<'_, T> {
    fn cmp(&self, other: &Self) -> Ordering {
        other.first.cmp(&self.first) // Reverse for min-heap behavior
    }
}

pub fn merge<'a, T: Ord + Copy + 'a, I>(lists: I, len: usize) -> Vec<T>
where
    I: IntoIterator<Item = &'a [T]>,
{
    let mut candidates: BinaryHeap<MergeCandidate<'_, T>> = BinaryHeap::with_capacity(len);
    let mut total_length = 0;

    // Reverse each list to enable popping from the end efficiently
    for list in lists {
        total_length += list.len();
        if let Some(candidate) = MergeCandidate::new(list) {
            candidates.push(candidate);
        }
    }

    let mut sorted = Vec::with_capacity(total_length);

    // Extract the minimum element and push the next from the same list until empty
    while let Some(mut merge_candidate) = candidates.pop() {
        // Only push the element if it's not a duplicate of the last pushed element
        if sorted.last() != Some(&merge_candidate.first) {
            sorted.push(merge_candidate.first);
        }

        // Pop the next element from the rest if it exists, then push it into the heap
        if merge_candidate.next() {
            candidates.push(merge_candidate);
        }
    }

    sorted
}

// ************************************************************************************************
// impl
// ************************************************************************************************

impl<V: Ord + Copy> UniqueSortedVec<V> {
    // ********************************************************************************************
    // Fast API
    // ********************************************************************************************

    pub fn subtract_custom(&self, other: &UniqueSortedVec<V>, limit: usize, result: &mut Vec<V>) {
        let mut cx_iter = self.usv.iter();
        let mut cy_iter = other.usv.iter();
        let mut cx = cx_iter.next();
        let mut cy = cy_iter.next();
        let mut count = 0;
        loop {
            if count == limit {
                break;
            }
            match (&cx, &cy) {
                (None, _) => {
                    break;
                }
                (Some(_), None) => {
                    count += 1;
                    result.push(*cx.unwrap());
                    cx = cx_iter.next();
                }
                (Some(x), Some(y)) => match x.cmp(y) {
                    std::cmp::Ordering::Less => {
                        count += 1;
                        result.push(*cx.unwrap());
                        cx = cx_iter.next();
                    }
                    std::cmp::Ordering::Equal => {
                        cx = cx_iter.next();
                        cy = cy_iter.next();
                    }
                    std::cmp::Ordering::Greater => {
                        cy = cy_iter.next();
                    }
                },
            }
        }
    }

    pub fn merge_custom(&self, other: &UniqueSortedVec<V>, limit: usize, result: &mut Vec<V>) {
        let mut cx_iter = self.usv.iter();
        let mut cy_iter = other.usv.iter();
        let mut cx = cx_iter.next();
        let mut cy = cy_iter.next();
        let mut count = 0;
        loop {
            if count == limit {
                break;
            }
            match (&cx, &cy) {
                (None, None) => {
                    break;
                }
                (None, Some(_)) => {
                    count += 1;
                    result.push(*cy.unwrap());
                    cy = cy_iter.next();
                }
                (Some(_), None) => {
                    count += 1;
                    result.push(*cx.unwrap());
                    cx = cx_iter.next();
                }
                (Some(x), Some(y)) => match x.cmp(y) {
                    std::cmp::Ordering::Less => {
                        count += 1;
                        result.push(*cx.unwrap());
                        cx = cx_iter.next();
                    }
                    std::cmp::Ordering::Equal => {
                        count += 1;
                        result.push(*cx.unwrap());
                        cx = cx_iter.next();
                        cy = cy_iter.next();
                    }
                    std::cmp::Ordering::Greater => {
                        count += 1;
                        result.push(*cy.unwrap());
                        cy = cy_iter.next();
                    }
                },
            }
        }
    }

    pub fn intersect_custom(&self, other: &UniqueSortedVec<V>, limit: usize, result: &mut Vec<V>) {
        let mut cx_iter = self.usv.iter();
        let mut cy_iter = other.usv.iter();
        let mut cx = cx_iter.next();
        let mut cy = cy_iter.next();
        let mut count = 0;
        loop {
            if count == limit {
                break;
            }
            match (&cx, &cy) {
                (None, None) | (None, Some(_)) | (Some(_), None) => {
                    break;
                }
                (Some(x), Some(y)) => match x.cmp(y) {
                    std::cmp::Ordering::Less => {
                        cx = cx_iter.next();
                    }
                    std::cmp::Ordering::Equal => {
                        count += 1;
                        result.push(*cx.unwrap());
                        cx = cx_iter.next();
                        cy = cy_iter.next();
                    }
                    std::cmp::Ordering::Greater => {
                        cy = cy_iter.next();
                    }
                },
            }
        }
    }

    pub fn symmetric_difference_custom(
        &self,
        other: &UniqueSortedVec<V>,
        limit_1: usize,
        limit_2: usize,
        result_1: &mut Vec<V>,
        result_2: &mut Vec<V>,
    ) {
        let mut cx_iter = self.usv.iter();
        let mut cy_iter = other.usv.iter();
        let mut cx = cx_iter.next();
        let mut cy = cy_iter.next();
        debug_assert!(result_1.is_empty());
        debug_assert!(result_2.is_empty());

        loop {
            if result_1.len() == limit_1 || result_2.len() == limit_2 {
                break;
            }
            match (&cx, &cy) {
                (None, None) => {
                    break;
                }
                (None, Some(_)) => {
                    result_2.push(*cy.unwrap());
                    cy = cy_iter.next();
                }
                (Some(_), None) => {
                    result_1.push(*cx.unwrap());
                    cx = cx_iter.next();
                }
                (Some(x), Some(y)) => match x.cmp(y) {
                    std::cmp::Ordering::Less => {
                        result_1.push(*cx.unwrap());
                        cx = cx_iter.next();
                    }
                    std::cmp::Ordering::Equal => {
                        cx = cx_iter.next();
                        cy = cy_iter.next();
                    }
                    std::cmp::Ordering::Greater => {
                        result_2.push(*cy.unwrap());
                        cy = cy_iter.next();
                    }
                },
            }
        }
    }

    // ********************************************************************************************
    // simple API
    // ********************************************************************************************

    pub fn merge(&self, other: &UniqueSortedVec<V>) -> UniqueSortedVec<V> {
        let mut usv: Vec<V> = Vec::with_capacity(self.usv.len() + other.usv.len());
        self.merge_custom(other, usize::MAX, &mut usv);
        UniqueSortedVec::from_ordered_set(usv)
    }

    pub fn subtract(&self, other: &UniqueSortedVec<V>) -> UniqueSortedVec<V> {
        let mut usv: Vec<V> = Vec::with_capacity(self.len());
        self.subtract_custom(other, usize::MAX, &mut usv);
        UniqueSortedVec::from_ordered_set(usv)
    }

    pub fn intersect(&self, other: &UniqueSortedVec<V>) -> UniqueSortedVec<V> {
        let mut usv: Vec<V> = Vec::with_capacity(min(self.usv.len(), other.usv.len()));
        self.intersect_custom(other, usize::MAX, &mut usv);
        UniqueSortedVec::from_ordered_set(usv)
    }

    pub fn symmetric_difference(
        &self,
        other: &UniqueSortedVec<V>,
    ) -> (UniqueSortedVec<V>, UniqueSortedVec<V>) {
        let mut usv_1: Vec<V> = Vec::with_capacity(self.len());
        let mut usv_2: Vec<V> = Vec::with_capacity(other.len());
        self.symmetric_difference_custom(other, usize::MAX, usize::MAX, &mut usv_1, &mut usv_2);
        (
            UniqueSortedVec::from_ordered_set(usv_1),
            UniqueSortedVec::from_ordered_set(usv_2),
        )
    }
}

impl<'a, V: Ord + Copy + 'a> UniqueSortedVec<V> {
    pub fn k_merge<I>(i: I, len: usize) -> UniqueSortedVec<V>
    where
        I: IntoIterator<Item = &'a UniqueSortedVec<V>>,
    {
        match len {
            0 => UniqueSortedVec::from_ordered_set(vec![]),
            1 => {
                let it = i.into_iter().next().unwrap();
                it.clone()
            }
            2 => {
                let mut it = i.into_iter();
                let a = it.next().unwrap();
                let b = it.next().unwrap();
                a.merge(b)
            }
            _ => {
                let mut result = merge(i.into_iter().map(|x| x.usv.as_slice()), len);
                result.shrink_to_fit();
                Self::from_ordered_set(result)
            }
        }
    }
}

// ************************************************************************************************
// unit test
// ************************************************************************************************

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_merge_without_consuming() {
        let a = UniqueSortedVec::from_sequence(vec![1, 2, 3, 4, 5]);
        let b = UniqueSortedVec::from_sequence(vec![3, 4, 5, 6, 7]);
        let c = a.merge(&b);
        assert_eq!(c, UniqueSortedVec::from_sequence(vec![1, 2, 3, 4, 5, 6, 7]));
    }

    #[test]
    fn test_subtract_without_consuming() {
        let a = UniqueSortedVec::from_sequence(vec![1, 2, 3, 4, 5]);
        let b = UniqueSortedVec::from_sequence(vec![3, 4, 5, 6, 7]);
        let c = a.subtract(&b);
        assert_eq!(c, UniqueSortedVec::from_sequence(vec![1, 2]));
    }

    #[test]
    fn test_intersect_without_consuming() {
        let a = UniqueSortedVec::from_sequence(vec![1, 2, 3, 4, 5]);
        let b = UniqueSortedVec::from_sequence(vec![3, 4, 5, 6, 7]);
        let c = a.intersect(&b);
        assert_eq!(c, UniqueSortedVec::from_sequence(vec![3, 4, 5]));
    }

    #[test]
    fn test_symmetric_difference_without_consuming() {
        let a = UniqueSortedVec::from_sequence(vec![1, 2, 3, 4, 5, 8, 9, 12]);
        let b = UniqueSortedVec::from_sequence(vec![3, 4, 5, 6, 7, 8, 10, 11]);
        let (c, d) = a.symmetric_difference(&b);
        assert_eq!(c, UniqueSortedVec::from_sequence(vec![1, 2, 9, 12]));
        assert_eq!(d, UniqueSortedVec::from_sequence(vec![6, 7, 10, 11]));
    }
}
