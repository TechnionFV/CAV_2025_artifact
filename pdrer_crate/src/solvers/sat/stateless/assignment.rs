// ************************************************************************************************
// use
// ************************************************************************************************

use fxhash::FxHashSet;

use crate::formulas::Variable;

// ************************************************************************************************
// struct
// ************************************************************************************************

#[derive(Debug, PartialEq, Eq)]
pub struct Assignment {
    true_variables: FxHashSet<Variable>,
    false_variables: FxHashSet<Variable>,
}

// ************************************************************************************************
// impl
// ************************************************************************************************

impl Assignment {
    // ********************************************************************************************
    // helper functions
    // ********************************************************************************************

    fn new(true_variables: FxHashSet<Variable>, false_variables: FxHashSet<Variable>) -> Self {
        Self {
            true_variables,
            false_variables,
        }
    }

    // ********************************************************************************************
    // API
    // ********************************************************************************************

    pub fn from_dimacs_assignment(vector: &[i32]) -> Self {
        let mut true_variables = FxHashSet::<Variable>::default();
        let mut false_variables = FxHashSet::<Variable>::default();

        for var in vector.iter() {
            let var_num = var.unsigned_abs();
            debug_assert!(var_num != 0);
            let var_num = Variable::new(var_num);
            if var < &0 {
                false_variables.insert(var_num);
            } else {
                true_variables.insert(var_num);
            }
        }

        Self::new(true_variables, false_variables)
    }

    pub fn get_value(&self, variable: &Variable) -> Option<bool> {
        if self.true_variables.contains(variable) {
            Some(true)
        } else if self.false_variables.contains(variable) {
            Some(false)
        } else {
            None
        }
    }
}
