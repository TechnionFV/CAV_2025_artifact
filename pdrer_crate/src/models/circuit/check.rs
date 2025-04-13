// ************************************************************************************************
// use
// ************************************************************************************************

use crate::models::circuit::node_types::CircuitNodeType;
use crate::models::{Signal, UniqueSortedHashMap, UniqueSortedVec, Utils};
// use std::collections::FxHashSet;
// use std::hash::Hash;

use super::Circuit;

// ************************************************************************************************
// impl
// ************************************************************************************************

impl Circuit {
    // ********************************************************************************************
    // helper functions
    // ********************************************************************************************

    fn check_gate(
        &self,
        signal: &Signal,
        users: &UniqueSortedHashMap<Signal, UniqueSortedVec<Signal>>,
        internal_users: &UniqueSortedHashMap<Signal, UniqueSortedVec<Signal>>,
        levels: &UniqueSortedHashMap<Signal, u32>,
    ) -> Result<(), String> {
        Utils::ensure(self.gates.contains(signal), "Signal does not exist")?;
        let node = self.nodes.get(signal).unwrap();
        let node_level = *levels.get(signal).unwrap();

        // Utils::ensure(
        //     1 + self.inputs.len() + self.latches.len() + self.inputs.len()
        //         <= signal.get_number()
        // );
        // Utils::ensure(
        //     signal.get_number()
        //         < 1 + self.inputs.len() + self.latches.len() + self.ands.len()
        // );
        let inputs = match &node.node_type {
            CircuitNodeType::And(a) => a.inputs.peek().clone(),
            CircuitNodeType::GenericGate(a) => a
                .truth_table
                .get_signals()
                .iter()
                .map(|s| s.wire(false))
                .collect(),
            _ => panic!("This should never happen."),
        };

        Utils::ensure(!inputs.is_empty(), "Gate with no inputs")?;
        Utils::ensure(
            Utils::is_sorted_and_unique(&inputs),
            "Gate inputs are not sorted or not unique.",
        )?;
        for x in inputs.iter() {
            Utils::ensure(
                self.nodes.contains_key(&x.signal()),
                "Gate input does not exist.",
            )?;
        }
        Utils::ensure(
            node_level
                == inputs
                    .iter()
                    .map(|x| levels.get(&x.signal()).unwrap())
                    .max()
                    .unwrap()
                    + 1,
            "Gate level is not correct.",
        )?;
        for x in inputs.iter() {
            Utils::ensure(&x.signal() < signal, "Gate input is not smaller than gate.")?;
            Utils::ensure(
                users.get(&x.signal()).unwrap().contains(signal),
                "Gate input does not use gate.",
            )?;
            Utils::ensure(
                internal_users.get(&x.signal()).unwrap().contains(signal),
                "Gate input does not use gate in internal users.",
            )?;
        }
        Ok(())
    }

    fn check_node(
        &self,
        signal: &Signal,
        users: &UniqueSortedHashMap<Signal, UniqueSortedVec<Signal>>,
        internal_users: &UniqueSortedHashMap<Signal, UniqueSortedVec<Signal>>,
        levels: &UniqueSortedHashMap<Signal, u32>,
    ) -> Result<(), String> {
        let node = self.nodes.get(signal).unwrap();
        let node_level = *levels.get(signal).unwrap();
        let binding = UniqueSortedVec::new();
        let node_users = users.get(signal).unwrap_or(&binding);
        let node_internal_users = internal_users.get(signal).unwrap_or(&binding);
        // Utils::ensure(signal, &node.id);

        match &node.node_type {
            CircuitNodeType::ConstantZero => {
                Utils::ensure(node_level == 0, "Ground level incorrect.")?;
                Utils::ensure(
                    signal.number() == 0,
                    "Constant zero give to signal other than zero.",
                )?;
            }
            CircuitNodeType::Input => {
                Utils::ensure(node_level == 0, "Input level incorrect.")?;
                Utils::ensure(
                    self.inputs.contains(signal),
                    "Inputs does not contain all inputs",
                )?;
                Utils::ensure(0 < signal.number(), "Input number too small.")?;
            }
            CircuitNodeType::Latch(l) => {
                Utils::ensure(self.latches.contains(signal), "")?;
                Utils::ensure(node_level == 0, "")?;
                Utils::ensure(
                    self.inputs.max().map(|s| s.number()).unwrap_or(0) < signal.number(),
                    "Latch signal is not greater than all of inputs.",
                )?;
                Utils::ensure(
                    self.nodes.contains_key(&l.input.signal()),
                    "Latch input does not exist.",
                )?;
                Utils::ensure(
                    users.get(&l.input.signal()).unwrap().contains(signal),
                    "Latch input does not use latch.",
                )?;
                Utils::ensure(
                    !internal_users
                        .get(&l.input.signal())
                        .unwrap_or(&UniqueSortedVec::new())
                        .contains(signal),
                    "Latch input does uses latch as internal signal.",
                )?;
                Utils::ensure(l.input.signal() <= self.get_highest_signal(), "")?;
            }
            CircuitNodeType::And(_) => self.check_gate(signal, users, internal_users, levels)?,
            CircuitNodeType::GenericGate(_) => {
                self.check_gate(signal, users, internal_users, levels)?
            }
        };

        // // check users
        // if !self.important_signals.contains(signal) && !signal.is_constant() {
        //     Utils::ensure(!node.users.is_empty(), "By the way we build the circuit (from AIG taking only important signals), this should never happen.");
        // }

        // Utils::ensure(
        //     Utils::has_unique_elements(&node.users),
        //     "Node has duplicate users."
        // );

        for x in node_users.iter() {
            Utils::ensure(self.nodes.contains_key(x), "User does not exist.")?;
        }

        Utils::ensure(
            node_internal_users.iter().all(|x| node_users.contains(x)),
            "Internal users is not a subset of users.",
        )?;

        for x in node_users.iter() {
            match &self.nodes.get(x).unwrap().node_type {
                CircuitNodeType::And(a) => {
                    Utils::ensure(
                        a.inputs.contains(&signal.wire(false))
                            || a.inputs.contains(&signal.wire(true)),
                        "Node user does not use node.",
                    )?;

                    Utils::ensure(
                        node_internal_users.contains(x),
                        "And gate uses node but is not in internal users of said node.",
                    )?;
                }
                CircuitNodeType::Latch(l) => {
                    Utils::ensure(&l.input.signal() == signal, "Node user does not use node.")?;
                    Utils::ensure(
                        !node_internal_users.contains(x),
                        "Latch uses node but is in internal users of said node.",
                    )?;
                }
                CircuitNodeType::GenericGate(g) => {
                    Utils::ensure(
                        g.truth_table.get_signals().contains(signal),
                        "Node user does not use node.",
                    )?;

                    Utils::ensure(
                        node_internal_users.contains(x),
                        "And gate uses node but is not in internal users of said node.",
                    )?;
                }
                _ => panic!(
                    "User is not an and gate and not a latch (How can it use another gate?)."
                ),
            }
        }
        Ok(())
    }

