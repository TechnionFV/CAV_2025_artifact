// ************************************************************************************************
// use
// ************************************************************************************************

use super::FiniteStateTransitionSystem;
use crate::formulas::{Clause, Variable};
use crate::formulas::{Cube, CNF};
use crate::models::UniqueSortedVec;

// ************************************************************************************************
// impl
// ************************************************************************************************

impl FiniteStateTransitionSystem {
    // ********************************************************************************************
    // API
    // ********************************************************************************************

    pub fn get_max_variable(&self) -> Variable {
        self.max_variable
    }

    pub fn get_state_variables(&self) -> &UniqueSortedVec<Variable> {
        &self.state_variables
    }

    pub fn get_input_variables(&self) -> &UniqueSortedVec<Variable> {
        &self.input_variables
    }

    // ********************************************************************************************
    // API
    // ********************************************************************************************

    pub fn get_initial_relation(&self) -> &Cube {
        &self.initial_states
    }

    // ********************************************************************************************
    // API - on internals
    // ********************************************************************************************

    /// Function that gets the transition relation for the FiniteStateTransitionSystem.
    ///
    /// # Arguments
    ///
    /// * `self: &FiniteStateTransitionSystem` - the FiniteStateTransitionSystem desired.
    ///
    /// # Examples
    /// ```
    /// use rust_formal_verification::models::{Signal, AndInverterGraph, Wire, TernaryValue, Circuit, FiniteStateTransitionSystem};
    /// use rust_formal_verification::formulas::CNF;
    ///
    /// let aig = AndInverterGraph::new(
    ///     Signal::new(5),
    ///     0,
    ///     &[
    ///         (Signal::new(5).wire(false), TernaryValue::False),
    ///         (Signal::new(1).wire(false), TernaryValue::False),
    ///         (Signal::new(2).wire(false), TernaryValue::False),
    ///     ],
    ///     vec![Signal::new(5).wire(false)],
    ///     vec![],
    ///     vec![],
    ///     &[
    ///         (Signal::new(2).wire(true), Signal::new(3).wire(true)),
    ///         (
    ///             Signal::new(1).wire(true),
    ///             Signal::new(4).wire(false),
    ///         ),
    ///     ],
    ///     String::new(),
    /// )
    /// .unwrap();
    /// let circuit = Circuit::from_aig(&aig);
    /// let fsts = FiniteStateTransitionSystem::new(&circuit, false).unwrap();
    /// let mut tr_x_x_tag = CNF::from_sequence(vec![]);
    /// let tr_x_x_tag = fsts.get_transition_on_internals();
    /// assert_eq!(
    ///     tr_x_x_tag.to_string(),
    ///     "p cnf 8 6\n-5 6 0\n5 -6 0\n-1 7 0\n1 -7 0\n-2 8 0\n2 -8 0"
    /// );
    /// ```
    pub fn get_transition_on_internals(&self) -> &CNF {
        &self.transition_on_internals
    }

    pub fn get_property_on_internals(&self) -> &Cube {
        &self.property_on_internals
    }

    pub fn get_invariant_constraints_on_internals(&self) -> &Cube {
        &self.invariant_constraints_on_internals
    }

    // ********************************************************************************************
    // API - connectors
    // ********************************************************************************************

    pub fn get_transition_connector(&self) -> &CNF {
        &self.transition_connector
    }

    pub fn get_property_connector(&self) -> &CNF {
        &self.property_connector
    }

    pub fn get_invariant_constraints_connector(&self) -> &CNF {
        &self.invariant_constraints_connector
    }

    // ********************************************************************************************
    // API - get definition
    // ********************************************************************************************

    pub fn get_variable_definition(&self, variable: &Variable) -> Option<&Vec<Clause>> {
        self.variable_definitions.get(variable)
    }

    // ********************************************************************************************
    // API - get internal variable of state variable
    // ********************************************************************************************

    /// Given some state variable, this function returns the variable that represents the input
    /// to the latch that corresponds to that state variable, and whether or not it is negated
    pub fn get_internal_variable_in_cone_of_state_variable(
        &self,
        state_variable: &Variable,
    ) -> Option<&(Variable, bool)> {
        self.state_variable_to_its_internal_signal_variable
            .get(state_variable)
    }

    /// Takes state an returns state variables that would affect this state after one transition
    pub fn get_state_variables_in_cone_of_state(&self, state: &Cube) -> UniqueSortedVec<Variable> {
        let vectors = state.iter().map(|l| l.variable()).map(|v| {
            self.state_variable_to_state_variables_in_its_cone
                .get(&v)
                .unwrap()
        });
        UniqueSortedVec::k_merge(vectors, state.len())
    }
}
