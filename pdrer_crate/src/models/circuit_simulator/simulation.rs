// ************************************************************************************************
// use
// ************************************************************************************************

use super::CircuitSimulator;
use crate::{
    function,
    models::{
        circuit_simulator::SimulationGate, ternary_value::TernaryValue,
        time_stats::function_timer::FunctionTimer, Signal, TruthTable, UniqueSortedVec, Wire,
    },
};
use std::{cmp::min, collections::BinaryHeap};

// ************************************************************************************************
// propagation strategy
// ************************************************************************************************

// pub enum PropagationStrategy {
//     FullSimulation,
//     UsingUsersWithVector,
//     UsingUsersWithBinaryHeap,
// }

macro_rules! timer {
    ($self: ident) => {
        if false {
            let _timer = FunctionTimer::start(function!(), ($self).time_stats.clone());
        }
    };
}

// ************************************************************************************************
// impl
// ************************************************************************************************

impl CircuitSimulator {
    // ********************************************************************************************
    // helper functions
    // ********************************************************************************************

    const NOT: [TernaryValue; 3] = [TernaryValue::X, TernaryValue::True, TernaryValue::False];
    fn ternary_not(v: TernaryValue) -> TernaryValue {
        Self::NOT[v as usize]
    }

    fn get_wire_value(&self, wire: &Wire) -> TernaryValue {
        let index = wire.get_signal_number() as usize;
        if wire.is_negated() {
            Self::ternary_not(self.simulation_state.get(index))
        } else {
            self.simulation_state.get(index)
        }
    }

    const SEEN: [TernaryValue; 2] = [TernaryValue::True, TernaryValue::X];
    fn get_ternary_and_result(&self, inputs: &[Wire]) -> TernaryValue {
        let mut x_seen = false;
        for input in inputs.iter().rev() {
            let input_value = self.get_wire_value(input);
            if input_value == TernaryValue::False {
                return TernaryValue::False;
            }
            x_seen |= input_value == TernaryValue::X;
        }

        Self::SEEN[x_seen as usize]

        // TernaryValue::X
    }

    fn get_ternary_tt_result(&self, tt: &TruthTable) -> TernaryValue {
        let input_values = tt
            .get_signals()
            .iter()
            .map(|x| self.simulation_state.get(x.number() as usize));

        tt.get_ternary_result(input_values)
    }

    // ********************************************************************************************
    // propagate 1
    // ********************************************************************************************

    /// Propagates the state of the circuit using full simulation.
    /// Short circuit allows a simple check to perform after enough iterations with no
    /// change to the simulation state.
    ///
    /// Warning: short circuit could result in incorrect results when generic gates with constant
    /// truth tables are used since the simulation state need no be updated prior to that gate.
    pub(super) fn propagate_change_using_full_simulation(&mut self, short_circuit: bool) {
        timer!(self);

        if self.first_gate.is_none() {
            return;
        }

        let start = self.first_gate.unwrap().number() as usize;

        let mut consecutive_iteration_with_no_change = 0;
        for i in start..self.gates.len() {
            if short_circuit {
                consecutive_iteration_with_no_change += 1;
                if consecutive_iteration_with_no_change > self.max_delta_between_node_and_its_users
                {
                    break;
                }
            }

            let node = match &self.gates[i] {
                Some(a) => a,
                None => continue,
            };

            let new_value = match &node {
                SimulationGate::And(a) => self.get_ternary_and_result(a.peek()),
                SimulationGate::Generic(tt) => self.get_ternary_tt_result(tt),
            };

            if new_value != self.simulation_state.get(i) {
                self.simulation_state.set(i, new_value);
                consecutive_iteration_with_no_change = 0;
            }
        }
    }

    // ********************************************************************************************
    // propagate 2
    // ********************************************************************************************

    fn initialize_bitmap(
        &self,
        signals_that_changed: &[Signal],
        signals_to_visit_mask: &mut [bool],
    ) -> (usize, usize) {
        timer!(self);
        let mut min_i = self.gates.len();
        let mut max_i = 0;
        let binding = UniqueSortedVec::new();
        for signal in signals_that_changed.iter() {
            let users = self._internal_users.get(signal).unwrap_or(&binding);
            for user in users.iter() {
                signals_to_visit_mask[user.number() as usize] = true;
            }
            if let Some(max) = users.max() {
                max_i = (max_i).max(max.number() as usize);
            }
            if let Some(min) = users.min() {
                min_i = (min_i).min(min.number() as usize);
            }
        }
        (min_i, max_i)
    }

