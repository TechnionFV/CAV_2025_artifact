// ************************************************************************************************
// use
// ************************************************************************************************

use super::UniqueSortedVec;

use std::cmp::Ordering;
use std::collections::BinaryHeap;
use std::vec::Vec;

// ************************************************************************************************
// k_merge on vectors
// ************************************************************************************************

struct MergeCandidate<T> {
    first: T,
    rest: Vec<T>,
}

impl<T> MergeCandidate<T> {
    // Creates a new `MergeCandidate`, taking the last element from the reversed vector and keeping the rest.
    fn new(mut list: Vec<T>) -> Option<Self> {
        list.pop().map(|first| MergeCandidate { first, rest: list })
    }
}

impl<T: PartialEq> PartialEq for MergeCandidate<T> {
    fn eq(&self, other: &Self) -> bool {
        self.first == other.first
    }
}

impl<T: Eq> Eq for MergeCandidate<T> {}

impl<T: PartialOrd> PartialOrd for MergeCandidate<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        other.first.partial_cmp(&self.first) // Reverse for min-heap behavior
    }
}

impl<T: Ord> Ord for MergeCandidate<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        other.first.cmp(&self.first) // Reverse for min-heap behavior
    }
}

pub fn merge<T: Ord>(lists: Vec<Vec<T>>) -> Vec<T> {
    let mut candidates = BinaryHeap::with_capacity(lists.len());
    let mut total_length = 0;

    // Reverse each list to enable popping from the end efficiently
    for mut list in lists {
        list.reverse();
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
        if let Some(next_first) = merge_candidate.rest.pop() {
            candidates.push(MergeCandidate {
                first: next_first,
                rest: merge_candidate.rest,
            });
        }
    }

    sorted
}
// ************************************************************************************************
// impl
// ************************************************************************************************

impl<V: Ord> UniqueSortedVec<V> {
    pub fn merge_consuming(self, other: UniqueSortedVec<V>) -> UniqueSortedVec<V> {
        let mut usv = Vec::with_capacity(self.usv.len() + other.usv.len());
        let mut cx_iter = self.usv.into_iter();
        let mut cy_iter = other.usv.into_iter();
        let mut cx = cx_iter.next();
        let mut cy = cy_iter.next();
        loop {
            match (&cx, &cy) {
                (None, None) => {
                    break;
                }
                (None, Some(_)) => {
                    usv.push(cy.unwrap());
                    cy = cy_iter.next();
                }
                (Some(_), None) => {
                    usv.push(cx.unwrap());
                    cx = cx_iter.next();
                }
                (Some(x), Some(y)) => match x.cmp(y) {
                    std::cmp::Ordering::Less => {
                        usv.push(cx.unwrap());
                        cx = cx_iter.next();
                    }
                    std::cmp::Ordering::Equal => {
                        usv.push(cx.unwrap());
                        cx = cx_iter.next();
                        cy = cy_iter.next();
                    }
                    std::cmp::Ordering::Greater => {
                        usv.push(cy.unwrap());
                        cy = cy_iter.next();
                    }
                },
            }
        }
        Self::from_ordered_set(usv)
    }

    pub fn k_merge_consuming_old(mut vectors: Vec<UniqueSortedVec<V>>) -> UniqueSortedVec<V> {
        loop {
            // stop condition
            if vectors.is_empty() {
                return UniqueSortedVec::from_ordered_set(vec![]);
            } else if vectors.len() == 1 {
                // debug_assert!(Self::is_sorted_and_unique(&vectors[0]));
                return vectors.remove(0);
            }

            // make new vectors vector
            let mut index = 0;
            while index < vectors.len() {
                if index + 1 < vectors.len() {
                    vectors[index] = std::mem::take(&mut vectors[index])
                        .merge_consuming(std::mem::take(&mut vectors[index + 1]));
                    // std::mem::take(&mut vectors[index]),
                    // std::mem::take(&mut vectors[index + 1]),
                    // );
                }

                index += 2;
            }

            // remove empty vectors
            vectors.retain(|v| !v.is_empty());
        }
    }

    pub fn k_merge_consuming(mut vectors: Vec<UniqueSortedVec<V>>) -> UniqueSortedVec<V> {
        match vectors.len() {
            0 => UniqueSortedVec::from_ordered_set(vec![]),
            1 => std::mem::take(&mut vectors[0]),
            2 => std::mem::take(&mut vectors[0]).merge_consuming(std::mem::take(&mut vectors[1])),
            _ => {
                let mut result = merge(vectors.into_iter().map(|v| v.usv).collect());
                result.shrink_to_fit();
                Self::from_ordered_set(result)
            }
        }
    }

