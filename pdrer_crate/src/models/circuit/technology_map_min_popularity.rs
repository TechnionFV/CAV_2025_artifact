// ************************************************************************************************
// use
// ************************************************************************************************

use super::{cut_enumeration::Cut, Circuit};
use crate::models::{Signal, UniqueSortedHashMap};

// ************************************************************************************************
// impl
// ************************************************************************************************

impl Circuit {
    // ********************************************************************************************
    // helper functions
    // ********************************************************************************************

    /// For each signal, figure out how many times that signal appears in the cuts
    fn get_signal_popularity_from_cut_set(
        cut_function: &UniqueSortedHashMap<Signal, Vec<Cut>>,
    ) -> UniqueSortedHashMap<Signal, usize> {
        let mut signal_popularity = UniqueSortedHashMap::new_like(cut_function);
        for cut_set in cut_function.iter_items() {
            for cut in cut_set.iter() {
                for signal in cut.iter() {
                    let h = signal_popularity.get_mut(signal);
                    match h {
                        Some(c) => *c += 1,
                        None => {
                            signal_popularity.insert(signal.to_owned(), 1);
                        }
                    }
                }
            }
        }
        signal_popularity
    }

    /// Gets the minimum popularity of the signals in the cut.
    fn get_minimum_popularity_of_cut(
        signal_of_cut: &Signal,
        cut: &Cut,
        signal_popularity: &UniqueSortedHashMap<Signal, usize>,
    ) -> usize {
        if cut.peek() == &[signal_of_cut.to_owned()] {
            // avoid the unit cut
            return 0;
        }
        let mut min_popularity = usize::MAX;
        for signal in cut.iter() {
            let popularity = signal_popularity.get(signal).unwrap();
            if *popularity < min_popularity {
                min_popularity = *popularity;
            }
        }
        min_popularity
    }

    /// from a vector of cuts, choose the "best" cut, where "best" is defined as the cut
    /// with the greatest minimum popularity of its signals. This is because we want to
    /// minimize the number of cuts.
    fn choose_best_cut_by_min_popularity(
        signal_of_cut: &Signal,
        cut_set: &[Cut],
        signal_popularity: &UniqueSortedHashMap<Signal, usize>,
    ) -> Cut {
        let min_popularity_for_each_index = cut_set
            .iter()
            .map(|cut| Self::get_minimum_popularity_of_cut(signal_of_cut, cut, signal_popularity));
        let arg_max = min_popularity_for_each_index
            .enumerate()
            .max_by(|(_, value0), (_, value1)| value0.cmp(value1))
            .map(|(idx, _)| idx)
            .unwrap();
        cut_set[arg_max].to_owned()
    }

    // ********************************************************************************************
    // API
    // ********************************************************************************************

    /// for each signal, choose the "best" cut for that signal
    /// "best" in the sense that the number of different signals in the cuts chosen is minimized
    pub fn choose_cut_for_each_signal_using_min_popularity(
        cut_function: &UniqueSortedHashMap<Signal, Vec<Cut>>,
    ) -> UniqueSortedHashMap<Signal, Cut> {
        let signal_popularity = Self::get_signal_popularity_from_cut_set(cut_function);
        let mut result = UniqueSortedHashMap::new_like(cut_function);
        for (signal, cut_set) in cut_function.iter_pairs() {
            // let cut_set = cut_function.get(&signal).unwrap();
            let cut = Self::choose_best_cut_by_min_popularity(&signal, cut_set, &signal_popularity);
            debug_assert!(
                (cut.peek() == &[signal.to_owned()]) // this is either the unit cut
                    || (!cut.contains(&signal)) // or this is a cut that doesn't contain the signal
            );
            result.insert(signal.to_owned(), cut);
        }
        result
    }
}
