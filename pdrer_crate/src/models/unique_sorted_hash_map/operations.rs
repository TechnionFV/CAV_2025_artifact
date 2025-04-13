// ************************************************************************************************
// use
// ************************************************************************************************

use std::fmt::Display;

use crate::models::PrettyTable;

use super::{UniqueSortedHash, UniqueSortedHashMap};

// ************************************************************************************************
// impl
// ************************************************************************************************

impl<K: UniqueSortedHash, V> UniqueSortedHashMap<K, V> {
    // ********************************************************************************************
    // helper functions
    // ********************************************************************************************

    fn index(&self, key: &K) -> Option<usize> {
        key.hash().checked_sub(self.lowest_key_hash)
    }

    fn key(&self, index: usize) -> K {
        K::un_hash(index + self.lowest_key_hash)
    }

    // ********************************************************************************************
    // API
    // ********************************************************************************************

    pub fn get(&self, key: &K) -> Option<&V> {
        let index = self.index(key)?;
        match self.vector.get(index) {
            None => None,       // out of bounds
            Some(None) => None, // in bounds but doesn't exist
            Some(Some(value)) => Some(value),
        }
    }

    pub fn get_mut(&mut self, key: &K) -> Option<&mut V> {
        let index = self.index(key)?;
        match self.vector.get_mut(index) {
            None => None,       // out of bounds
            Some(None) => None, // in bounds but doesn't exist
            Some(Some(value)) => Some(value),
        }
    }

    pub fn get_mut_or_add<F: Fn() -> V>(&mut self, key: &K, val: F) -> Option<&mut V> {
        let index = self.index(key)?;
        match self.vector.get_mut(index) {
            None => None, // out of bounds
            Some(x) => match x {
                None => {
                    *x = Some(val());
                    self.length += 1;
                    x.as_mut()
                }
                Some(value) => Some(value),
            }, // in bounds but doesn't exist
        }
    }

    /// Inserts a key-value pair into the map.
    /// panics if out of bounds or if a value already exists for this signal.
    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        let index = self.index(&key).unwrap();
        match self.vector.get_mut(index) {
            None => None,
            // Some(Some(x)) => None,
            Some(x) => {
                let old_value = (*x).replace(value);
                if old_value.is_none() {
                    self.length += 1;
                }
                old_value
            }
        }
    }

    pub fn remove(&mut self, key: &K) -> Option<V> {
        let index = self.index(key)?;
        match self.vector.get_mut(index) {
            None => None,
            Some(x) => {
                let old = (*x).take();
                if old.is_some() {
                    self.length -= 1;
                }
                old
            }
        }
    }

    pub fn max_key(&self) -> Option<K> {
        self.vector.iter().enumerate().rev().find_map(|(i, x)| {
            if x.is_some() {
                Some(K::un_hash(i + self.lowest_key_hash))
            } else {
                None
            }
        })
    }

    pub fn max_possible_key(&self) -> &K {
        &self.highest_key
    }

    /// iterate over map in order of keys
    pub fn iter_sorted(&self) -> impl Iterator<Item = K> + '_ {
        self.vector
            .iter()
            .enumerate()
            .filter(|(_, v)| v.is_some())
            .map(|(i, _)| self.key(i))
    }

    // pub fn iter_sorted_key_and_value(&self) -> impl Iterator<Item = (&K, &V)> + '_ {
    //     self.index
    //         .as_ref()
    //         .unwrap()
    //         .iter()
    //         .zip(self.vector.iter().filter_map(|x| x.as_ref()))
    // }

    pub fn iter_items(&self) -> impl Iterator<Item = &V> + '_ {
        self.vector.iter().filter_map(|x| x.as_ref())
    }

    pub fn iter_pairs(&self) -> impl DoubleEndedIterator<Item = (K, &V)> + '_ {
        self.vector
            .iter()
            .enumerate()
            .filter_map(|(i, x)| x.as_ref().map(|v| (self.key(i), v)))
    }

    pub fn contains_key(&self, key: &K) -> bool {
        self.get(key).is_some()
    }

    pub fn len(&self) -> usize {
        self.length
    }

    pub fn is_empty(&self) -> bool {
        self.length == 0
    }

    pub fn resize_with(&mut self, highest_key: K) {
        let highest_key_hash = highest_key.hash();
        // if self.highest_key.hash() >= highest_key_hash {
        //     return;
        // }
        let new_length = highest_key_hash.saturating_sub(self.lowest_key_hash) + 1;
        self.vector.resize_with(new_length, || None);
        self.highest_key = highest_key;
    }
}

impl<K: UniqueSortedHash + Display, V: Display> Display for UniqueSortedHashMap<K, V> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut table = PrettyTable::new(vec!["Key".to_string(), "Value".to_string()]);
        for (k, v) in self.iter_pairs() {
            table.add_row(vec![k.to_string(), v.to_string()]).unwrap();
        }
        write!(f, "{}", table)
    }
}

// ************************************************************************************************
// tests
// ************************************************************************************************

