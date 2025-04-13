// ************************************************************************************************
// use
// ************************************************************************************************

use crate::models::Utils;

// ************************************************************************************************
// struct
// ************************************************************************************************

pub struct PowerSetIterator<T> {
    objects: Vec<T>,
    indexes: Vec<usize>,
    done: bool,
}

// ************************************************************************************************
// impl
// ************************************************************************************************

impl<T> PowerSetIterator<T> {
    pub fn new(objects: Vec<T>, subset_size: usize) -> Self {
        let indexes = (0..subset_size).collect();
        let done = subset_size > objects.len();
        Self {
            objects,
            indexes,
            done,
        }
    }
}

// ************************************************************************************************
// Iterator
// ************************************************************************************************

impl<T: Copy> Iterator for PowerSetIterator<T> {
    type Item = Vec<T>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.done {
            return None;
        } else if self.indexes.is_empty() {
            self.done = true;
            return Some(vec![]);
        }

        debug_assert!(Utils::is_sorted_and_unique(&self.indexes));
        let result = self.indexes.iter().map(|i| self.objects[*i]).collect();

        let mut index_to_move_forward = self.indexes.len() - 1;
        loop {
            let can_be_moved_forward = |index| {
                if index == self.indexes.len() - 1 {
                    self.indexes[index] != self.objects.len() - 1
                } else {
                    self.indexes[index] + 1 != self.indexes[index + 1]
                }
            };
            if can_be_moved_forward(index_to_move_forward) {
                self.indexes[index_to_move_forward] += 1;
                for i in index_to_move_forward + 1..self.indexes.len() {
                    self.indexes[i] = self.indexes[i - 1] + 1;
                }
                break;
            } else if index_to_move_forward == 0 {
                self.done = true;
                break;
            } else {
                index_to_move_forward -= 1;
            }
        }

        Some(result)
    }
}

// ************************************************************************************************
// tests
// ************************************************************************************************

