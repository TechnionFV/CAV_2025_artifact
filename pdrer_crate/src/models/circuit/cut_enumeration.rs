//! Cut enumeration is mentioned in the paper:
//! ?
//!
//! Implemented using iterators to reduce memory usage.

// ************************************************************************************************
// use
// ************************************************************************************************

use super::{node_types::CircuitNodeType, Circuit};
use crate::models::{Signal, TruthTable, UniqueSortedHashMap, UniqueSortedVec, Wire};

// ************************************************************************************************
// types
// ************************************************************************************************

pub type Cut = UniqueSortedVec<Signal>;
pub type CutSet = Vec<CutSetItem>;
// type to store cuts in vector
#[derive(Debug, Clone)]
pub struct CutSetItem {
    pub cut: Cut,
    pub truth_table: Option<TruthTable>,
    pub cost: i32,
}

// ************************************************************************************************
// types
// ************************************************************************************************

struct CutSetItemIterator<'a> {
    circuit: &'a Circuit,
    signal: Signal,
    k: usize,
    lengths: Vec<usize>,
    cut_sets: Vec<&'a CutSet>,
    inputs: Vec<Wire>,
    gather_truth_table: bool,
    indexes: Vec<usize>,
    is_done: bool,
}

impl<'a> CutSetItemIterator<'a> {
    // #[allow(clippy::too_many_arguments)]
    fn new(
        circuit: &'a Circuit,
        signal: Signal,
        k: usize,
        gather_truth_table: bool,
        cut_sets: Vec<&'a CutSet>,
        inputs: Vec<Wire>,
    ) -> Self {
        debug_assert_eq!(inputs.len(), cut_sets.len());
        let lengths: Vec<usize> = cut_sets.iter().map(|x| x.len()).collect();
        debug_assert!(!lengths.contains(&0));
        let indexes = vec![0; lengths.len()];
        Self {
            is_done: false,
            circuit,
            signal,
            k,
            lengths,
            cut_sets,
            inputs,
            gather_truth_table,
            indexes,
        }
    }

    fn get_cut(&self) -> Result<CutSetItem, usize> {
        // get the correct cuts
        let (cut_set, truth_tables): (Vec<Cut>, Vec<Option<TruthTable>>) = self
            .indexes
            .iter()
            .zip(self.cut_sets.iter())
            .map(|(k, cs)| cs[*k].to_owned())
            .map(|x| (x.cut, x.truth_table))
            .unzip();

        let mut cut: Option<Cut> = None;
        for (i, other) in cut_set.iter().enumerate() {
            cut = match cut.as_mut() {
                Some(c) => Some(c.merge(other)),
                None => Some(other.to_owned()),
            };

            if cut.as_ref().unwrap().len() > self.k {
                // must skip multiple iterations if we have some and gates with many inputs.
                return Err(i);
            }
        }
        let cut = cut.unwrap();

        // get truth table of cut
        let tt = if self.gather_truth_table {
            Some(
                match &self.circuit.get_node(&self.signal).unwrap().node_type {
                    CircuitNodeType::And(_) => truth_tables
                        .into_iter()
                        .zip(self.inputs.iter())
                        .fold(TruthTable::new_constant_1(), |acc, (tt, input)| {
                            let mut current_tt = tt.unwrap();
                            if input.is_negated() {
                                current_tt.negate();
                            }
                            TruthTable::and(acc, current_tt)
                        }),
                    CircuitNodeType::GenericGate(_) => todo!(),
                    _ => unreachable!(),
                },
            )
        } else {
            None
        };

        Ok(CutSetItem {
            cut,
            truth_table: tt,
            cost: 0,
        })
    }

    fn try_to_increment_indexes(&mut self, i: usize) -> Result<(), ()> {
        // try to add
        for (j, (index, length)) in self
            .indexes
            .iter_mut()
            .zip(self.lengths.iter())
            .enumerate()
            .rev()
        {
            debug_assert!(*index < *length);
            if j > i {
                // clear indexes bigger than i
                *index = 0;
                continue;
            } else {
                // try to add increment index
                if *index < (*length - 1) {
                    // increment this index and we are done
                    *index += 1;
                    return Ok(());
                } else {
                    // clear this index too and move on
                    *index = 0;
                }
            }
        }
        // went over all indexes
        Err(())
    }
}

impl Iterator for CutSetItemIterator<'_> {
    type Item = CutSetItem;

    fn next(&mut self) -> Option<Self::Item> {
        if self.is_done {
            return None;
        }
        loop {
            let r = self.get_cut();
            match r {
                Ok(obj) => {
                    match self.try_to_increment_indexes(self.indexes.len() - 1) {
                        Ok(_) => {}
                        Err(_) => {
                            /* must fail nex time */
                            self.is_done = true;
                        }
                    }
                    return Some(obj);
                }
                Err(i) => match self.try_to_increment_indexes(i) {
                    Ok(_) => {}
                    Err(_) => return None,
                },
            }
        }
    }
}

