// ************************************************************************************************
// use
// ************************************************************************************************

use super::{unique_sorted_hash_map::UniqueSortedHash, Wire};
use std::fmt::{self, Debug};

// ************************************************************************************************
// const type
// ************************************************************************************************

pub const CONSTANT_ZERO_SIGNAL: Signal = Signal { signal_number: 0 };

// ************************************************************************************************
// struct
// ************************************************************************************************

#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Signal {
    signal_number: u32,
}

// ************************************************************************************************
// impl
// ************************************************************************************************

impl Signal {
    pub const GROUND: Signal = Signal { signal_number: 0 };

    pub fn new(signal_number: u32) -> Self {
        Self { signal_number }
    }

    pub fn wire(&self, is_negated: bool) -> Wire {
        debug_assert!(self.signal_number < Wire::MAX.get_signal_number());
        if is_negated {
            Wire::new((self.signal_number << 1) + 1)
        } else {
            Wire::new(self.signal_number << 1)
        }
    }

    pub fn number(&self) -> u32 {
        self.signal_number
    }

    pub fn is_constant(&self) -> bool {
        self.signal_number == 0
    }
}

// ************************************************************************************************
// fmt
// ************************************************************************************************

impl fmt::Display for Signal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "S{}", self.signal_number)
    }
}

impl Debug for Signal {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "S{}", self.signal_number)
    }
}

// ************************************************************************************************
// impl UniqueSortedHash
// ************************************************************************************************

impl UniqueSortedHash for Signal {
    fn hash(&self) -> usize {
        self.signal_number as usize
    }

    fn un_hash(i: usize) -> Self {
        Signal::new(i as u32)
    }
}

// impl From<u32> for Signal {
//     fn from(s: u32) -> Self {
//         Self::new(s)
//     }
// }

// impl TryFrom<usize> for Signal {
//     type Error = ();

//     fn try_from(s: usize) -> Result<Self, Self::Error> {
//         if s > u32::MAX as usize {
//             Err(())
//         } else {
//             Ok(Self::new(s as u32))
//         }
//     }
// }