#[test]
fn test_numbers() {
    let mut iter = PowerSetIterator::new(vec![1, 2, 3, 4, 5, 6], 0);
    assert_eq!(iter.next(), Some(vec![]));
    assert_eq!(iter.next(), None);

    let mut iter = PowerSetIterator::new(vec![1, 2, 3, 4, 5, 6], 1);
    assert_eq!(iter.next(), Some(vec![1]));
    assert_eq!(iter.next(), Some(vec![2]));
    assert_eq!(iter.next(), Some(vec![3]));
    assert_eq!(iter.next(), Some(vec![4]));
    assert_eq!(iter.next(), Some(vec![5]));
    assert_eq!(iter.next(), Some(vec![6]));
    assert_eq!(iter.next(), None);

    let mut iter = PowerSetIterator::new(vec![1, 2, 3, 4, 5, 6], 2);
    assert_eq!(iter.next(), Some(vec![1, 2]));
    assert_eq!(iter.next(), Some(vec![1, 3]));
    assert_eq!(iter.next(), Some(vec![1, 4]));
    assert_eq!(iter.next(), Some(vec![1, 5]));
    assert_eq!(iter.next(), Some(vec![1, 6]));
    assert_eq!(iter.next(), Some(vec![2, 3]));
    assert_eq!(iter.next(), Some(vec![2, 4]));
    assert_eq!(iter.next(), Some(vec![2, 5]));
    assert_eq!(iter.next(), Some(vec![2, 6]));
    assert_eq!(iter.next(), Some(vec![3, 4]));
    assert_eq!(iter.next(), Some(vec![3, 5]));
    assert_eq!(iter.next(), Some(vec![3, 6]));
    assert_eq!(iter.next(), Some(vec![4, 5]));
    assert_eq!(iter.next(), Some(vec![4, 6]));
    assert_eq!(iter.next(), Some(vec![5, 6]));
    assert_eq!(iter.next(), None);

    let mut iter = PowerSetIterator::new(vec![1, 2, 3, 4, 5, 6], 3);
    assert_eq!(iter.next(), Some(vec![1, 2, 3]));
    assert_eq!(iter.next(), Some(vec![1, 2, 4]));
    assert_eq!(iter.next(), Some(vec![1, 2, 5]));
    assert_eq!(iter.next(), Some(vec![1, 2, 6]));
    assert_eq!(iter.next(), Some(vec![1, 3, 4]));
    assert_eq!(iter.next(), Some(vec![1, 3, 5]));
    assert_eq!(iter.next(), Some(vec![1, 3, 6]));
    assert_eq!(iter.next(), Some(vec![1, 4, 5]));
    assert_eq!(iter.next(), Some(vec![1, 4, 6]));
    assert_eq!(iter.next(), Some(vec![1, 5, 6]));
    assert_eq!(iter.next(), Some(vec![2, 3, 4]));
    assert_eq!(iter.next(), Some(vec![2, 3, 5]));
    assert_eq!(iter.next(), Some(vec![2, 3, 6]));
    assert_eq!(iter.next(), Some(vec![2, 4, 5]));
    assert_eq!(iter.next(), Some(vec![2, 4, 6]));
    assert_eq!(iter.next(), Some(vec![2, 5, 6]));
    assert_eq!(iter.next(), Some(vec![3, 4, 5]));
    assert_eq!(iter.next(), Some(vec![3, 4, 6]));
    assert_eq!(iter.next(), Some(vec![3, 5, 6]));
    assert_eq!(iter.next(), Some(vec![4, 5, 6]));
    assert_eq!(iter.next(), None);

    let mut iter = PowerSetIterator::new(vec![1, 2, 3, 4, 5, 6], 4);
    assert_eq!(iter.next(), Some(vec![1, 2, 3, 4]));
    assert_eq!(iter.next(), Some(vec![1, 2, 3, 5]));
    assert_eq!(iter.next(), Some(vec![1, 2, 3, 6]));
    assert_eq!(iter.next(), Some(vec![1, 2, 4, 5]));
    assert_eq!(iter.next(), Some(vec![1, 2, 4, 6]));
    assert_eq!(iter.next(), Some(vec![1, 2, 5, 6]));
    assert_eq!(iter.next(), Some(vec![1, 3, 4, 5]));
    assert_eq!(iter.next(), Some(vec![1, 3, 4, 6]));
    assert_eq!(iter.next(), Some(vec![1, 3, 5, 6]));
    assert_eq!(iter.next(), Some(vec![1, 4, 5, 6]));
    assert_eq!(iter.next(), Some(vec![2, 3, 4, 5]));
    assert_eq!(iter.next(), Some(vec![2, 3, 4, 6]));
    assert_eq!(iter.next(), Some(vec![2, 3, 5, 6]));
    assert_eq!(iter.next(), Some(vec![2, 4, 5, 6]));
    assert_eq!(iter.next(), Some(vec![3, 4, 5, 6]));
    assert_eq!(iter.next(), None);

    let mut iter = PowerSetIterator::new(vec![1, 2, 3, 4, 5, 6], 5);
    assert_eq!(iter.next(), Some(vec![1, 2, 3, 4, 5]));
    assert_eq!(iter.next(), Some(vec![1, 2, 3, 4, 6]));
    assert_eq!(iter.next(), Some(vec![1, 2, 3, 5, 6]));
    assert_eq!(iter.next(), Some(vec![1, 2, 4, 5, 6]));
    assert_eq!(iter.next(), Some(vec![1, 3, 4, 5, 6]));
    assert_eq!(iter.next(), Some(vec![2, 3, 4, 5, 6]));
    assert_eq!(iter.next(), None);

    let mut iter = PowerSetIterator::new(vec![1, 2, 3, 4, 5, 6], 6);
    assert_eq!(iter.next(), Some(vec![1, 2, 3, 4, 5, 6]));
    assert_eq!(iter.next(), None);

    let mut iter = PowerSetIterator::new(vec![1, 2, 3, 4, 5, 6], 7);
    assert_eq!(iter.next(), None);
}
