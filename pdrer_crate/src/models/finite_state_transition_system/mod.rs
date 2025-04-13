//! Object for holding a finite state transition system, and performing operations like adding
//! tags and so on.

// ************************************************************************************************
// use
// ************************************************************************************************

use std::fmt::Debug;

use super::{
    definition::Definition, CircuitSimulator, Counterexample, Proof, Signal, UniqueSortedHashMap,
    UniqueSortedVec,
};
use crate::formulas::{Clause, Cube, Variable, CNF};

// ************************************************************************************************
// FiniteStateTransitionSystemError
// ************************************************************************************************

#[derive(Debug)]
pub enum FiniteStateTransitionSystemError {
    EmptyCircuit,
    MaxWireTooHigh,
    ConstraintWireIsConstantZero,
    ConstraintWiresIncludeWireAndItsNegation,
    BadWireIsConstantOne,
    BadWiresIncludeWireAndItsNegation,
}

// ************************************************************************************************
// ProofResult
// ************************************************************************************************

pub type ProofResult = Result<Proof, Counterexample>;

// ************************************************************************************************
// struct
// ************************************************************************************************

/// Struct that describes memory layout of the finite state transition system.
///
/// implementations of many additional features can be found in sub-modules.
pub struct FiniteStateTransitionSystem {
    signal_to_variable: Box<dyn Fn(Signal) -> Variable>,
    variable_to_signal: Box<dyn Fn(Variable) -> Signal>,

    // cube describing all initial states
    initial_states: Cube,

    // connecter between state literals and desired internal signal literals
    transition_connector: CNF,
    property_connector: CNF,
    invariant_constraints_connector: CNF,

    // desired properties
    invariant_constraints_on_internals: Cube,
    property_on_internals: Cube,
    transition_on_internals: CNF,

    // some meta data
    max_variable: Variable,
    variable_definitions: UniqueSortedHashMap<Variable, Vec<Clause>>,

    // i and x in S=<x, i, Tr(x, i, x'), P(x)>
    input_variables: UniqueSortedVec<Variable>,
    state_variables: UniqueSortedVec<Variable>,

    // for extraction
    state_variables_in_cone_of_safety: UniqueSortedVec<Variable>,
    state_variables_in_cone_of_invariant_constraint: UniqueSortedVec<Variable>,
    state_variables_in_cone_of_safety_and_invariant_constraint: UniqueSortedVec<Variable>,
    state_variable_to_its_internal_signal_variable: UniqueSortedHashMap<Variable, (Variable, bool)>,
    state_variable_to_state_variables_in_its_cone:
        UniqueSortedHashMap<Variable, UniqueSortedVec<Variable>>,

    // tri simulation
    circuit: CircuitSimulator,
    signals_to_implicate_on: UniqueSortedVec<Signal>,
}

impl Debug for FiniteStateTransitionSystem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "FiniteStateTransitionSystem")
    }
}

// ************************************************************************************************
// rust submodule declaration, they get searched in their respective file  names
// ************************************************************************************************

pub mod check;
pub mod cnf;
pub mod construction;
pub mod conversion;
pub mod extraction;
pub mod features;
pub mod getting;
pub mod inductive_subset;
pub mod random;
pub mod x_simulation;

// ************************************************************************************************
// re-exports of structs in these modules to simplify paths for other imports
// ************************************************************************************************
