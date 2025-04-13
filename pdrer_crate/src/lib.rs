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
