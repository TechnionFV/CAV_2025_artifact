// ************************************************************************************************
// use
// ************************************************************************************************

use super::FiniteStateTransitionSystem;
use crate::formulas::Variable;
use crate::formulas::{Clause, Cube, Literal, CNF};
use crate::models::SortedVecOfLiterals;

// ************************************************************************************************
// impl
// ************************************************************************************************

impl FiniteStateTransitionSystem {
    // ********************************************************************************************
    // helper functions
    // ********************************************************************************************

    // ********************************************************************************************
    // api - checking variables
    // ********************************************************************************************

    pub fn is_input_variable(&self, var: Variable) -> bool {
        self.input_variables.contains(&var)
    }

    pub fn is_state_variable(&self, var: Variable) -> bool {
        self.state_variables.contains(&var)
    }

    // ********************************************************************************************
    // api - checking literals
    // ********************************************************************************************

    pub fn is_input_literal(&self, lit: &Literal) -> bool {
        self.is_input_variable(lit.variable())
    }

    pub fn is_state_literal(&self, lit: &Literal) -> bool {
        self.is_state_variable(lit.variable())
    }

    // ********************************************************************************************
    // api - adding tags
    // ********************************************************************************************

    /// number  of tags to add to each variable (negative number is allowed, it removes tags)
    pub fn add_tags_to_relation(&self, relation: &mut CNF, number_of_tags: i32) {
        if number_of_tags != 0 {
            let delta = (self.max_variable.number() as i32) * number_of_tags;
            relation.bump_all_literals(delta);
        }
    }

    pub fn add_tags_to_cube(&self, cube: &mut Cube, number_of_tags: i32) {
        if number_of_tags != 0 {
            let delta = (self.max_variable.number() as i32) * number_of_tags;
            cube.bump_all_literals(delta);
        }
    }

    pub fn add_tags_to_clause(&self, clause: &mut Clause, number_of_tags: i32) {
        if number_of_tags != 0 {
            let delta = (self.max_variable.number() as i32) * number_of_tags;
            clause.bump_all_literals(delta);
        }
    }

    pub fn add_tags_to_literal(&self, literal: &mut Literal, number_of_tags: i32) {
        if number_of_tags != 0 {
            let delta = (self.max_variable.number() as i32) * number_of_tags;
            literal.bump(delta);
        }
    }

    pub fn add_tags_to_variable(&self, variable: &mut Variable, number_of_tags: i32) {
        if number_of_tags != 0 {
            let delta = (self.max_variable.number() as i32) * number_of_tags;
            variable.bump(delta);
        }
    }

    // ********************************************************************************************
    // api - checks
    // ********************************************************************************************

    /// * return true if cube contains no contradiction with initial state, **this cube can still not be possible under constraints**.
    /// * return false if cube contains a literal that contradicts all initial states.
    /// * return None if clause contains a literal that is not a latch, and so we cannot know for sure.
    pub fn is_cube_satisfied_by_some_initial_state(&self, cube: &Cube) -> Option<bool> {
        // check that cube contains no contradiction with initial.
        let cube_negation = !cube.to_owned();
        self.is_clause_satisfied_by_all_initial_states(&cube_negation)
            .map(|x| !x)
    }

    /// * return true if clause is guaranteed to be satisfied by all initial states.
    /// * return false if there exists an initial state that does not satisfy the clause, **this state can still not be possible under constraints**.
    /// * return None if clause contains a literal that is not a latch, and so we cannot know for sure.
    pub fn is_clause_satisfied_by_all_initial_states(&self, clause: &Clause) -> Option<bool> {
        let intersection = clause
            .peek()
            .peek()
            .intersect(self.initial_states.peek().peek());

        if intersection.is_empty() {
            // maybe there is an internal signal that is always present in internal states
            if clause.iter().all(|l| self.is_state_literal(l)) {
                Some(false)
            } else {
                None
            }
        } else {
            // found literal in clause that is always present in initial states
            Some(true)
        }
    }

    // ********************************************************************************************
    // api - Nice representation of cubes
    // ********************************************************************************************

    /// Function that gets a nice way to represent cubes that are on state variables
    pub fn represent_cube_on_states(&self, literals: &SortedVecOfLiterals) -> String {
        self.get_state_variables()
            .iter()
            .map(|x| {
                if literals.contains(&x.literal(false)) {
                    "1"
                } else if literals.contains(&x.literal(true)) {
                    "0"
                } else {
                    "-"
                }
            })
            .collect::<Vec<_>>()
            .join("")
    }

    // ********************************************************************************************
    // api - complete initial cube
    // ********************************************************************************************

    // /// Takes an initial cube that does not contain literals for all latch variables and
    // /// completes it so it does.
    // pub fn complete_initial_cube(&self, initial_cube: Cube) -> Cube {
    //     debug_assert!(self.is_cube_initial(&initial_cube).unwrap());

    //     let initial_cube_with_more_variables = Utils::merge_unique_sorted_vectors(
    //         initial_cube.unpack().unpack(),
    //         self.get_initial_relation().unpack().unpack(),
    //     );

    //     let range = self.get_latch_variable_range();
    //     let mut complete_initial_cube = Vec::with_capacity(range.len());

    //     for v in self.latch_variable_to_its_input_variable.iter_sorted() {
    //         if initial_cube_with_more_variables
    //             .binary_search(&v.get_literal(false))
    //             .is_ok()
    //         {
    //             complete_initial_cube.push(v.get_literal(false));
    //         } else {
    //             complete_initial_cube.push(v.get_literal(true));
    //         }
    //     }
    //     let initial_complete_cube = Cube::new(complete_initial_cube);
    //     debug_assert!(self.is_cube_initial(&initial_complete_cube).unwrap());
    //     initial_complete_cube
    // }
}