    fn update_bitmap(&self, i: usize, signals_to_visit_mask: &mut [bool], max_i: &mut usize) {
        timer!(self);

        let signal = Signal::new(i as u32);
        let binding = UniqueSortedVec::new();
        let users = &self._internal_users.get(&signal).unwrap_or(&binding);
        debug_assert!(users.iter().all(|x| &signal < x));
        for user in users.iter() {
            signals_to_visit_mask[user.number() as usize] = true;
        }
        if let Some(max) = users.max() {
            *max_i = (*max_i).max(max.number() as usize);
        }
    }

    fn propagate_change_on_cone(
        &mut self,
        cone: &UniqueSortedVec<Signal>,
        signals_that_changed: &[Signal],
    ) {
        timer!(self);

        let mut signals_to_visit_mask = vec![false; self.gates.len() + 1];

        let (min_i, mut max_i) =
            self.initialize_bitmap(signals_that_changed, &mut signals_to_visit_mask);

        debug_assert!(signals_to_visit_mask[..min_i].iter().all(|x| !*x));

        let start_position = match cone.peek().binary_search(&Signal::new(min_i as u32)) {
            Ok(x) => x,
            Err(x) => x,
        };

        for s in cone.iter().skip(start_position) {
            let i = s.number() as usize;
            if i > max_i {
                break;
            }

            if !signals_to_visit_mask[i] {
                continue;
            }

            let node = self.gates[i].as_ref().unwrap();

            let new_value = match &node {
                SimulationGate::And(a) => {
                    let new_and_value = self.get_ternary_and_result(a.peek());
                    new_and_value
                }
                SimulationGate::Generic(tt) => self.get_ternary_tt_result(tt),
            };

            let old_value = self.simulation_state.get(i);

            if new_value != old_value {
                self.simulation_state.set(i, new_value);
                self.update_bitmap(i, &mut signals_to_visit_mask, &mut max_i);
            }
        }
    }

    // ********************************************************************************************
    // other
    // ********************************************************************************************

    // fn propagate_change_using_users_with_binary_heap(&mut self, signals_that_changed: &[Signal]) {
    //     // heap for faster execution
    //     let mut signals_we_need_to_update: BinaryHeap<Reverse<Signal>> =
    //         BinaryHeap::with_capacity(self.get_highest_signal().number());

    //     // update which and gates need to be updated
    //     for signal in signals_that_changed.iter() {
    //         // if the signal that changed is not in the cone of the wires we care about then it will not be in forward flow.
    //         // that means that it does not affect the cone
    //         // add these gates to the heap
    //         for node in self.nodes.get(signal).unwrap().users.iter() {
    //             signals_we_need_to_update.push(Reverse(*node));
    //         }
    //     }
    //     let mut stage = 0;

    //     // while there is still some gate that needs to be updated
    //     while !signals_we_need_to_update.is_empty() {
    //         let signal = signals_we_need_to_update.pop().unwrap().0;

    //         // remove duplicates
    //         while signals_we_need_to_update.peek().is_some()
    //             && signals_we_need_to_update.peek().unwrap().0 == signal
    //         {
    //             signals_we_need_to_update.pop();
    //         }

    //         // sanity check
    //         debug_assert!(!signal.is_constant());
    //         if self.inputs.contains(&signal) {
    //             debug_assert_eq!(stage, 0);
    //         } else if self.latches.contains(&signal) {
    //             debug_assert!(stage <= 1, "latch in the middle of the circuit");
    //             stage = 1;
    //         } else if self.gates.contains(&signal) {
    //             debug_assert!(stage <= 2);
    //             stage = 2;
    //         } else {
    //             unreachable!();
    //         }

    //         // get the node
    //         let node = self.nodes.get(&signal).unwrap();

    //         // sanity check
    //         // debug_assert_eq!(signal, node.id);

