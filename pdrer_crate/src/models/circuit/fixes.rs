// ************************************************************************************************
// use
// ************************************************************************************************

use super::{node_types::CircuitNode, Circuit};
use crate::models::{
    circuit::node_types::CircuitNodeType, Signal, UniqueSortedHashMap, UniqueSortedVec,
};

// ************************************************************************************************
// impl
// ************************************************************************************************

impl Circuit {
    // ********************************************************************************************
    // API
    // ********************************************************************************************

    pub fn get_level_per_signal(&self) -> UniqueSortedHashMap<Signal, u32> {
        let mut map = UniqueSortedHashMap::new_like(&self.nodes);

        for (s, node) in self.nodes.iter_pairs() {
            let get_level = |inputs: &[Signal]| {
                inputs
                    .iter()
                    .map(|x| map.get(x).unwrap())
                    .max()
                    .map(|x| *x + 1)
                    .unwrap_or(0)
            };

            let new_level = match &node.node_type {
                CircuitNodeType::And(a) => {
                    get_level(&a.inputs.iter().map(|x| x.signal()).collect::<Vec<Signal>>())
                }
                CircuitNodeType::GenericGate(g) => get_level(g.truth_table.get_signals().peek()),
                CircuitNodeType::ConstantZero => 0,
                CircuitNodeType::Input => 0,
                CircuitNodeType::Latch { .. } => 0,
            };

            map.insert(s, new_level);
        }

        map
    }

    /// Function that returns the number of times this signal was referenced.
    /// A reference is either:
    /// 1. Input of some gate.
    /// 2. Input to some latch.
    /// 3. An output.
    /// 4. A bad wire.
    /// 5. A constraint wire.
    ///
    /// Note: (2 - 5) are often called combinational outputs.
    pub fn get_number_of_references(&self) -> UniqueSortedHashMap<Signal, usize> {
        let users = self.get_users_per_signal();
        let mut result = UniqueSortedHashMap::new_like(&self.nodes);

        for signal in self.nodes.iter_sorted() {
            let refs = users.get(&signal).map(|x| x.len()).unwrap_or(0);
            result.insert(signal, refs);
        }

        for wire in self
            .outputs
            .iter()
            .chain(self.bad.iter())
            .chain(self.constraints.iter())
        {
            match result.get_mut(&wire.signal()) {
                Some(x) => *x += 1,
                None => {
                    result.insert(wire.signal(), 1);
                }
            }
        }

        result
    }

    pub fn get_users_per_signal(&self) -> UniqueSortedHashMap<Signal, UniqueSortedVec<Signal>> {
        let mut map: UniqueSortedHashMap<Signal, UniqueSortedVec<Signal>> =
            UniqueSortedHashMap::new_like(&self.nodes);

        for (s, node) in self.nodes.iter_pairs() {
            match &node.node_type {
                CircuitNodeType::And(a) => {
                    let mut last_push = None;
                    for x in a.inputs.iter() {
                        if Some(x.signal()) == last_push {
                            continue;
                        }
                        map.get_mut_or_add(&x.signal(), UniqueSortedVec::new)
                            .unwrap()
                            .push(s);
                        last_push = Some(x.signal());
                    }
                }
                CircuitNodeType::GenericGate(g) => {
                    g.truth_table.get_signals().peek().iter().for_each(|x| {
                        map.get_mut_or_add(x, UniqueSortedVec::new).unwrap().push(s);
                    });
                }
                CircuitNodeType::ConstantZero => {}
                CircuitNodeType::Input => {}
                CircuitNodeType::Latch(l) => {
                    map.get_mut_or_add(&l.input.signal(), UniqueSortedVec::new)
                        .unwrap()
                        .push(s);
                }
            };
        }

        map
    }

    pub fn get_internal_users_per_signal(
        &self,
    ) -> UniqueSortedHashMap<Signal, UniqueSortedVec<Signal>> {
        let mut map: UniqueSortedHashMap<Signal, UniqueSortedVec<Signal>> =
            UniqueSortedHashMap::new_like(&self.nodes);

        for (s, node) in self.nodes.iter_pairs() {
            match &node.node_type {
                CircuitNodeType::And(a) => {
                    let mut last_push = None;
                    for x in a.inputs.iter() {
                        if Some(x.signal()) == last_push {
                            continue;
                        }
                        map.get_mut_or_add(&x.signal(), UniqueSortedVec::new)
                            .unwrap()
                            .push(s);
                        last_push = Some(x.signal());
                    }
                }
                CircuitNodeType::GenericGate(g) => {
                    g.truth_table.get_signals().peek().iter().for_each(|x| {
                        map.get_mut_or_add(x, UniqueSortedVec::new).unwrap().push(s);
                    });
                }
                CircuitNodeType::ConstantZero => {}
                CircuitNodeType::Input => {}
                CircuitNodeType::Latch(_) => {}
            };
        }

        map
    }

    pub fn add_ground_if_possible(&mut self) {
        if !self.nodes.contains_key(&Signal::GROUND) {
            self.nodes.insert(
                Signal::GROUND,
                CircuitNode {
                    node_type: CircuitNodeType::ConstantZero,
                },
            );
        }
    }

    pub fn remove_ground_if_possible(&mut self) {
        if self.greatest_signal == Signal::GROUND {
            return;
        }

        if !self.nodes.contains_key(&Signal::GROUND) {
            return;
        }

        if self.important_signals.contains(&Signal::GROUND) {
            return;
        }

        let users = self.get_users_per_signal();
        if !users
            .get(&Signal::GROUND)
            .unwrap_or(&UniqueSortedVec::new())
            .is_empty()
        {
            return;
        }

        self.nodes.remove(&Signal::GROUND);
    }
}
