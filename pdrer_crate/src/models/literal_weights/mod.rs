// ************************************************************************************************
// use
// ************************************************************************************************

// ************************************************************************************************
// struct
// ************************************************************************************************

#[derive(Debug, Clone)]
pub struct LiteralWeights {
    // literal priorities that indicate how important they are
    literal_weights: Vec<usize>,
    // inputs_index: usize,
    // latches_index: usize,
    // gates_index: usize,
    // state_variable_range: VariableRange,
    // input_variable_range: VariableRange,
    amount_to_add_when_literal_appears: usize,
    amount_to_subtract_when_literal_does_not_appear: usize,
}

// ************************************************************************************************
// rust submodule declaration, they get searched in their respective file  names
// ************************************************************************************************

pub mod construction;
pub mod get;
pub mod operations;
pub mod printing;
pub mod update;

// ************************************************************************************************
// re-exports of structs in these modules to simplify paths for other imports
// ************************************************************************************************
