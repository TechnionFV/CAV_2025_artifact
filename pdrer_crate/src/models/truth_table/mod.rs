// ************************************************************************************************
// use
// ************************************************************************************************

use super::{Signal, UniqueSortedVec};

// ************************************************************************************************
// type
// ************************************************************************************************

pub type TruthTableEntry = u16;
// pub const TRUTH_TABLE_ENTRY_IDENTITY: TruthTableEntry = 0xAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA;
pub const TRUTH_TABLE_MAX_INPUTS: usize = 4; // 2^7 = 128

// ************************************************************************************************
// struct
// ************************************************************************************************

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TruthTable {
    input_names: UniqueSortedVec<Signal>,
    truth_table: TruthTableEntry,
    mask: TruthTableEntry, // this field tells us which bits in the truth table matter.
}

// ************************************************************************************************
// rust submodule declaration, they get searched in their respective file  names
// ************************************************************************************************

pub mod cnf;
pub mod construction;
pub mod operations;
pub mod ternary_result;
pub mod utils;

// ************************************************************************************************
// re-exports of structs in these modules to simplify paths for other imports
// ************************************************************************************************
