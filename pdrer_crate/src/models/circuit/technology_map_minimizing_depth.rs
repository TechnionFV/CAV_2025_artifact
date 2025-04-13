// ************************************************************************************************
// use
// ************************************************************************************************

use super::{
    cut_enumeration::{Cut, CutSet, CutSetItem},
    Circuit,
};
use crate::models::{Signal, UniqueSortedHashMap};

// ************************************************************************************************
// impl
// ************************************************************************************************

impl Circuit {
    // ********************************************************************************************
    // helper functions
    // ********************************************************************************************

    fn get_depth_of_cut(
        &self,
        signal: &Signal,
        cut: &Cut,
        depths: &UniqueSortedHashMap<Signal, usize>,
    ) -> usize {
        let get_depth_from_signals = |signals: &[Signal]| {
            signals
                .iter()
                .map(|x| depths.get(x).unwrap())
                .max()
                .map(|d| *d + 1)
                .unwrap_or(0)
        };
        if depths.contains_key(signal) {
            // avoid the unit cut
            *depths.get(signal).unwrap()
        } else if cut.peek() == &[signal.to_owned()] {
            // the cut is the trivial cut, must calculate depth using signal
            match &self.nodes.get(signal).unwrap().node_type {
                super::node_types::CircuitNodeType::ConstantZero => 0,
                super::node_types::CircuitNodeType::Input => 0,
                super::node_types::CircuitNodeType::Latch(_) => 0,
                super::node_types::CircuitNodeType::And(a) => {
                    get_depth_from_signals(&a.inputs.iter().map(|w| w.signal()).collect::<Vec<_>>())
                }
                super::node_types::CircuitNodeType::GenericGate(a) => get_depth_from_signals(
                    &a.truth_table
                        .get_signals()
                        .iter()
                        .copied()
                        .collect::<Vec<_>>(),
                ),
            }
        } else {
            get_depth_from_signals(cut.peek())
        }
    }

    fn update_depth_for_signal(
        &self,
        signal: &Signal,
        chosen_cut: &Cut,
        depths: &mut UniqueSortedHashMap<Signal, usize>,
    ) {
        let depth = self.get_depth_of_cut(signal, chosen_cut, depths);
        depths.insert(signal.to_owned(), depth);
    }

    fn choose_best_cut_minimizing_depth(
        &self,
        signal: &Signal,
        cut_set: &CutSet,
        depths: &UniqueSortedHashMap<Signal, usize>,
    ) -> usize {
        let mut best_depth_and_its_index = None;
        for (i, cut_set_item) in cut_set.iter().enumerate() {
            let depth = self.get_depth_of_cut(signal, &cut_set_item.cut, depths);
            match best_depth_and_its_index {
                // first iteration, update best
                None => {
                    best_depth_and_its_index = Some((depth, i));
                }
                Some((best_depth, index)) => {
                    let better_depth = depth < best_depth;
                    let same_depth = depth == best_depth;
                    let smaller_cut = cut_set_item.cut.len() < cut_set[index].cut.len();
                    if better_depth || (same_depth && smaller_cut) {
                        best_depth_and_its_index = Some((depth, i));
                    }
                }
            }
        }
        best_depth_and_its_index.unwrap().1
    }

    // ********************************************************************************************
    // API
    // ********************************************************************************************

    pub fn choose_cut_for_each_signal_minimizing_depth(
        &self,
        cut_function: &UniqueSortedHashMap<Signal, CutSet>,
    ) -> UniqueSortedHashMap<Signal, CutSetItem> {
        let mut depths = UniqueSortedHashMap::new_like(cut_function);

        depths.insert(Signal::new(0), 0);

        for signal in self.get_input_signals().iter() {
            depths.insert(signal.to_owned(), 0);
        }

        for signal in self.get_latch_signals().iter() {
            depths.insert(signal.to_owned(), 0);
        }

        let mut result = UniqueSortedHashMap::new_like(cut_function);
        for (signal, cut_set) in cut_function.iter_pairs() {
            let i = self.choose_best_cut_minimizing_depth(&signal, cut_set, &depths);
            let cut_set_item = cut_set[i].to_owned();
            self.update_depth_for_signal(&signal, &cut_set_item.cut, &mut depths);
            debug_assert!(
                (cut_set_item.cut.peek() == &[signal.to_owned()]) // this is either the unit cut
                    || (!cut_set_item.cut.contains(&signal)) // or this is a cut that doesn't contain the signal
            );
            result.insert(signal.to_owned(), cut_set_item);
        }
        result
    }
}
