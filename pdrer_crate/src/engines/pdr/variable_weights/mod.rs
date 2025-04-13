// ************************************************************************************************
// use
// ************************************************************************************************

// ************************************************************************************************
// type
// ************************************************************************************************

type VariableWeight = f64;
const INITIAL_WEIGHT: VariableWeight = 0.0;

// const W_POSITIVE: VariableWeight = (WEIGHT_WHEN_OBSERVED * MULTIPLIER) as VariableWeight;
// const W_NEGATIVE: VariableWeight = (WEIGHT_WHEN_NOT_OBSERVED * MULTIPLIER) as VariableWeight;

// ************************************************************************************************
// struct
// ************************************************************************************************

#[derive(Debug, Clone)]
/// Exponential moving average (EMA) with variable weights.
/// For a variable that was observed:
/// EMA = (WEIGHT_WHEN_OBSERVED x MULTIPLIER) + (OLD_EMA x (1 - MULTIPLIER))
/// For a variable that was not observed:
/// EMA = (WEIGHT_WHEN_NOT_OBSERVED x MULTIPLIER) + (OLD_EMA x (1 - MULTIPLIER))
pub struct VariableWeights {
    weight_per_variable: Vec<VariableWeight>,
    decay: VariableWeight,
    one_minus_decay: VariableWeight,
    // multiplier: f32,
    // one_minus_multiplier: f32,
    // weight_when_observed: VariableWeight,
    // weight_when_not_observed: VariableWeight,
}

// ************************************************************************************************
// rust submodule declaration, they get searched in their respective file  names
// ************************************************************************************************

pub mod construction;
pub mod get;
pub mod printing;
pub mod update;

// ************************************************************************************************
// re-exports of structs in these modules to simplify paths for other imports
// ************************************************************************************************
