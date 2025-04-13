// ************************************************************************************************
// use
// ************************************************************************************************

use super::{TruthTable, TruthTableEntry};
use crate::models::{Signal, TernaryValue};

// ************************************************************************************************
// impl
// ************************************************************************************************

const INDEX_MASK: [TruthTableEntry; 7] = [
    (0xAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA & (TruthTableEntry::MAX as u128)) as TruthTableEntry,
    (0xCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCC & (TruthTableEntry::MAX as u128)) as TruthTableEntry,
    (0xF0F0F0F0F0F0F0F0F0F0F0F0F0F0F0F0 & (TruthTableEntry::MAX as u128)) as TruthTableEntry,
    (0xFF00FF00FF00FF00FF00FF00FF00FF00 & (TruthTableEntry::MAX as u128)) as TruthTableEntry,
    (0xFFFF0000FFFF0000FFFF0000FFFF0000 & (TruthTableEntry::MAX as u128)) as TruthTableEntry,
    (0xFFFFFFFF00000000FFFFFFFF00000000 & (TruthTableEntry::MAX as u128)) as TruthTableEntry,
    (0xFFFFFFFFFFFFFFFF0000000000000000 & (TruthTableEntry::MAX as u128)) as TruthTableEntry,
];

// ************************************************************************************************
// impl
// ************************************************************************************************

impl TruthTable {
    // ********************************************************************************************
    // helper functions
    // ********************************************************************************************

    /// TODO: make this function more efficient
    fn _get_mask_from_tri_inputs<I>(&self, inputs: I) -> TruthTableEntry
    where
        I: IntoIterator<Item = TernaryValue>,
    {
        // debug_assert_eq!(self.input_names.len(), inputs.len());

        // we start with all the bits being considered
        let mut mask = self.mask;
        for (value, index_mask) in inputs.into_iter().zip(INDEX_MASK.iter()) {
            match value {
                TernaryValue::True => {
                    mask &= index_mask;
                }
                TernaryValue::False => {
                    mask &= !index_mask;
                }
                TernaryValue::X => {} // no reduction in bits in mask
            }
        }
        // println!("inputs = {:?}, mask = {:b}", inputs, mask);
        mask
    }

    // ********************************************************************************************
    // API
    // ********************************************************************************************

    /// return the value of the index-th row in the truth table.
    pub fn get_value(&self, index: usize) -> bool {
        let mask = 1 << index;
        let r = self.truth_table & mask;
        r != 0
    }

    /// Get the ternary result of the truth table for the given tri-input.
    /// For example for the truth table 0b1110 over 'a' and 'b':
    /// 1. get_ternary_result(&[TernaryValue::X, TernaryValue::True]) == TernaryValue::True
    /// 2. get_ternary_result(&[TernaryValue::X, TernaryValue::False]) == TernaryValue::X
    /// 3. get_ternary_result(&[TernaryValue::False, TernaryValue::False]) == TernaryValue::False
    /// 4. get_ternary_result(&[TernaryValue::True, TernaryValue::X]) == TernaryValue::True
    pub fn get_ternary_result<I>(&self, inputs: I) -> TernaryValue
    where
        I: IntoIterator<Item = TernaryValue>,
    {
        // debug_assert_eq!(self.input_names.len(), inputs.len());
        // let mask = self.get_mask_from_tri_inputs(inputs);

        let mut mask = self.mask;
        for (value, index_mask) in inputs.into_iter().zip(INDEX_MASK.iter()) {
            match value {
                TernaryValue::True => {
                    mask &= index_mask;
                }
                TernaryValue::False => {
                    mask &= !index_mask;
                }
                TernaryValue::X => {
                    // no reduction in bits in mask
                    continue;
                }
            }

            let result = self.truth_table & mask;
            if result == 0 {
                return TernaryValue::False;
            } else if result == mask {
                // all lines are one
                return TernaryValue::True;
            }
        }

        let result = self.truth_table & mask;
        if result == 0 {
            TernaryValue::False
        } else if result == mask {
            // all lines are one
            return TernaryValue::True;
        } else {
            TernaryValue::X
        }
    }

    /// Get the signals that are irrelevant to the truth table.
    /// A signal is irrelevant if it does not change the output of the truth table.
    /// For example, for the truth table 0b1010 over 'a' and 'b'.
    pub fn get_irrelevant_signals(&self) -> Vec<Signal> {
        let mut v = vec![];
        for ((i, s), mask) in self.input_names.iter().enumerate().zip(INDEX_MASK) {
            let r_when_on = self.truth_table & mask;
            let shift = 1 << i;
            let r_when_off = (self.truth_table & !mask) << shift;
            if r_when_on == r_when_off {
                v.push(*s);
            }
        }
        v
    }
}
