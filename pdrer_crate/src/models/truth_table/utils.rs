// ************************************************************************************************
// use
// ************************************************************************************************

use super::{TruthTable, TruthTableEntry};
use crate::models::{truth_table::TRUTH_TABLE_MAX_INPUTS, Signal, UniqueSortedVec};
use std::fmt;

// ************************************************************************************************
// impl
// ************************************************************************************************

impl TruthTable {
    // ********************************************************************************************
    // helper functions
    // ********************************************************************************************

    /// duplicate the truth table in batches of size batch_size, works only for batch_size that is
    /// a power of 2.
    ///
    /// For example 01100011 will become:
    /// 1. with batch size 1 -> 0 1 1 0 0 0 1 1 1 1 -> 00 11 11 00 00 00 11 11 -> 0011110000001111
    /// 2. with batch size 2 -> 01 10 00 11 -> 0101 1010 0000 1111 -> 0101101000001111
    /// 3. with batch size 4 -> 0110 0011 -> 01100110 00110011 -> 0110011000110011
    /// 4. with batch size 8 -> 01100011 -> 0110001101100011
    fn duplicate_truth_table_in_batches(&mut self, batch_size: usize) {
        // println!(
        //     "truth_table before = {:b}, batch_size = {}",
        //     self.truth_table, batch_size
        // );
        debug_assert!(batch_size.is_power_of_two());
        let mut new_result = TruthTableEntry::MIN;
        let mask = !(TruthTableEntry::MAX << batch_size); // 0000000000001111 for batch size 4

        for i in 0..(((TruthTableEntry::BITS / 2) as usize) / batch_size) {
            let current_mask = mask << (i * batch_size);
            let batch = self.truth_table & current_mask;
            new_result |= batch << (i * batch_size);
            new_result |= batch << ((i + 1) * batch_size);
            // println!("i = {}, new_result = {:b}", i, new_result);
        }

        self.truth_table = new_result;
        self.mask = self.calculate_mask();
        // println!("truth_table after = {:b}", self.truth_table);
    }

    // ********************************************************************************************
    // API
    // ********************************************************************************************

    pub fn calculate_number_of_rows(&self) -> usize {
        1 << self.input_names.len()
    }

    /// adds an input to a truth table. Does nothing if the input already exists.
    pub fn add_truth_table_input(&mut self, signal: &Signal) {
        // huge performance hit
        // debug_assert!(Utils::is_sorted(&self.input_names));
        // debug_assert!(Utils::has_unique_elements(&self.input_names));

        // find where the point should be added
        let index = match self.input_names.peek().binary_search(signal) {
            Ok(_) => return, // already exists
            Err(index) => index,
        };

        assert!(
            self.input_names.len() < TRUTH_TABLE_MAX_INPUTS,
            "Truth table has too many inputs."
        );
        // insert the point
        debug_assert!(self.check().is_ok());
        self.input_names.insert(signal.to_owned());
        debug_assert!(self.check().is_err());
        // sanity checks
        // huge performance hit
        // debug_assert!(Utils::is_sorted(&self.input_names));
        // debug_assert!(Utils::has_unique_elements(&self.input_names));

        // bit shift depending on the index.
        self.duplicate_truth_table_in_batches(1 << index);
        debug_assert!(self.check().is_ok());
    }

    pub fn get_signals(&self) -> &UniqueSortedVec<Signal> {
        &self.input_names
    }

    /// Gets the number of inputs that the truth table accepts.
    pub fn len(&self) -> usize {
        self.input_names.len()
    }

    /// checks if the truth table has no inputs.
    /// Meaning that it is a constant.
    pub fn is_empty(&self) -> bool {
        self.input_names.is_empty()
    }

    /// Get the truth table.
    pub fn peek(&self) -> TruthTableEntry {
        self.truth_table
    }

    /// Get the mask of the truth table.
    pub fn get_mask(&self) -> TruthTableEntry {
        self.mask
    }

    /// checks if the truth table is a constant 0.
    /// Meaning that it has a single line that is zero.
    pub fn is_constant_0(&self) -> bool {
        self == &TruthTable::new_constant_0()
    }

    /// checks if the truth table is a constant 1.
    /// Meaning that it has a single line that is one.
    pub fn is_constant_1(&self) -> bool {
        self == &TruthTable::new_constant_1()
    }

    /// checks if all lines in the truth table are one.
    pub fn is_all_ones(&self) -> bool {
        self.truth_table == self.mask
    }

    /// checks if all lines in the truth table are zero.
    pub fn is_all_zeros(&self) -> bool {
        self.truth_table == 0
    }

    pub fn check(&self) -> Result<(), String> {
        if self.input_names.len() > TRUTH_TABLE_MAX_INPUTS {
            return Err("Truth table has too many inputs.".to_string());
        }
        if self.mask.count_ones() != (1 << self.input_names.len()) {
            return Err("Mask does not have the correct number of ones.".to_string());
        }
        if self.mask != TruthTableEntry::MAX && (self.mask + 1).count_ones() != 1 {
            return Err("Mask is not a power of 2 subtract 1.".to_string());
        }
        if self.truth_table > self.mask {
            return Err("Truth table is larger than the mask.".to_string());
        }
        Ok(())
    }
}

impl fmt::Display for TruthTable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let tt = self.truth_table;
        write!(f, "{tt:b}")
    }
}
