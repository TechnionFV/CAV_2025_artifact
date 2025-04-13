// ************************************************************************************************
// use
// ************************************************************************************************

use super::FiniteStateTransitionSystem;
use crate::{formulas::Cube, models::UniqueSortedVec};
use rand::Rng;

// ************************************************************************************************
// impl
// ************************************************************************************************

impl FiniteStateTransitionSystem {
    pub fn get_random_state<R: Rng>(&self, rng: &mut R) -> Cube {
        let mut literals = Vec::new();
        for state_lit_num in self.state_variables.iter() {
            let n = state_lit_num.to_owned();
            let not = rng.gen();
            let new_lit = n.literal(not);
            literals.push(new_lit)
        }
        Cube::from_sequence(literals)
    }

    pub fn get_random_input<R: Rng>(&self, rng: &mut R) -> Cube {
        let mut literals = Vec::new();
        for input_lit_num in self.input_variables.iter() {
            let n = input_lit_num.to_owned();
            let not: bool = rng.gen();
            let new_lit = n.literal(not);
            literals.push(new_lit)
        }
        Cube::from_sequence(literals)
    }

    pub fn get_random_initial_state<R: Rng>(&self, rng: &mut R) -> Cube {
        let mut initial_state = self
            .get_initial_relation()
            .to_owned()
            .unpack()
            .unpack()
            .unpack();
        let initial_vars = initial_state.iter().map(|l| l.variable()).collect();
        let init = UniqueSortedVec::from_sequence(initial_vars);
        let vars_to_add = self.state_variables.subtract(&init);
        for var in vars_to_add.iter() {
            let n = var.to_owned();
            let not: bool = rng.gen();
            let new_lit = n.literal(not);
            initial_state.push(new_lit)
        }
        Cube::from_sequence(initial_state)
    }
}
