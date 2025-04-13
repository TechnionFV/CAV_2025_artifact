// ************************************************************************************************
// use
// ************************************************************************************************

use std::fmt::Display;

use crate::models::{PrettyTable, TernaryValue};

// ************************************************************************************************
// types
// ************************************************************************************************

#[derive(Clone, Debug)]
pub struct TernaryValueVector {
    vector: Vec<TernaryValue>,
}

// ************************************************************************************************
// constants
// ************************************************************************************************

// ************************************************************************************************
// impl
// ************************************************************************************************

impl TernaryValueVector {
    // ********************************************************************************************
    // API
    // ********************************************************************************************

    pub fn new(length: usize) -> Self {
        Self {
            vector: vec![TernaryValue::X; length],
        }
    }

    pub fn set(&mut self, index: usize, value: TernaryValue) {
        self.vector[index] = value;
    }

    pub fn get(&self, index: usize) -> TernaryValue {
        self.vector[index]
    }

    pub fn clear(&mut self) {
        for x in self.vector.iter_mut() {
            *x = TernaryValue::X;
        }
    }

    pub fn clone_from_slice(&mut self, other: &Self) {
        self.vector.clone_from_slice(&other.vector);
    }
}

impl Display for TernaryValueVector {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut table = PrettyTable::new(vec!["Signal".to_string(), "Value".to_string()]);
        for (i, value) in self.vector.iter().enumerate() {
            table
                .add_row(vec![i.to_string(), value.to_string()])
                .unwrap();
        }
        write!(f, "{}", table)
    }
}