    // ********************************************************************************************
    // aig creator
    // ********************************************************************************************

    pub fn check(&self) -> Result<(), String> {
        // basic check
        Utils::ensure(
            self.nodes.len()
                == self.inputs.len()
                    + self.latches.len()
                    + self.gates.len()
                    + (if self.nodes.contains_key(&Signal::new(0)) {
                        1
                    } else {
                        0
                    }),
            "The number of variables does not add up.",
        )?;
        Utils::ensure(
            self.greatest_signal == self.nodes.max_key().unwrap_or(Signal::GROUND),
            "Max signal is not correct.",
        )?;
        // Utils::ensure(self.nodes.len(), self.greatest_signal.get_number() + 1); // this is only true if the graph is compressed

        if self.nodes.contains_key(&Signal::new(0)) {
            Utils::ensure(
                matches!(
                    self.nodes.get(&Signal::new(0)).unwrap().node_type,
                    CircuitNodeType::ConstantZero
                ),
                "Constant is not constant.",
            )?;
        }

        for x in self.gates.iter() {
            Utils::ensure(self.nodes.contains_key(x), "Gate does not exist.")?;
            Utils::ensure(
                matches!(
                    self.nodes.get(x).unwrap().node_type,
                    CircuitNodeType::And { .. }
                ) || matches!(
                    self.nodes.get(x).unwrap().node_type,
                    CircuitNodeType::GenericGate { .. }
                ),
                "Gate is not and nor generic.",
            )?;
        }

        for x in self.inputs.iter() {
            Utils::ensure(self.nodes.contains_key(x), "Input does not exist.")?;
            Utils::ensure(
                matches!(self.nodes.get(x).unwrap().node_type, CircuitNodeType::Input),
                "Input is not input.",
            )?;
        }

        for x in self.latches.iter() {
            Utils::ensure(self.nodes.contains_key(x), "Latch does not exist.")?;
            Utils::ensure(
                matches!(
                    self.nodes.get(x).unwrap().node_type,
                    CircuitNodeType::Latch { .. }
                ),
                "Latch is not latch.",
            )?;
        }

        for x in self.outputs.iter() {
            Utils::ensure(
                self.nodes.contains_key(&x.signal()),
                "Output does not exist.",
            )?;
        }
        for x in self.bad.iter() {
            Utils::ensure(self.nodes.contains_key(&x.signal()), "Bad does not exist.")?;
        }
        for x in self.constraints.iter() {
            Utils::ensure(
                self.nodes.contains_key(&x.signal()),
                "Constraint does not exist.",
            )?;
        }

        let important = self.recalculate_important_signals();
        Utils::ensure(
            self.important_signals == important,
            "Important signals are not correct.",
        )?;

        let users: UniqueSortedHashMap<Signal, UniqueSortedVec<Signal>> =
            self.get_users_per_signal();
        let internal_users: UniqueSortedHashMap<Signal, UniqueSortedVec<Signal>> =
            self.get_internal_users_per_signal();
        let levels: UniqueSortedHashMap<Signal, u32> = self.get_level_per_signal();

        // let signals: Vec<Signal> = .collect();
        for signal in self.nodes.iter_sorted() {
            self.check_node(&signal, &users, &internal_users, &levels)?;
        }

        Ok(())
    }
}