#[test]
fn test() {
    use crate::formulas::Variable;

    let v0 = Variable::new(0);
    let v1 = Variable::new(1);
    let v2 = Variable::new(2);
    let v3 = Variable::new(3);
    let v4 = Variable::new(4);
    let v5 = Variable::new(5);
    let v6 = Variable::new(6);
    let v7 = Variable::new(7);

    let mut map: UniqueSortedHashMap<Variable, i32> = UniqueSortedHashMap::new_ranged(v6, v1);

    // test insert
    // assert!(map.insert(v0, 0).is_none());
    assert!(map.insert(v1, 1).is_none());
    assert!(map.insert(v6, 6).is_none());
    assert!(map.insert(v2, 2).is_none());
    assert!(map.insert(v5, 5).is_none());
    assert!(map.insert(v3, 3).is_none());
    assert!(map.insert(v4, 4).is_none());

    // test get
    assert_eq!(map.get(&v0), None);
    assert_eq!(map.get(&v1), Some(&1));
    assert_eq!(map.get(&v6), Some(&6));
    assert_eq!(map.get(&v2), Some(&2));
    assert_eq!(map.get(&v5), Some(&5));
    assert_eq!(map.get(&v3), Some(&3));
    assert_eq!(map.get(&v4), Some(&4));
    assert_eq!(map.get(&v7), None);

    // test get_mut
    assert_eq!(map.get_mut(&v0), None);
    assert_eq!(map.get_mut(&v1), Some(&mut 1));
    assert_eq!(map.get_mut(&v6), Some(&mut 6));
    assert_eq!(map.get_mut(&v2), Some(&mut 2));
    assert_eq!(map.get_mut(&v5), Some(&mut 5));
    assert_eq!(map.get_mut(&v3), Some(&mut 3));
    assert_eq!(map.get_mut(&v4), Some(&mut 4));
    assert_eq!(map.get_mut(&v7), None);

    // test remove
    assert!(map.remove(&v0).is_none());
    assert!(map.remove(&v1).is_some());
    assert!(map.remove(&v6).is_some());
    assert!(map.remove(&v2).is_some());
    assert!(map.remove(&v5).is_some());
    assert!(map.remove(&v3).is_some());
    assert!(map.remove(&v4).is_some());
    assert!(map.remove(&v7).is_none());

    // test max_key and min_key
    assert_eq!(map.max_key(), None);
    assert!(map.insert(v1, 1).is_none());
    assert_eq!(map.max_key(), Some(v1));
    assert!(map.insert(v6, 6).is_none());
    assert_eq!(map.max_key(), Some(v6));
    assert!(map.insert(v2, 2).is_none());
    assert_eq!(map.max_key(), Some(v6));
    assert!(map.insert(v5, 5).is_none());
    assert_eq!(map.max_key(), Some(v6));
    assert!(map.insert(v3, 3).is_none());
    assert_eq!(map.max_key(), Some(v6));
    assert!(map.insert(v4, 4).is_none());
    assert_eq!(map.max_key(), Some(v6));
    // assert!(map.insert(v7, 7).is_none());
    // assert_eq!(map.max_key(), Some(v6));

    // test iter_sorted
    {
        let mut iter = map.iter_sorted();
        assert_eq!(iter.next(), Some(v1));
        assert_eq!(iter.next(), Some(v2));
        assert_eq!(iter.next(), Some(v3));
        assert_eq!(iter.next(), Some(v4));
        assert_eq!(iter.next(), Some(v5));
        assert_eq!(iter.next(), Some(v6));
        assert_eq!(iter.next(), None);
    }

    // test iter_items
    {
        let mut iter = map.iter_items();
        assert_eq!(iter.next(), Some(&1));
        assert_eq!(iter.next(), Some(&2));
        assert_eq!(iter.next(), Some(&3));
        assert_eq!(iter.next(), Some(&4));
        assert_eq!(iter.next(), Some(&5));
        assert_eq!(iter.next(), Some(&6));
        assert_eq!(iter.next(), None);
    }

    // test contains_key
    assert!(!map.contains_key(&v0));
    assert!(map.contains_key(&v1));
    assert!(map.contains_key(&v6));
    assert!(map.contains_key(&v2));
    assert!(map.contains_key(&v5));
    assert!(map.contains_key(&v3));
    assert!(map.contains_key(&v4));
    assert!(!map.contains_key(&v7));

    // test len
    assert_eq!(map.len(), 6);
    assert!(map.remove(&v1).is_some());
    assert_eq!(map.len(), 5);
    assert!(map.remove(&v6).is_some());
    assert_eq!(map.len(), 4);
    assert!(map.remove(&v2).is_some());
    assert_eq!(map.len(), 3);
    assert!(map.remove(&v5).is_some());
    assert_eq!(map.len(), 2);
    assert!(map.remove(&v3).is_some());
    assert_eq!(map.len(), 1);
    assert!(map.remove(&v4).is_some());
    assert_eq!(map.len(), 0);
    assert!(map.remove(&v7).is_none());
    assert_eq!(map.len(), 0);
    assert!(map.insert(v1, 1).is_none());
    assert_eq!(map.len(), 1);
    assert!(map.insert(v6, 6).is_none());
    assert_eq!(map.len(), 2);
    assert!(map.insert(v2, 2).is_none());
    assert_eq!(map.len(), 3);
    assert!(map.insert(v5, 5).is_none());
    assert_eq!(map.len(), 4);
    assert!(map.insert(v3, 3).is_none());
    assert_eq!(map.len(), 5);
    assert!(map.insert(v4, 4).is_none());
    assert_eq!(map.len(), 6);

    // test is_empty
    assert!(!map.is_empty());
    assert!(map.remove(&v1).is_some());
    assert!(!map.is_empty());
    assert!(map.remove(&v6).is_some());
    assert!(!map.is_empty());
    assert!(map.remove(&v2).is_some());
    assert!(!map.is_empty());
    assert!(map.remove(&v5).is_some());
    assert!(!map.is_empty());
    assert!(map.remove(&v3).is_some());
    assert!(!map.is_empty());
    assert!(map.remove(&v4).is_some());
    assert!(map.is_empty());
    assert!(map.remove(&v7).is_none());
    assert!(map.is_empty());
    assert!(map.insert(v1, 1).is_none());
    assert!(!map.is_empty());
}
