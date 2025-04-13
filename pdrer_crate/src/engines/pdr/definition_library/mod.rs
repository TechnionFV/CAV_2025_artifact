// ************************************************************************************************
// use
// ************************************************************************************************

use fxhash::FxHashMap;

use super::{shared_objects::SharedObjects, PropertyDirectedReachabilitySolver};
use crate::{
    formulas::{Clause, Variable},
    models::{Definition, UniqueSortedVec},
    solvers::dd::DecisionDiagramManager,
};
use quick_cache::unsync::Cache;

// ************************************************************************************************
// types
// ************************************************************************************************

// ************************************************************************************************
// struct
// ************************************************************************************************

pub struct DefinitionLibrary<T: PropertyDirectedReachabilitySolver, D: DecisionDiagramManager> {
    definitions: Vec<Definition>,
    var_to_bdd: FxHashMap<Variable, D::DecisionDiagram>,
    extended_var_to_state_coi: FxHashMap<Variable, UniqueSortedVec<Variable>>,
    extended_var_to_coi: FxHashMap<Variable, UniqueSortedVec<Variable>>,
    base: usize,
    free_variable: usize,
    solver: T,
    manager_ref: D,
    clause_cache: Cache<Clause, D::DecisionDiagram>,
    implies_cache: Cache<(Clause, Clause), bool>,
    s: SharedObjects,
}

// ************************************************************************************************
// rust submodule declaration, they get searched in their respective file  names
// ************************************************************************************************

pub mod api;
pub mod solve;

// ************************************************************************************************
// re-exports of structs in these modules to simplify paths for other imports
// ************************************************************************************************
