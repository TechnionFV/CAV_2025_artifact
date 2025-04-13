// ************************************************************************************************
// use
// ************************************************************************************************

use crate::models::ternary_value::TernaryValue;
use crate::models::TruthTable;
use crate::models::UniqueSortedVec;
use crate::models::Wire;

// ************************************************************************************************
// structs
// ************************************************************************************************

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct CircuitLatch {
    pub input: Wire,
    pub initial: TernaryValue,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct CircuitAnd {
    pub inputs: UniqueSortedVec<Wire>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct CircuitGenericGate {
    pub truth_table: TruthTable,
}

// ************************************************************************************************
// enum
// ************************************************************************************************

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum CircuitNodeType {
    ConstantZero,
    Input,
    Latch(CircuitLatch),
    And(CircuitAnd),
    GenericGate(CircuitGenericGate),
}

// ************************************************************************************************
// final struct
// ************************************************************************************************

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CircuitNode {
    // pub level: usize,
    pub node_type: CircuitNodeType,
    // pub users: UniqueSortedVec<Signal>, // also known as fan-out
    // pub internal_users: UniqueSortedVec<Signal>, // users without latches
}
