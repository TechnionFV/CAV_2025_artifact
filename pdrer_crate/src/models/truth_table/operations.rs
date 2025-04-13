// ************************************************************************************************
// use
// ************************************************************************************************

use crate::models::{
    truth_table::{TruthTableEntry, TRUTH_TABLE_MAX_INPUTS},
    Signal, UniqueSortedVec,
};

use super::TruthTable;

// ************************************************************************************************
// impl
// ************************************************************************************************

impl TruthTable {
    // ********************************************************************************************
    // helper functions
    // ********************************************************************************************

    // fn convert_index_to_assignment(index: usize, inputs: &[Signal]) -> FxHashMap<Signal, bool> {
    //     let mut result = FxHashMap::new();
    //     let mut index = index;
    //     for signal in inputs.iter() {
    //         let bit = index & 1;
    //         result.insert(signal.to_owned(), bit == 1);
    //         index >>= 1;
    //     }
    //     result
    // }

    pub(super) fn calculate_mask(&self) -> TruthTableEntry {
        let s = self.calculate_number_of_rows();
        if s == (TruthTableEntry::BITS as usize) {
            TruthTableEntry::MAX
        } else {
            !(TruthTableEntry::MAX << s)
        }
    }

    // ********************************************************************************************
    // API
    // ********************************************************************************************

    /// Negates all the rows in the truth table.
    pub fn negate(&mut self) {
        self.truth_table = self.mask & (!self.truth_table);
        debug_assert!(self.check().is_ok());
    }

    /// returns a new truth table that is the and of the two truth tables.
    /// If the two truth tables have different inputs, the inputs will be merged.
    /// If the new inputs exceed the maximum number of inputs, this function will panic.
    pub fn and(mut a: TruthTable, mut b: TruthTable) -> TruthTable {
        let inputs = UniqueSortedVec::merge(&a.input_names, &b.input_names);
        assert!(
            inputs.len() <= TRUTH_TABLE_MAX_INPUTS,
            "Truth table has too many inputs."
        );
        for input in inputs.iter() {
            a.add_truth_table_input(input);
            b.add_truth_table_input(input);
        }

        let mut result = Self {
            truth_table: TruthTableEntry::MIN,
            input_names: inputs,
            mask: 0,
        };
        result.mask = result.calculate_mask();

        // debug_assert_eq!(result.input_names, a.input_names);
        // debug_assert_eq!(result.input_names, b.input_names);

        // and the truth tables
        result.truth_table = a.truth_table & b.truth_table;
        debug_assert!(result.check().is_ok());
        result
    }

    /// Remove signal from the truth table.
    /// If assumed_value is true, the rows where the signal is true will be removed.
    /// If assumed_value is false, the rows where the signal is false will be removed.
    ///
    /// 1010000010100000 remove index 3 -> 10100000
    ///
    /// # Arguments
    ///
    /// * `signal` - The signal to remove.
    /// * `assumed_value` - The assumed value of the signal.
    ///
    /// # Panics
    ///
    /// This function will panic if the signal is not in the truth table.
    pub fn remove(&mut self, signal: Signal, assumed_value: bool) {
        let index = self.input_names.peek().binary_search(&signal).unwrap();
        let batch_size = 1 << index;

        let mut include = assumed_value;
        let mut truth_table = TruthTableEntry::MIN;
        let mut write_index = 0;

        let iterations = self.mask.count_ones();
        for i in 0..iterations {
            if i % batch_size == 0 {
                include = !include;
            }

            if include {
                let mask = 1 << i;
                let mut bit = self.truth_table & mask;
                bit >>= i;
                truth_table |= bit << write_index;
                write_index += 1;
            }
        }

        self.truth_table = truth_table;
        self.input_names.remove(&signal);
        // mask 11111111 -> 1111
        self.mask >>= iterations >> 1;
        // self.update_mask();
        debug_assert_eq!(self.calculate_mask(), self.mask);
        debug_assert!(self.check().is_ok());
    }

    pub fn simplify(&mut self) {
        let is_const_0 = self.is_all_zeros();
        let is_const_1 = self.is_all_ones();
        let irrelevant = self.get_irrelevant_signals();
        for s in irrelevant {
            debug_assert!({
                let mut tt_1 = self.clone();
                let mut tt_2 = self.clone();
                tt_1.remove(s, true);
                tt_2.remove(s, false);
                // tt_1.get_signals() == tt_2.get_signals()
                tt_1 == tt_2
                    && self.get_signals().contains(&s)
                    && !tt_1.get_signals().contains(&s)
                    && tt_1.get_signals().is_subset_of(self.get_signals())
                    && tt_1.get_signals().len() == self.get_signals().len() - 1
            });
            self.remove(s, false);
        }
        debug_assert!(is_const_0 || is_const_1 || !self.get_signals().is_empty());
        debug_assert!(!is_const_0 || self == &TruthTable::new_constant_0());
        debug_assert!(!is_const_1 || self == &TruthTable::new_constant_1());
    }