    //         let (old_value, new_value) = match &node.node_type {
    //             CircuitNodeType::And(a) => {
    //                 let new_and_value = self.get_ternary_and_result(a.inputs.peek());
    //                 let old_and_value = self.simulation_state[signal.number()];
    //                 (old_and_value, new_and_value)
    //             }
    //             CircuitNodeType::GenericGate(g) => {
    //                 debug_assert!(Utils::is_sorted(g.inputs.peek()));
    //                 debug_assert!(&g.inputs.iter().all(|x| !x.is_negated())); // none should be negated
    //                 let input_values = g
    //                     .inputs
    //                     .iter()
    //                     .map(|x| self.simulation_state[x.get_signal_number()]);
    //                 let new_and_value = g.truth_table.get_ternary_result(input_values);
    //                 let old_and_value = self.simulation_state[signal.number()];
    //                 (old_and_value, new_and_value)
    //             }
    //             _ => {
    //                 let new_value = self.simulation_state[signal.number()];
    //                 (new_value, new_value)
    //             }
    //         };

    //         if new_value != old_value {
    //             // update the simulation state
    //             self.simulation_state[signal.number()] = new_value;
    //             // add affected gates to the heap
    //             let users = &self.nodes.get(&signal).unwrap().users;
    //             // skip next latches when reached
    //             for gate in users.iter().filter(|x| &signal < x) {
    //                 // debug_assert!(i < gate.get_number(), "recalculating same gate twice");
    //                 signals_we_need_to_update.push(Reverse(*gate));
    //             }
    //         }
    //     }
    // }

    // fn propagate_change(
    //     &mut self,
    //     _signals_that_changed: &[Signal],
    //     strategy: PropagationStrategy,
    // ) {
    //     match strategy {
    //         PropagationStrategy::FullSimulation => self.propagate_change_using_full_simulation(),
    //         _ => todo!(), // PropagationStrategy::UsingUsersWithVector => {
    //                       //     self.propagate_change_using_users_with_vector(signals_that_changed)
    //                       // }
    //                       // PropagationStrategy::UsingUsersWithBinaryHeap => {
    //                       //     self.propagate_change_using_users_with_binary_heap(signals_that_changed)
    //                       // }
    //     }
    // }

    fn clear_simulation_state(&mut self) {
        timer!(self);

        self.simulation_state
            .clone_from_slice(&self.cleared_simulation_state);
    }

    fn is_result_correct(
        &mut self,
        signals_we_want_to_drop: &[(Signal, TernaryValue)],
        static_signals: &[(Signal, TernaryValue)],
        result: &[(Signal, TernaryValue)],
        signals_to_not_change: &[Signal],
    ) -> bool {
        timer!(self);

        let before = {
            let signals = [signals_we_want_to_drop, static_signals].concat();
            self.full_simulation(signals);
            self.get_signal_simulation_values(signals_to_not_change)
        };

        let after = {
            let signals = [result, static_signals].concat();
            self.full_simulation(signals);
            self.get_signal_simulation_values(signals_to_not_change)
        };
        // if before != after {
        //     println!("xx before = {:?}", before);
        //     println!("xx after = {:?}", after);
        // }
        // debug_assert_eq!(before, after);
        before == after
    }

    // ********************************************************************************************
    // aig getting and gates
    // ********************************************************************************************

    pub fn put_values<I>(&mut self, signals_and_new_values: I) -> Vec<Signal>
    where
        I: IntoIterator<Item = (Signal, TernaryValue)>,
    {
        timer!(self);

        let mut changed_signals = Vec::new();

        // go over the changes
        for (s, v) in signals_and_new_values {
            let signal_number = s.number() as usize;
            if self.simulation_state.get(signal_number) != v {
                // mark that this signal changed
                self.simulation_state.set(signal_number, v);
                changed_signals.push(s);
            }
        }

        changed_signals
    }

    /// This function is used to simulate the circuit in its entirety, this is used when
    /// the simulation parameters (values ont latches and inputs) have changed dramatically.
    pub fn full_simulation<I>(&mut self, simulation_values: I)
    where
        I: IntoIterator<Item = (Signal, TernaryValue)>,
    {
        timer!(self);

        self.clear_simulation_state();

        // avoid simulation if there is nothing to simulate.
        let c = self.put_values(simulation_values);
        if c.is_empty() {
            return;
        }

        self.propagate_change_using_full_simulation(true);
    }

    // /// This function is used to change the simulation state of the circuit. It is used when
    // /// a previous simulation was done and the changed signals are few. It uses some
    // /// strategy to propagate the changes, and the strategy is given as an argument.
    // pub fn change_simulation_signals(&mut self, strategy: PropagationStrategy) {
    //     // prepare space to mark changed literals