// ************************************************************************************************
// impl
// ************************************************************************************************

impl Circuit {
    // ********************************************************************************************
    // helper functions
    // ********************************************************************************************

    fn get_all_k_feasible_cuts_of_gate<'a>(
        &'a self,
        signal: Signal,
        k: usize,
        gather_truth_table: bool,
        result: &'a UniqueSortedHashMap<Signal, CutSet>,
    ) -> impl Iterator<Item = CutSetItem> + 'a {
        match &self.get_node(&signal).unwrap().node_type {
            CircuitNodeType::And(a) => {
                let inputs = a.inputs.peek().clone();
                let cut_sets: Vec<&Vec<_>> = inputs
                    .iter()
                    .map(|i| result.get(&i.signal()).unwrap())
                    .collect();
                debug_assert!(cut_sets.iter().all(|x| !x.is_empty()));

                // return iterator
                CutSetItemIterator::new(self, signal, k, gather_truth_table, cut_sets, inputs)
            }
            CircuitNodeType::GenericGate(a) => {
                let inputs: Vec<Wire> = a
                    .truth_table
                    .get_signals()
                    .iter()
                    .map(|s| s.wire(false))
                    .collect();
                let cut_sets: Vec<&Vec<_>> = inputs
                    .iter()
                    .map(|i| result.get(&i.signal()).unwrap())
                    .collect();
                debug_assert!(cut_sets.iter().all(|x| !x.is_empty()));

                // return iterator
                CutSetItemIterator::new(self, signal, k, gather_truth_table, cut_sets, inputs)
            }
            _ => unreachable!(),
        }
    }

    pub fn cut_value_function_used_by_abc(
        &self,
        cut: &Cut,
        k: usize,
        refs: &UniqueSortedHashMap<Signal, usize>,
    ) -> usize {
        if cut.len() < 2 {
            return 1001;
        }

        let mut value = 0;
        let mut number_of_ones = 0;

        for signal in cut.iter() {
            // calculate refs similar to how ABC does it
            let refs = *refs.get(signal).unwrap();

            value += refs;
            if refs == 1 {
                number_of_ones += 1;
            }
        }

        if value > 1000 {
            value = 1000;
        }
        if number_of_ones > (k - 1) {
            value = k + 1 - number_of_ones;
        }
        value
    }

    /// simplify cut.
    ///
    /// If gather_truth_table is false, then the cut is simplified using bounded cone.
    /// This prevents the cut from including signals that are in the cone of other
    /// signals in the same cut.
    ///
    /// If gather_truth_table is true, then the cut is simplified using the truth table.
    /// Meaning if an element in the cut was found to be irrelevant by the truth table,
    /// it is removed from the cut.
    fn simplify_cut_if_possible(
        &self,
        signal: Signal,
        cut_set_item: &mut CutSetItem,
        gather_truth_table: bool,
    ) {
        if !gather_truth_table {
            // simplify using bounded cone
            debug_assert!(cut_set_item.truth_table.is_none());
            let bounded_cone = self.get_cone_of_signal_bounded_by_cut(&signal, &cut_set_item.cut);
            cut_set_item.cut.retain(|x| bounded_cone.contains(x));
        } else {
            // let bounded_cone = self.get_cone_of_signal_bounded_by_cut(&signal, cut);
            let tt = cut_set_item.truth_table.as_mut().unwrap();
            tt.simplify();
            tt.get_signals().clone_into(&mut cut_set_item.cut);
            debug_assert_eq!(tt.get_signals(), &cut_set_item.cut);
        }
    }

    /// Returns true if we should skip the cut
    fn removed_superset_cuts(cuts: &mut [Option<CutSetItem>], cut: &Cut) -> bool {
        for cut_and_cost_option in cuts.iter_mut() {
            match cut_and_cost_option {
                Some(cut_and_cost) => {
                    if cut_and_cost.cut.is_subset_of(cut) {
                        // there is already a cut that is a subset of this cut
                        return true;
                    } else if cut.is_subset_of(&cut_and_cost.cut) {
                        // this cut is a subset of a cut that is already in the list
                        // remove that cut, and move on to find more subsets
                        *cut_and_cost_option = None;
                    }
                }
                None => continue,
            };
        }
        false
    }

    fn calculate_cuts_for_gate<F>(
        &self,
        signal: Signal,
        k: usize,
        l: usize,
        cost_function: F,
        gather_truth_table: bool,
        result: &mut UniqueSortedHashMap<Signal, CutSet>,
    ) where
        F: Fn(&Cut) -> i32,
    {
        // create vector to store cuts
        let mut cuts: Vec<Option<CutSetItem>> = vec![None; l];

        // create trivial cut
        let tt = if gather_truth_table {
            Some(TruthTable::new_identity_truth_table(&signal))
        } else {
            None
        };
        let trivial_cut: Cut = UniqueSortedVec::from_ordered_set(vec![signal]);

        // insert trivial cut at end just like abc does
        cuts.first_mut().unwrap().replace(CutSetItem {
            cut: trivial_cut.clone(),
            truth_table: tt.clone(),
            cost: cost_function(&trivial_cut),
        });

        // go over k length cuts
        for mut cut_set_item in
            self.get_all_k_feasible_cuts_of_gate(signal, k, gather_truth_table, result)
        {
            self.simplify_cut_if_possible(signal, &mut cut_set_item, gather_truth_table);
            if Self::removed_superset_cuts(&mut cuts, &cut_set_item.cut) {
                continue;
            }
            cut_set_item.cost = cost_function(&cut_set_item.cut);

            // find empty index
            let mut index = cuts.iter().position(|x| x.is_none());
            // find index of cut with highest cost
            if index.is_none() {
                let highest_cost_index = cuts
                    .iter()
                    .enumerate()
                    .filter(|(_, c)| c.is_some())
                    .max_by_key(|(_, c)| c.as_ref().unwrap().cost)
                    .map(|(i, _)| i)
                    .unwrap();
                if cut_set_item.cost <= cuts[highest_cost_index].as_ref().unwrap().cost {
                    index = Some(highest_cost_index);
                }
            }

            // insert cut if applicable
            if let Some(index) = index {
                cuts[index].replace(cut_set_item);
            }
        }

        // insert cuts
        result.insert(signal, cuts.into_iter().flatten().collect());
    }

    // ********************************************************************************************
    // API
    // ********************************************************************************************

    pub fn cut_cost_function_used_by_abc(
        &self,
        cut: &Cut,
        k: usize,
        refs: &UniqueSortedHashMap<Signal, usize>,
    ) -> i32 {
        let value = self.cut_value_function_used_by_abc(cut, k, refs);
        // abc also prefers smaller cuts, to get that we make sure that
        // we firs compare by length then by compared using the size of the cut
        const MAX_VALUE: i32 = 10000;
        let mut cost = -(value as i32);
        cost += (cut.len() as i32) * MAX_VALUE;
        cost
        // -(value as i32)
    }

    // pub fn cut_cost_function_2(&self, cut: &Cut, _k: usize) -> i32 {
    //     let mut result = 0;
    //     for signal in cut.iter() {
    //         let level = self.nodes.get(signal).unwrap().level;
    //         result += 2_i32.pow(level as u32)
    //     }
    //     result
    // }

    /// returns that enumerates all the possible cuts for each node in the circuit.
    /// Since the number of cuts can be exponential, we only keep the k best cuts for each node.
    /// The cost function is used to determine the quality of the cut.
    pub fn enumerate_k_feasible_cuts_leaving_only_l_best<F>(
        &self,
        k: usize,
        l: usize,
        cost_function: F,
        gather_truth_table: bool,
    ) -> UniqueSortedHashMap<Signal, CutSet>
    where
        F: Fn(&Cut) -> i32,
    {
        // get signals in order
        let signals: Vec<Signal> = self.nodes.iter_sorted().collect();

        let mut result: UniqueSortedHashMap<Signal, CutSet> =
            UniqueSortedHashMap::new_like(&self.nodes);

        // go over all nodes and call pattern matcher.
        for signal in signals.into_iter() {
            // get node
            let node = self.nodes.get(&signal).unwrap();
            match &node.node_type {
                CircuitNodeType::ConstantZero
                | CircuitNodeType::Input
                | CircuitNodeType::Latch { .. } => {
                    let mut cut_set_item = CutSetItem {
                        cut: UniqueSortedVec::from_ordered_set(vec![signal]),
                        truth_table: None,
                        cost: 999999999,
                    };
                    if gather_truth_table {
                        cut_set_item.truth_table = Some(match &node.node_type {
                            CircuitNodeType::ConstantZero => TruthTable::new_constant_0(),
                            CircuitNodeType::Input | CircuitNodeType::Latch { .. } => {
                                TruthTable::new_identity_truth_table(&signal)
                            }
                            _ => unreachable!(),
                        });
                    }
                    result.insert(signal, vec![cut_set_item]);
                }
                CircuitNodeType::And(_) | CircuitNodeType::GenericGate(_) => self
                    .calculate_cuts_for_gate(
                        signal,
                        k,
                        l,
                        &cost_function,
                        gather_truth_table,
                        &mut result,
                    ),
            }
        }

        result
    }
}
