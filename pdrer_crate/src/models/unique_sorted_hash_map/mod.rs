// ************************************************************************************************
// use
// ************************************************************************************************

// ************************************************************************************************
// trait
// ************************************************************************************************

/// Trait for types that can be hashed and un-hashed to usize
/// Such types are for example Variable, Literal, Clause, etc.
pub trait UniqueSortedHash {
    fn hash(&self) -> usize;
    fn un_hash(i: usize) -> Self;
}

// ************************************************************************************************
// struct
// ************************************************************************************************

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct UniqueSortedHashMap<K: UniqueSortedHash, V> {
    vector: Vec<Option<V>>,
    length: usize, // number of not None elements
    lowest_key_hash: usize,
    highest_key: K,
}

// ************************************************************************************************
// rust submodule declaration, they get searched in their respective file  names
// ************************************************************************************************

pub mod construction;
pub mod operations;

// ************************************************************************************************
// re-exports of structs in these modules to simplify paths for other imports
// ************************************************************************************************
