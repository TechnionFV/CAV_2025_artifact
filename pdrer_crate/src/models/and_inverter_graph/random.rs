// ************************************************************************************************
// use
// ************************************************************************************************

use crate::models::and_inverter_graph::AndInverterGraph;
use crate::models::Signal;
use crate::models::TernaryValue;
use crate::models::Wire;
use rand::seq::SliceRandom;
use rand::Rng;

// ************************************************************************************************
// impl
// ************************************************************************************************

impl AndInverterGraph {
    // ********************************************************************************************
    // helper functions
    // ********************************************************************************************

    fn choose_random_wire<R: Rng>(rng: &mut R, max_signal: Signal) -> Wire {
        Signal::new(rng.gen_range(0..=max_signal.number())).wire(rng.gen())
    }

    fn choose_random_wire_below_signal<R: Rng>(rng: &mut R, s: &Signal) -> Wire {
        let signal_number = rng.gen_range(0..s.number());
        let is_negated: bool = rng.gen();
        Signal::new(signal_number).wire(is_negated)
    }

    fn choose_random_wire_below_signal_with_geometric_destribution<R: Rng>(
        rng: &mut R,
        s: &Signal,
    ) -> Wire {
        let start_value = s.number() - 1;
        let mut signal_number = start_value;
        loop {
            if rng.gen_bool(0.125) {
                break;
            }
            if signal_number == 0 {
                signal_number = start_value
            } else {
                signal_number -= 1
            }
        }
        let is_negated: bool = rng.gen();
        Signal::new(signal_number).wire(is_negated)
    }

    fn choose_random_ternary_value<R: Rng>(rng: &mut R) -> TernaryValue {
        [TernaryValue::False, TernaryValue::True, TernaryValue::X]
            .choose(rng)
            .unwrap()
            .to_owned()
    }

    // ********************************************************************************************
    // aig creator
    // ********************************************************************************************

    /// Create a new random AndInverterGraph, where each input or value is chosen uniformally over
    /// the range of possible values. For example, the input of an and gate is chosen uniformally
    /// over the wires that are available (smaller signal than that and gate).
    pub fn uniform_random<R: Rng>(
        rng: &mut R,
        number_of_inputs: usize,
        number_of_latches: usize,
        number_of_and_gates: usize,
        number_of_outputs: usize,
        number_of_bad_wires: usize,
        number_of_constraint_wires: usize,
    ) -> AndInverterGraph {
        let max_signal =
            Signal::new((number_of_inputs + number_of_latches + number_of_and_gates) as u32);

        let latches = {
            let mut latches = Vec::with_capacity(number_of_latches);
            for _ in 0..number_of_latches {
                latches.push((
                    Self::choose_random_wire(rng, max_signal),
                    Self::choose_random_ternary_value(rng),
                ))
            }
            latches
        };

        let and_gates = {
            let mut and_gates = Vec::with_capacity(number_of_and_gates);
            for i in 0..number_of_and_gates {
                let s = Signal::new((number_of_inputs + number_of_latches + i + 1) as u32);
                and_gates.push((
                    Self::choose_random_wire_below_signal(rng, &s),
                    Self::choose_random_wire_below_signal(rng, &s),
                ))
            }
            and_gates
        };

        Self::new(
            max_signal,
            number_of_inputs as u32,
            &latches,
            (0..number_of_outputs)
                .map(|_| Self::choose_random_wire(rng, max_signal))
                .collect(),
            (0..number_of_bad_wires)
                .map(|_| Self::choose_random_wire(rng, max_signal))
                .collect(),
            (0..number_of_constraint_wires)
                .map(|_| Self::choose_random_wire(rng, max_signal))
                .collect(),
            &and_gates,
            String::new(),
        )
        .unwrap()
    }

    pub fn geometric_random<R: Rng>(
        rng: &mut R,
        number_of_inputs: usize,
        number_of_latches: usize,
        number_of_and_gates: usize,
        number_of_outputs: usize,
        number_of_bad_wires: usize,
        number_of_constraint_wires: usize,
    ) -> AndInverterGraph {
        let max_signal =
            Signal::new((number_of_inputs + number_of_latches + number_of_and_gates) as u32);

        let latches = {
            let mut latches = Vec::with_capacity(number_of_latches);
            for _ in 0..number_of_latches {
                latches.push((
                    Self::choose_random_wire(rng, max_signal),
                    Self::choose_random_ternary_value(rng),
                ))
            }
            latches
        };

        let and_gates = {
            let mut and_gates = Vec::with_capacity(number_of_and_gates);
            for i in 0..number_of_and_gates {
                let s = Signal::new((number_of_inputs + number_of_latches + i + 1) as u32);
                and_gates.push((
                    Self::choose_random_wire_below_signal_with_geometric_destribution(rng, &s),
                    Self::choose_random_wire_below_signal_with_geometric_destribution(rng, &s),
                ))
            }
            and_gates
        };

        Self::new(
            max_signal,
            number_of_inputs as u32,
            &latches,
            (0..number_of_outputs)
                .map(|_| Self::choose_random_wire(rng, max_signal))
                .collect(),
            (0..number_of_bad_wires)
                .map(|_| Self::choose_random_wire(rng, max_signal))
                .collect(),
            (0..number_of_constraint_wires)
                .map(|_| Self::choose_random_wire(rng, max_signal))
                .collect(),
            &and_gates,
            String::new(),
        )
        .unwrap()
    }
}