    //     // propagate the changes
    //     self.propagate_change(&changed_signals, strategy);
    // }

    /// read a simulation value of a signal.
    pub fn get_signal_simulation_values(&self, signals: &[Signal]) -> Vec<TernaryValue> {
        timer!(self);
        let mut result = Vec::with_capacity(signals.len());
        for signal in signals.iter() {
            result.push(self.simulation_state.get(signal.number() as usize));
        }
        result
    }

    fn get_cone(&self, signals: &[Signal]) -> UniqueSortedVec<Signal> {
        timer!(self);

        let mut result = Vec::with_capacity(signals.len());
        let mut queue = BinaryHeap::from(signals.to_vec());
        while let Some(signal) = queue.pop() {
            while queue.peek().is_some() && queue.peek().unwrap() == &signal {
                queue.pop();
            }
            debug_assert!(!result.contains(&signal));
            result.push(signal);

            if let Some(gate) = self.gates[signal.number() as usize].as_ref() {
                match &gate {
                    SimulationGate::And(a) => {
                        queue.extend(a.iter().map(|w| w.signal()));
                    }
                    SimulationGate::Generic(tt) => {
                        queue.extend(tt.get_signals().peek().iter().copied());
                    }
                }
            }
        }
        result.reverse();
        UniqueSortedVec::from_ordered_set(result)
    }

    pub fn automatic_simulation_for_dropping_signals(
        &mut self,
        signals_we_want_to_drop: &[(Signal, TernaryValue)],
        static_signals: &[(Signal, TernaryValue)],
        signals_to_not_change: &[Signal],
    ) -> Vec<(Signal, TernaryValue)> {
        timer!(self);

        let initial_signals = [signals_we_want_to_drop, static_signals].concat();

        let cone = self.get_cone(signals_to_not_change);
        self.clear_simulation_state();
        let c = self.put_values(initial_signals);
        self.propagate_change_on_cone(&cone, &c);

        // self.full_simulation(initial_signals);

        let initial = self.get_signal_simulation_values(signals_to_not_change);
        // let cone = self.get_cone(signals_to_not_change);

        // create backup
        let mut result = Vec::with_capacity(signals_we_want_to_drop.len());

        // create vector to house result
        let mut state_before = self.simulation_state.to_owned();

        // make a window to drop at the same time
        let mut window_size = 1;

        let mut literals_since_last_failure = 0;

        // go over the signals
        let mut i = 0;
        while i < signals_we_want_to_drop.len() {
            let current_drop =
                &signals_we_want_to_drop[i..min(i + window_size, signals_we_want_to_drop.len())];
            // ternary simulate circuit
            // self.clear_simulation_state();
            // println!("before state = {:?}", self.simulation_state);
            let new_wires = {
                let c = self.put_values(
                    current_drop
                        .iter()
                        .map(|(s, _)| (s.to_owned(), TernaryValue::X)),
                );
                self.propagate_change_on_cone(&cone, &c);
                self.get_signal_simulation_values(signals_to_not_change)
            };
            // println!("after state = {:?}", self.simulation_state);

            if new_wires != initial {
                // remove failed undo change in simulation
                literals_since_last_failure = 0;
                self.simulation_state.clone_from_slice(&state_before);
                if window_size == 1 {
                    // this literal cannot be dropped
                    debug_assert_eq!(current_drop.len(), 1);
                    result.push((current_drop[0].0, current_drop[0].1));
                    i += 1;
                } else {
                    // set window size to one and try again
                    debug_assert!(window_size > 1);
                    window_size = 1;
                }
            } else {
                // remove successful, update backup
                state_before.clone_from_slice(&self.simulation_state);
                literals_since_last_failure += window_size;
                i += window_size;
                if literals_since_last_failure > 5 {
                    // add to window size
                    window_size += 1;
                }
            }
        }
        debug_assert!(self.is_result_correct(
            signals_we_want_to_drop,
            static_signals,
            &result,
            signals_to_not_change
        ));
        result
    }

    pub fn print_time_stats(&self) {
        println!("{}", self.time_stats.borrow());
    }
}

// ************************************************************************************************
// test
// ************************************************************************************************

#[test]
fn test_simulation() {}
