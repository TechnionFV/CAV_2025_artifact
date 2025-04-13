//! models like AndInverterGraph and FiniteStateTransitionSystem.

// ************************************************************************************************
// rust submodule decleration, they get searched in their respective file  names
// ************************************************************************************************

pub mod finite_state_transition_system;
// requires folder in this directory with the name 'finite_state_transition_system'
pub mod and_inverter_graph;
// requires folder in this directory with the name 'and_inverter_graph'
pub mod btor;
pub mod circuit;
pub mod circuit_builder;
pub mod circuit_simulator;
pub mod clause_combinations_iterator;
pub mod counterexample;
pub mod definition;
pub mod literal_weights;
pub mod one_pass_reader;
pub mod power_set_iterator;
pub mod pretty_table;
pub mod proof;
pub mod signal;
pub mod signal_tracker;
pub mod sorted_vec_of_literals;
pub mod ternary_value;
pub mod time_stats;
pub mod truth_table;
pub mod union_find;
pub mod unique_sorted_hash_map;
pub mod unique_sorted_vec;
pub mod utils;
pub mod vec2d;
pub mod vec3d;
pub mod wire;

// ************************************************************************************************
// re-exports of structs in these modules to simplify paths for other imports
// ************************************************************************************************

pub use and_inverter_graph::AndInverterGraph;
pub use btor::BTOR;
pub use circuit::Circuit;
pub use circuit_builder::CircuitBuilder;
pub use circuit_simulator::CircuitSimulator;
pub use counterexample::Counterexample;
pub use definition::Definition;
pub use finite_state_transition_system::FiniteStateTransitionSystem;
pub use literal_weights::LiteralWeights;
pub use one_pass_reader::OnePassReader;
pub use pretty_table::PrettyTable;
pub use proof::Proof;
pub use signal::Signal;
pub use signal_tracker::SignalTracker;
pub use sorted_vec_of_literals::SortedVecOfLiterals;
pub use ternary_value::TernaryValue;
pub use time_stats::TimeStats;
pub use truth_table::TruthTable;
pub use union_find::UnionFind;
pub use unique_sorted_hash_map::UniqueSortedHashMap;
pub use unique_sorted_vec::UniqueSortedVec;
pub use utils::Utils;
pub use vec2d::Vec2d;
pub use wire::Wire;