// ************************************************************************************************
// tests
// ************************************************************************************************

#[test]
fn test_initial() {
    use crate::models::{AndInverterGraph, Circuit, Signal, TernaryValue};
    let aig = AndInverterGraph::new(
        Signal::new(3),
        0,
        &[
            (Signal::new(2).wire(false), TernaryValue::False),
            (Signal::new(1).wire(false), TernaryValue::True),
        ],
        vec![Signal::new(3).wire(false)],
        vec![],
        vec![],
        &[(Signal::new(1).wire(false), Signal::new(2).wire(false))],
        String::new(),
    )
    .unwrap();
    let mut circuit = Circuit::from_aig(&aig);
    circuit.remove_unused_signals();
    // Circuit:
    // ┌−−−−−−−−−−−−−−┐                   ┌−−−−−−−−−−−−−−−−−−−−−−−−−−−−−┐
    // ╎   Outputs    ╎                   ╎               Latches       ╎
    // ╎              ╎                   ╎                             ╎
    // ╎ ┌──────────┐ ╎     ┌───────┐     ╎ ┌───────────┐               ╎
    // ╎ │ Output 6 │ ╎ ◀── │ 3 And │ ◀── ╎ │ 1 Latch 0 │          ─┐   ╎
    // ╎ └──────────┘ ╎     └───────┘     ╎ └───────────┘           │   ╎
    // ╎              ╎                   ╎   ▲                     │   ╎
    // └−−−−−−−−−−−−−−┘                   ╎   │                     │   ╎
    //                        ▲           ╎   │                     │   ╎
    //                        │           ╎   │                     │   ╎
    //                        │           ╎   │                     │   ╎
    //                        │           ╎ ┌───────────┐           │   ╎
    //                        └────────── ╎ │ 2 Latch 1 │          ◀┘   ╎
    //                                    ╎ └───────────┘               ╎
    //                                    ╎                             ╎
    //                                    └−−−−−−−−−−−−−−−−−−−−−−−−−−−−−┘
    let fin_state = FiniteStateTransitionSystem::new(&circuit, true).unwrap();
    let x1 = Variable::new(1).literal(false);
    let x2 = Variable::new(2).literal(false);
    let x3 = Variable::new(3).literal(false);

    // is_cube_satisfied_by_some_initial_state
    // 1 literal
    assert!(!fin_state
        .is_cube_satisfied_by_some_initial_state(&Cube::from_sequence(vec![x1]))
        .unwrap());
    assert!(fin_state
        .is_cube_satisfied_by_some_initial_state(&Cube::from_sequence(vec![!x1]))
        .unwrap());
    assert!(fin_state
        .is_cube_satisfied_by_some_initial_state(&Cube::from_sequence(vec![x2]))
        .unwrap());
    assert!(!fin_state
        .is_cube_satisfied_by_some_initial_state(&Cube::from_sequence(vec![!x2]))
        .unwrap());

    // 2 literals
    assert!(!fin_state
        .is_cube_satisfied_by_some_initial_state(&Cube::from_sequence(vec![x1, x2]))
        .unwrap());
    assert!(!fin_state
        .is_cube_satisfied_by_some_initial_state(&Cube::from_sequence(vec![x1, !x2]))
        .unwrap());
    assert!(fin_state
        .is_cube_satisfied_by_some_initial_state(&Cube::from_sequence(vec![!x1, x2]))
        .unwrap());
    assert!(!fin_state
        .is_cube_satisfied_by_some_initial_state(&Cube::from_sequence(vec![!x1, !x2]))
        .unwrap());

    // 3 literals
    assert!(!fin_state
        .is_cube_satisfied_by_some_initial_state(&Cube::from_sequence(vec![x1, x2, x3]))
        .unwrap());
    assert!(!fin_state
        .is_cube_satisfied_by_some_initial_state(&Cube::from_sequence(vec![x1, x2, !x3]))
        .unwrap());
    assert!(!fin_state
        .is_cube_satisfied_by_some_initial_state(&Cube::from_sequence(vec![x1, !x2, x3]))
        .unwrap());
    assert!(!fin_state
        .is_cube_satisfied_by_some_initial_state(&Cube::from_sequence(vec![x1, !x2, !x3]))
        .unwrap());
    assert!(fin_state
        .is_cube_satisfied_by_some_initial_state(&Cube::from_sequence(vec![!x1, x2, x3]))
        .is_none());
    assert!(fin_state
        .is_cube_satisfied_by_some_initial_state(&Cube::from_sequence(vec![!x1, x2, !x3]))
        .is_none());
    assert!(!fin_state
        .is_cube_satisfied_by_some_initial_state(&Cube::from_sequence(vec![!x1, !x2, x3]))
        .unwrap());
    assert!(!fin_state
        .is_cube_satisfied_by_some_initial_state(&Cube::from_sequence(vec![!x1, !x2, !x3]))
        .unwrap());

    // is clause satisfied by all initial states
    // 1 literal
    assert!(!fin_state
        .is_clause_satisfied_by_all_initial_states(&Clause::from_sequence(vec![x1]))
        .unwrap());
    assert!(fin_state
        .is_clause_satisfied_by_all_initial_states(&Clause::from_sequence(vec![!x1]))
        .unwrap());
    assert!(fin_state
        .is_clause_satisfied_by_all_initial_states(&Clause::from_sequence(vec![x2]))
        .unwrap());
    assert!(!fin_state
        .is_clause_satisfied_by_all_initial_states(&Clause::from_sequence(vec![!x2]))
        .unwrap());
}
