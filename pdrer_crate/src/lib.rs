//! Utilities for the creation and use of bit-level symbolic model checking algorithms.
//!
//! rust_formal_verification provides utilities to read AIGs, to convert them to
//! useful types such as finite state transition formulas, and some common algorithms.
//! This crate is for developing and prototyping algorithms for formal verification for hardware
//! devices. Algorithms like BMC, IC3, PDR and so on...
//!
//! # Quick Start
//!
//! To get you started quickly, all you need to do is read an .aig file.
//!
//! ```
//! use rust_formal_verification::models::{AndInverterGraph, FiniteStateTransitionSystem, Circuit};
//!
//! // read aig file:
//! let file_path = "tests/examples/hwmcc20/2019/goel/crafted/paper_v3/paper_v3.aig";
//! let aig = AndInverterGraph::from_aig_path(file_path).unwrap();
//!
//! // convert aig to internal circuit representation:
//! let mut circuit = Circuit::from_aig(&aig);
//!
//! // merge and gates
//! circuit.merge_and_gates();
//!
//! // create boolean logic formulas that represent aig:
//! let fin_state = FiniteStateTransitionSystem::new(&circuit, false).unwrap();
//!
//! // the formulas can then be read and turned to strings in DIMACS format.
//! assert_eq!(
//!     fin_state.get_initial_relation().to_cnf().to_string(),
//!     "p cnf 17 16\n-2 0\n-3 0\n-4 0\n-5 0\n-6 0\n-7 0\n-8 0\n-9 0\n-10 0\n-11 0\n-12 0\n-13 0\n-14 0\n-15 0\n-16 0\n-17 0"
//! );
//! ```

// ************************************************************************************************
// Forbid the use of unsafe in this crate
// ************************************************************************************************

#![deny(unsafe_code)]

// ************************************************************************************************
// rust submodule declaration, they get searched in their respective file  names
// ************************************************************************************************

pub mod engines; // requires existence of 'algorithms/mod.rs'
pub mod formulas; // requires existence of 'formulas/mod.rs'
pub mod models; // requires existence of 'models/mod.rs'
pub mod solvers; // requires existence of 'solvers/mod.rs'

// ************************************************************************************************
// re-exports of structs in these modules to simplify paths for other imports
// ************************************************************************************************
