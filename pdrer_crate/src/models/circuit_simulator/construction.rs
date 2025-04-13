// ************************************************************************************************
// use
// ************************************************************************************************

use super::{Circuit, CircuitSimulator, SimulationGate, TernaryValueVector};
use crate::models::{
    circuit::node_types::{CircuitNode, CircuitNodeType},
    TernaryValue,
};

// ************************************************************************************************
// impl
// ************************************************************************************************

impl CircuitSimulator {
    fn convert_circuit_node_to_simulator_gate(node: &CircuitNode) -> Option<SimulationGate> {
        match &node.node_type {
            CircuitNodeType::And(a) => Some(SimulationGate::And(a.inputs.clone())),
            CircuitNodeType::GenericGate(tt) => {
                Some(SimulationGate::Generic(tt.truth_table.clone()))
            }

            _ => None,
        }
    }

    fn clear_simulation_state_slow(&mut self) {
        self.simulation_state.clear();

        // set constant
        self.simulation_state.set(0, TernaryValue::False);

        self.propagate_change_using_full_simulation(false);
    }

    // ********************************************************************************************
    // API
    // ********************************************************************************************

    pub fn new(circuit: &Circuit) -> Self {
        let length = circuit.get_highest_signal().number() as usize + 1;
        let mut gates = vec![None; length];
        let mut max_delta_between_node_and_its_users = 0;
        let mut first_gate = None;
        let internal_users = circuit.get_internal_users_per_signal();

        for signal in circuit.iter_sorted() {
            let node = circuit.get_node(&signal).unwrap();
            let value = Self::convert_circuit_node_to_simulator_gate(node);
            let value = match value {
                None => continue,
                Some(a) => a,
            };
            if first_gate.is_none() {
                first_gate = Some(signal);
            }

            let s = signal.number() as isize;
            let furthest_used = match &value {
                SimulationGate::And(a) => {
                    a.iter().map(|w| w.signal().number()).min().unwrap() as isize
                }
                SimulationGate::Generic(g) => {
                    g.get_signals().iter().map(|s| s.number()).min().unwrap() as isize
                }
            };

            let delta = (s - furthest_used) as usize;

            max_delta_between_node_and_its_users =
                std::cmp::max(max_delta_between_node_and_its_users, delta);
            gates[signal.number() as usize] = Some(value);
        }

        let mut r = Self {
            simulation_state: TernaryValueVector::new(length),
            cleared_simulation_state: TernaryValueVector::new(length),
            gates,
            _internal_users: internal_users,
            first_gate,
            max_delta_between_node_and_its_users,
            time_stats: Default::default(),
        };

        r.clear_simulation_state_slow();
        r.cleared_simulation_state
            .clone_from_slice(&r.simulation_state);

        r
    }
}
