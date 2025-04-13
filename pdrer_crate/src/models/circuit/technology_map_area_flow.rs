// ************************************************************************************************
// use
// ************************************************************************************************

// use std::cmp::min;

use super::{
    cut_enumeration::{CutSet, CutSetItem},
    Circuit,
};
use crate::models::{Signal, UniqueSortedHashMap};
type AreaFlow = u32;
type Value = i32;

// ************************************************************************************************
// impl
// ************************************************************************************************

impl Circuit {
    // ********************************************************************************************
    // helper functions
    // ********************************************************************************************

    // fn calculate_area_flow(&self, cut: &Cut, area_flows: &[usize]) -> (usize, usize) {
    //     let mut value = 0;
    //     let mut u_sign = 40; // should be sop size
    //     for signal in cut.iter() {
    //         let node = self.nodes.get(signal).unwrap();
    //         value += node.users.len();
    //         match node.node_type {
    //             CircuitNodeType::ConstantZero
    //             | CircuitNodeType::Input
    //             | CircuitNodeType::Latch { .. } => continue,
    //             CircuitNodeType::And { .. } => {}
    //             _ => panic!("Haven't implemented this yet."),
    //         }

    //         assert!(!node.users.is_empty());
    //         u_sign += area_flows[signal.number()] / node.users.len();
    //     }
    //     (value, u_sign)
    // }

    fn get_area_of_cut(&self, csi: &CutSetItem) -> usize {
        let tt = csi.truth_table.as_ref().unwrap();

        // tt.negate();
        // let b = tt.calculate_area();
        tt.calculate_area()
    }

    fn get_area_flow_of_gate_with_only_trivial_cut(
        &self,
        signal: &Signal,
        area_flows: &UniqueSortedHashMap<Signal, AreaFlow>,
        refs: &UniqueSortedHashMap<Signal, usize>,
    ) -> AreaFlow {
        match &self.nodes.get(signal).unwrap().node_type {
            super::node_types::CircuitNodeType::And(a) => {
                // area is number of clauses it takes to express this
                let mut area_flow: AreaFlow = (a.inputs.len() + 1) as AreaFlow;
                for signal in a.inputs.iter().map(|w| w.signal()) {
                    let refs = *refs.get(&signal).unwrap();
                    if !self.gates.contains(&signal) {
                        continue;
                    }
                    debug_assert!(refs > 0);
                    let den = if refs > 0 { refs } else { 1 } as AreaFlow;
                    area_flow += area_flows.get(&signal).unwrap() / den;
                }
                area_flow
            }
            _ => unreachable!(),
        }
    }

    fn get_area_flow_and_value(
        &self,
        csi: &CutSetItem,
        area_flows: &UniqueSortedHashMap<Signal, AreaFlow>,
        refs: &UniqueSortedHashMap<Signal, usize>,
    ) -> (AreaFlow, Value) {
        let mut value: Value = 0;
        let mut area_flow: AreaFlow = (10 * self.get_area_of_cut(csi)) as AreaFlow;
        for signal in csi.cut.iter() {
            let refs = *refs.get(signal).unwrap();
            value += refs as Value;
            if !self.gates.contains(signal) {
                continue;
            }
            debug_assert!(refs > 0);
            let den = if refs > 0 { refs } else { 1 } as AreaFlow;
            area_flow += area_flows.get(signal).unwrap() / den;
        }
        (area_flow, value)
    }

    /// from a vector of cuts, choose the "best" cut, where "best" is defined as the cut
    /// with the greatest minimum popularity of its signals. This is because we want to
    /// minimize the number of cuts. The only way for the unit cut to be chosen is if
    /// it is the only cut there is to choose from.
    fn choose_best_cut_by_area_flow(
        &self,
        signal: &Signal,
        area_flows: &mut UniqueSortedHashMap<Signal, AreaFlow>,
        cut_function: &UniqueSortedHashMap<Signal, CutSet>,
        refs: &UniqueSortedHashMap<Signal, usize>,
    ) -> CutSetItem {
        let possible_cuts = cut_function.get(signal).unwrap();
        debug_assert!(!possible_cuts.is_empty());

        if !self.gates.contains(signal) {
            debug_assert_eq!(possible_cuts.len(), 1);
            debug_assert_eq!(possible_cuts[0].cut.peek(), &[*signal]);
            area_flows.insert(*signal, 0);
            return possible_cuts[0].to_owned();
        }

        let mut best_cut: Option<(usize, AreaFlow, Value)> = None;
        for (index, cut) in possible_cuts.iter().enumerate() {
            if cut.cut.peek() == &[*signal] {
                continue;
            }
            let (are_flow, value) = self.get_area_flow_and_value(cut, area_flows, refs);

            // check if this is the first cut we see
            if best_cut.is_none()
                || (best_cut.unwrap().1 > are_flow)
                || ((best_cut.unwrap().1 == are_flow) && (best_cut.unwrap().2 < value))
            {
                best_cut = Some((index, are_flow, value));
                continue;
            }
        }

        if best_cut.is_none() {
            debug_assert_eq!(possible_cuts.len(), 1);
            debug_assert_eq!(possible_cuts[0].cut.peek(), &[*signal]);
            let area_flow =
                self.get_area_flow_of_gate_with_only_trivial_cut(signal, area_flows, refs);
            best_cut = Some((0, area_flow, 0))
        }

        // update are flows
        area_flows.insert(*signal, best_cut.unwrap().1);

        // return result
        possible_cuts[best_cut.unwrap().0].to_owned()
    }

    // ********************************************************************************************
    // API
    // ********************************************************************************************

    /// for each signal, choose the "best" cut for that signal
    /// "best" in the sense that the number of different signals in the cuts chosen is minimized
    pub fn choose_cut_for_each_signal_using_area_flow(
        &self,
        cut_function: &UniqueSortedHashMap<Signal, CutSet>,
    ) -> UniqueSortedHashMap<Signal, CutSetItem> {
        let mut result = UniqueSortedHashMap::new_like(cut_function);
        let mut area_flows = UniqueSortedHashMap::new_like(cut_function);
        let refs: UniqueSortedHashMap<Signal, usize> = self.get_number_of_references();

        for signal in cut_function.iter_sorted() {
            let cut =
                self.choose_best_cut_by_area_flow(&signal, &mut area_flows, cut_function, &refs);

            // debug_assert!({
            //     let is_unit_cut = cut.cut.peek() == &[signal.to_owned()];
            //     let is_gate = self.gates.contains(&signal);
            //     !is_unit_cut || (!is_gate)
            // });

            result.insert(signal.to_owned(), cut);
        }
        result
    }
}
