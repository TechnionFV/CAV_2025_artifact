// ************************************************************************************************
// use
// ************************************************************************************************

use rand::Rng;

use crate::models::{Signal, UniqueSortedVec};

use super::TruthTable;

// ************************************************************************************************
// impl
// ************************************************************************************************

impl TruthTable {
    // ********************************************************************************************
    // API
    // ********************************************************************************************

    // pub fn make_array(
    //     number_of_truth_table_inputs: usize,
    //     default_value: TruthTableEntry,
    // ) -> Vec<TruthTableEntry> {
    //     let len = Self::calculate_number_of_variables_needed(number_of_truth_table_inputs);
    //     vec![default_value; len]
    // }

    /// Creates a new truth table that has one input and all the rows are false.
    pub fn new_all_false_truth_table(signal: &Signal) -> Self {
        let r = Self {
            truth_table: 0b00,
            input_names: UniqueSortedVec::from_ordered_set(vec![signal.to_owned()]),
            mask: 0b11,
        };
        debug_assert!(r.check().is_ok());
        r
    }

    /// Creates a new truth table that has one input and the function is the identity function.
    pub fn new_identity_truth_table(signal: &Signal) -> Self {
        let r = Self {
            truth_table: 0b10,
            input_names: UniqueSortedVec::from_ordered_set(vec![signal.to_owned()]),
            mask: 0b11,
        };
        debug_assert!(r.check().is_ok());
        r
    }

    pub fn new_truth_table_with_signals_renamed(
        truth_table: &Self,
        f: impl Fn(&Signal) -> Signal,
    ) -> Self {
        let mut input_names = truth_table.input_names.peek().to_owned();
        for x in input_names.iter_mut() {
            *x = f(x)
        }
        let order_before = input_names.clone();
        input_names.sort_unstable();
        let order_after = input_names;
        if order_before == order_after {
            let r = Self {
                truth_table: truth_table.truth_table,
                input_names: UniqueSortedVec::from_ordered_set(order_after),
                mask: truth_table.mask,
            };
            debug_assert!(r.check().is_ok());
            r
        } else {
            todo!("Re-ordering of truth tables is not yet implemented.")
        }
    }

    pub fn new_constant_0() -> Self {
        let r = Self {
            truth_table: 0,
            input_names: UniqueSortedVec::new(),
            mask: 1,
        };
        debug_assert!(r.check().is_ok());
        r
    }

    pub fn new_constant_1() -> Self {
        let mut tt = Self::new_constant_0();
        tt.negate();
        tt
    }

    /// New random truth table.
    pub fn new_random_truth_table<R: Rng>(
        rng: &mut R,
        number_of_inputs: usize,
        max_signal: Signal,
    ) -> Self {
        let input_names = UniqueSortedVec::from_sequence(
            (0..number_of_inputs)
                .map(|_| Signal::new(rng.gen_range(0..max_signal.number())))
                .collect(),
        );
        let mut r = Self {
            truth_table: 0,
            input_names,
            mask: 0,
        };
        r.mask = r.calculate_mask();
        r.truth_table = rng.gen_range(0..=r.mask);
        debug_assert!(r.check().is_ok());
        r
    }
}
