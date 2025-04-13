// ************************************************************************************************
// use
// ************************************************************************************************

// ************************************************************************************************
// struct
// ************************************************************************************************

use std::{
    fmt::{self, Debug},
    ops::Not,
};

use super::{unique_sorted_hash_map::UniqueSortedHash, Signal};

#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Wire {
    wire_number: u32,
}

// ************************************************************************************************
// impl
// ************************************************************************************************

impl Wire {
    pub const MAX: Wire = Self {
        wire_number: u32::MAX,
    };

    pub const CONSTANT_ZERO: Wire = Self { wire_number: 0 };
    pub const CONSTANT_ONE: Wire = Self { wire_number: 1 };

    pub fn new(wire_number: u32) -> Self {
        Self { wire_number }
    }

    pub fn signal(&self) -> Signal {
        Signal::new(self.wire_number >> 1)
    }

    pub fn get_signal_number(&self) -> u32 {
        self.wire_number >> 1
    }

    pub fn number(&self) -> u32 {
        self.wire_number
    }

    pub fn is_constant(&self) -> bool {
        self.signal().is_constant()
    }

    pub fn is_constant_one(&self) -> bool {
        self == &Self::CONSTANT_ONE
    }

    pub fn is_constant_zero(&self) -> bool {
        self == &Self::CONSTANT_ZERO
    }

    pub fn is_negated(&self) -> bool {
        (self.wire_number & 1) == 1
    }
}

// ************************************************************************************************
// hash
// ************************************************************************************************

impl UniqueSortedHash for Wire {
    fn hash(&self) -> usize {
        self.wire_number as usize
    }

    fn un_hash(i: usize) -> Self {
        Self::new(i as u32)
    }
}

// ************************************************************************************************
// negation
// ************************************************************************************************

impl Not for Wire {
    type Output = Wire;

    fn not(self) -> Self::Output {
        Wire::new(self.wire_number ^ 1)
    }
}

// ************************************************************************************************
// printing
// ************************************************************************************************

impl fmt::Display for Wire {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "W{}", self.wire_number)
    }
}

impl Debug for Wire {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "W{}", self.wire_number)
    }
}

// impl From<u32> for Wire {
//     fn from(wire_number: u32) -> Self {
//         Self::new(wire_number)
//     }
// }

// impl TryFrom<usize> for Wire {
//     type Error = ();

//     fn try_from(wire_number: usize) -> Result<Self, Self::Error> {
//         if wire_number > u32::MAX as usize {
//             Err(())
//         } else {
//             Ok(Self::new(wire_number as u32))
//         }
//     }
// }
