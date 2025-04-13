// ************************************************************************************************
// use
// ************************************************************************************************

use fxhash::FxHashMap;

use super::{cut_enumeration::Cut, Circuit};
use crate::models::{Signal, UniqueSortedHashMap};

// ************************************************************************************************
// impl
// ************************************************************************************************

impl Circuit {
    // ********************************************************************************************
    // helper functions
    // ********************************************************************************************

    // Lower is better
    fn get_cost_of_levels_in_cut(
        &self,
        cut: &Cut,
        levels: &UniqueSortedHashMap<Signal, u32>,
    ) -> f64 {
        let mut result = 0.0;
        for signal in cut.iter() {
            let level = *levels.get(signal).unwrap();
            if level == 0 {
                continue;
            }
            // the higher the level, the higher the cost in an exponential way
            result += 2.0_f64.powi(level as i32);
        }
        result
    }

    /// from a vector of cuts, choose the "best" cut, where "best" is defined as the cut
    /// with the greatest minimum popularity of its signals. This is because we want to
    /// minimize the number of cuts.
    fn choose_best_cut_by_lowest_levels(
        &self,
        signal: &Signal,
        cut_set: &[Cut],
        levels: &UniqueSortedHashMap<Signal, u32>,
    ) -> Cut {
        //
        if cut_set.len() == 1 {
            return cut_set[0].to_owned();
        }
        let mut min_cost = f64::INFINITY;
        let mut arg_min = -1;
        for (i, cut) in cut_set.iter().enumerate() {
            // skip the unit cut
            let is_unit_cut = cut.peek() == &[signal.to_owned()];
            if is_unit_cut {
                continue;
            }

            // calculate the cost of the levels in the cut
            let cost = self.get_cost_of_levels_in_cut(cut, levels);

            // update the min cost
            if (arg_min == -1) || (cost < min_cost) {
                arg_min = i as i32;
                min_cost = cost;
            }
        }
        assert!(
            arg_min != -1,
            "No cut chosen for signal = {}",
            signal.number()
        );
        cut_set[arg_min as usize].to_owned()
    }

    // ********************************************************************************************
    // API
    // ********************************************************************************************

    /// for each signal, choose the "best" cut for that signal
    /// "best" in the sense that the number of different signals in the cuts chosen is minimized
    pub fn choose_cut_for_each_signal_using_lowest_level_cuts(
        &self,
        cut_function: &FxHashMap<Signal, Vec<Cut>>,
    ) -> FxHashMap<Signal, Cut> {
        let mut result =
            FxHashMap::with_capacity_and_hasher(cut_function.len(), Default::default());
        let levels: UniqueSortedHashMap<Signal, u32> = self.get_level_per_signal();
        for (signal, cut_set) in cut_function.iter() {
            let cut = self.choose_best_cut_by_lowest_levels(signal, cut_set, &levels);
            let is_unit_cut = cut.peek() == &[signal.to_owned()];
            let is_gate = self.gates.contains(signal);
            if cut_set.len() > 1 {
                assert!(
                    !(is_unit_cut && is_gate),
                    "Unit cut chosen for signal = {}",
                    signal.number()
                );
            }
            result.insert(signal.to_owned(), cut);
        }
        result
    }
}
