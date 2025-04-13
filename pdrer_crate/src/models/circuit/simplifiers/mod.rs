// ************************************************************************************************
// rust submodule decleration, they get searched in their respective file  names
// ************************************************************************************************

pub mod condense;
pub mod detect_generic_patterns;
pub mod merge_and_gates;
pub mod remove_unused;
pub mod simplify;
pub mod structural_hash;
pub mod technology_map;

// ************************************************************************************************
// re-exports of structs in these modules to simplify paths for other imports
// ************************************************************************************************

pub use condense::CircuitCondenser;
pub use merge_and_gates::CircuitAndGateMerger;
pub use remove_unused::CircuitUnusedSignalRemover;
pub use structural_hash::CircuitStructuralHashing;
pub use technology_map::CircuitTechnologyMapper;