    pub fn subtract_consuming(self, other: UniqueSortedVec<V>) -> UniqueSortedVec<V> {
        let mut usv = Vec::with_capacity(self.usv.len());
        let mut cx_iter = self.usv.into_iter();
        let mut cy_iter = other.usv.into_iter();
        let mut cx = cx_iter.next();
        let mut cy = cy_iter.next();
        loop {
            match (&cx, &cy) {
                (None, _) => {
                    break;
                }
                (Some(_), None) => {
                    usv.push(cx.unwrap());
                    cx = cx_iter.next();
                }
                (Some(x), Some(y)) => match x.cmp(y) {
                    std::cmp::Ordering::Less => {
                        usv.push(cx.unwrap());
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
        Self::from_ordered_set(usv)
    }

    pub fn intersect_consuming(self, other: UniqueSortedVec<V>) -> UniqueSortedVec<V> {
        let mut usv = Vec::with_capacity(self.usv.len());
        let mut cx_iter = self.usv.into_iter();
        let mut cy_iter = other.usv.into_iter();
        let mut cx = cx_iter.next();
        let mut cy = cy_iter.next();
        loop {
            match (&cx, &cy) {
                (None, None) | (None, Some(_)) | (Some(_), None) => {
                    break;
                }
                (Some(x), Some(y)) => match x.cmp(y) {
                    std::cmp::Ordering::Less => {
                        cx = cx_iter.next();
                    }
                    std::cmp::Ordering::Equal => {
                        usv.push(cx.unwrap());
                        cx = cx_iter.next();
                        cy = cy_iter.next();
                    }
                    std::cmp::Ordering::Greater => {
                        cy = cy_iter.next();
                    }
                },
            }
        }
        Self::from_ordered_set(usv)
    }
}

// ************************************************************************************************
// test
// ************************************************************************************************

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_merge() {
        let a = UniqueSortedVec::from_sequence(vec![1, 2, 3, 4, 5]);
        let b = UniqueSortedVec::from_sequence(vec![3, 4, 5, 6, 7]);
        assert_eq!(
            a.merge_consuming(b),
            UniqueSortedVec::from_sequence(vec![1, 2, 3, 3, 4, 4, 5, 5, 6, 7])
        );
    }

    #[test]
    fn test_k_merge() {
        let a = UniqueSortedVec::from_sequence(vec![1, 2, 3, 4, 5]);
        let b = UniqueSortedVec::from_sequence(vec![3, 4, 5, 6, 7]);
        let c = UniqueSortedVec::from_sequence(vec![5, 6, 7, 8, 9]);
        let d = UniqueSortedVec::from_sequence(vec![7, 8, 9, 10, 11]);
        let e = UniqueSortedVec::from_sequence(vec![9, 10, 11, 12, 13]);
        let f = UniqueSortedVec::from_sequence(vec![11, 12, 13, 14, 15]);
        let g = UniqueSortedVec::from_sequence(vec![13, 14, 15, 16, 17]);
        assert_eq!(
            UniqueSortedVec::k_merge_consuming(vec![a, b, c, d, e, f, g]),
            UniqueSortedVec::from_sequence(vec![
                1, 2, 3, 3, 4, 4, 5, 5, 5, 6, 6, 7, 7, 7, 8, 9, 9, 9, 10, 11, 11, 11, 12, 13, 13,
                13, 14, 15, 15, 16, 17
            ])
        );
    }

    #[test]
    fn test_subtract() {
        let a = UniqueSortedVec::from_sequence(vec![2, 3, 5, 8, 13, 21]);
        let b = UniqueSortedVec::from_sequence(vec![8, 9, 10, 11, 12, 13, 14, 15]);
        assert_eq!(
            a.subtract_consuming(b),
            UniqueSortedVec::from_sequence(vec![2, 3, 5, 21])
        );
    }

    #[test]
    fn test_intersect() {
        let a = UniqueSortedVec::from_sequence(vec![2, 3, 5, 8, 13, 21]);
        let b = UniqueSortedVec::from_sequence(vec![8, 9, 10, 11, 12, 13, 14, 15]);
        assert_eq!(
            a.intersect_consuming(b),
            UniqueSortedVec::from_sequence(vec![8, 13])
        );
    }
}
