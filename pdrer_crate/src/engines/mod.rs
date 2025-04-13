//! Some hardware model checking algorithms that are already implemented.

// ************************************************************************************************
// rust submodule declaration, they get searched in their respective file  names
// ************************************************************************************************

pub mod pdr;

// ************************************************************************************************
// re-exports of structs in these modules to simplify paths for other imports
// ************************************************************************************************

pub use pdr::PropertyDirectedReachability;
pub use pdr::PropertyDirectedReachabilityParameters;