    // pub fn get_value(&self, index: usize) -> bool {
    //     let u32_index = index / TruthT;
    //     let bit_index = index % 32;
    //     let mask = 1 << bit_index;
    //     (self.truth_table[u32_index] & mask) != 0
    // }

    // pub fn set_value(&mut self, index: usize, value: bool) {
    //     let u32_index = index / 32;
    //     let bit_index = index % 32;
    //     let mask = 1 << bit_index;
    //     if value {
    //         self.truth_table[u32_index] |= mask;
    //     } else {
    //         self.truth_table[u32_index] &= !mask;
    //     }
    // }

    // pub fn get_value_smart(&self, assignment: &FxHashMap<Signal, bool>) -> bool {
    //     let mut index = 0;
    //     for input in &self.input_names {
    //         let bit = assignment[input];
    //         index = (index << 1) | (bit as usize);
    //     }
    //     self.get_value(index)
    // }
}

// ************************************************************************************************
// test
// ************************************************************************************************

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::TernaryValue;
    use rand::{
        rngs::{StdRng, ThreadRng},
        Rng, SeedableRng,
    };

    #[test]
    fn test_and() {
        let max_signal = Signal::new(TRUTH_TABLE_MAX_INPUTS as u32);
        let max_number_of_inputs = TRUTH_TABLE_MAX_INPUTS;
        let iterations = 10000;
        for _ in 0..iterations {
            let seed: u64 = ThreadRng::default().gen();
            println!("seed = {}", seed);
            let mut rng = StdRng::seed_from_u64(seed);
            let i_a = rng.gen_range(0..max_number_of_inputs);
            let a = TruthTable::new_random_truth_table(&mut rng, i_a, max_signal);
            let i_b = rng.gen_range(0..max_number_of_inputs);
            let b = TruthTable::new_random_truth_table(&mut rng, i_b, max_signal);

            let c = TruthTable::and(a.clone(), b.clone());
            let d = TruthTable::and(b.clone(), a.clone());
            assert_eq!(c, d);
            // assert_eq!(c.get_signals(), a.get_signals());
            // assert_eq!(c.get_signals(), b.get_signals());
            assert!(a.get_signals().is_subset_of(c.get_signals()));
            assert!(b.get_signals().is_subset_of(c.get_signals()));
            // check truth table for correctness
            for _ in 0..d.calculate_number_of_rows() {
                let signal_and_values: Vec<_> = c
                    .get_signals()
                    .iter()
                    .copied()
                    .zip(c.get_signals().iter().map(|_| {
                        if rng.gen_bool(0.5) {
                            TernaryValue::False
                        } else {
                            TernaryValue::True
                        }
                    }))
                    .collect();

                let c_value = c.get_ternary_result(signal_and_values.iter().map(|(_, v)| *v));

                let a_value = a.get_ternary_result(
                    signal_and_values
                        .iter()
                        .filter(|(s, _)| a.get_signals().contains(s))
                        .map(|(_, v)| *v),
                );
                let b_value = b.get_ternary_result(
                    signal_and_values
                        .iter()
                        .filter(|(s, _)| b.get_signals().contains(s))
                        .map(|(_, v)| *v),
                );
                let tri_value_to_bool = |v: TernaryValue| match v {
                    TernaryValue::True => true,
                    TernaryValue::False => false,
                    TernaryValue::X => unreachable!(),
                };
                assert_eq!(
                    tri_value_to_bool(c_value),
                    tri_value_to_bool(a_value) && tri_value_to_bool(b_value)
                );
            }
        }
    }

    #[test]
    fn test_simplify() {
        let max_signal = Signal::new(TRUTH_TABLE_MAX_INPUTS as u32);
        let max_number_of_inputs = TRUTH_TABLE_MAX_INPUTS;

        let iterations = 100000;
        for _ in 0..iterations {
            let seed: u64 = ThreadRng::default().gen();
            println!("seed = {}", seed);
            let mut rng = StdRng::seed_from_u64(seed);
            let i_a = rng.gen_range(0..max_number_of_inputs);
            let a = TruthTable::new_random_truth_table(&mut rng, i_a, max_signal);
            let mut simp_a = a.clone();
            simp_a.simplify();
            assert!(simp_a.get_signals().is_subset_of(a.get_signals()));
            if a.is_constant_0() {
                assert_eq!(simp_a, TruthTable::new_constant_0());
            } else if a.is_constant_1() {
                assert_eq!(simp_a, TruthTable::new_constant_1());
            }
        }
    }

    #[test]
    fn test_negate() {
        let max_signal = Signal::new(TRUTH_TABLE_MAX_INPUTS as u32);
        let max_number_of_inputs = TRUTH_TABLE_MAX_INPUTS;

        let iterations = 100000;
        for _ in 0..iterations {
            let seed: u64 = ThreadRng::default().gen();
            println!("seed = {}", seed);
            let mut rng = StdRng::seed_from_u64(seed);
            let i_a = rng.gen_range(0..max_number_of_inputs);
            let a = TruthTable::new_random_truth_table(&mut rng, i_a, max_signal);
            let mut neg_a = a.clone();
            neg_a.negate();
            neg_a.negate();
            assert_eq!(neg_a, a);
        }
    }
}
