// ************************************************************************************************
// use
// ************************************************************************************************

use super::{UniqueSortedHash, UniqueSortedHashMap};

// ************************************************************************************************
// impl
// ************************************************************************************************

impl<K: UniqueSortedHash, V: Clone> UniqueSortedHashMap<K, V> {
    // ********************************************************************************************
    // API
    // ********************************************************************************************

    /// Creates a map with the given capacity.
    pub fn new(highest_key: K) -> Self {
        let lowest_key_hash = 0;
        let len = highest_key.hash() + 1 - lowest_key_hash;
        Self {
            vector: vec![None; len],
            length: 0,
            lowest_key_hash,
            highest_key,
        }
    }

    pub fn new_ranged(highest_key: K, lowest_key: K) -> Self {
        let lowest_key_hash = lowest_key.hash();
        let len = highest_key.hash() + 1 - lowest_key_hash;
        Self {
            vector: vec![None; len],
            length: 0,
            lowest_key_hash,
            highest_key,
        }
    }

    pub fn new_like<O>(map: &UniqueSortedHashMap<K, O>) -> Self {
        if map.lowest_key_hash == 0 {
            Self::new(K::un_hash(map.highest_key.hash()))
        } else {
            Self::new_ranged(
                K::un_hash(map.highest_key.hash()),
                K::un_hash(map.lowest_key_hash),
            )
        }
    }
}
