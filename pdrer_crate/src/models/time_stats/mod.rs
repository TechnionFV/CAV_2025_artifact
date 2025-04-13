// ************************************************************************************************
// use
// ************************************************************************************************

use std::time::{Duration, Instant};

use fxhash::FxHashMap;

// ************************************************************************************************
// RefEquality
// ************************************************************************************************

/// Struct that allows us to compare references for equality and not for the value they point to.
/// This is used in the TimeStats struct.
/// This make TimeStats a lot more efficient when updating the time statistics of a function.
#[derive(Debug, Clone, PartialOrd, Ord)]
struct RefEquality(&'static str);

impl std::hash::Hash for RefEquality {
    fn hash<H>(&self, state: &mut H)
    where
        H: std::hash::Hasher,
    {
        std::ptr::hash(self.0, state)
    }
}

impl PartialEq for RefEquality {
    fn eq(&self, other: &RefEquality) -> bool {
        std::ptr::eq(self.0, other.0)
    }
}

impl Eq for RefEquality {}

// ************************************************************************************************
// struct
// ************************************************************************************************

#[derive(Debug, Clone)]
pub struct TimeStats {
    times: FxHashMap<RefEquality, (Duration, usize)>,
    start_time: Instant,
}

// ************************************************************************************************
// rust submodule declaration, they get searched in their respective file  names
// ************************************************************************************************

pub mod api;
pub mod function_timer;

// ************************************************************************************************
// re-exports of structs in these modules to simplify paths for other imports
// ************************************************************************************************
