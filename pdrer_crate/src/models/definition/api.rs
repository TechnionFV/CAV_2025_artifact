// ************************************************************************************************
// use
// ************************************************************************************************

use super::{Definition, DefinitionFunction};
use crate::formulas::{Clause, Cube, Literal};

// ************************************************************************************************
// impl
// ************************************************************************************************

impl Definition {
    // ********************************************************************************************
    // helper functions
    // ********************************************************************************************

    // ********************************************************************************************
    // API
    // ********************************************************************************************

    pub fn get_ternary_result_of_definition_on_cube(&self, cube: &Cube) -> Option<Literal> {
        let x = self.variable.literal(false);
        match self.function {
            DefinitionFunction::And => {
                assert!(!self.inputs.is_empty());
                let mut all_true = true;
                for i in self.inputs.iter().copied() {
                    if cube.contains(&!i) {
                        return Some(!x);
                    } else if !cube.contains(&i) {
                        all_true = false
                    }
                }
                if all_true && !self.inputs.is_empty() {
                    Some(x)
                } else {
                    None
                }
            }
            DefinitionFunction::Xor => {
                let mut number_of_true = 0;
                for i in self.inputs.iter().copied() {
                    if cube.contains(&!i) {
                    } else if cube.contains(&i) {
                        number_of_true += 1;
                    } else {
                        return None;
                    }
                }
                Some(if number_of_true % 2 == 0 { !x } else { x })
            }
        }
    }

    pub fn to_cnf(&self) -> Vec<Clause> {
        let x = self.variable.literal(false);
        match self.function {
            DefinitionFunction::And => {
                let mut cnf = vec![];
                let clause = Clause::from_sequence(
                    self.inputs.iter().copied().map(|l| !l).chain([x]).collect(),
                );
                cnf.push(clause);
                for l in self.inputs.iter() {
                    let clause = Clause::from_sequence(vec![*l, !x]);
                    cnf.push(clause);
                }
                cnf
            }
            DefinitionFunction::Xor => {
                let mut cnf = vec![];
                let mut is_negated = vec![false; self.inputs.len()];
                loop {
                    let mut clause = vec![];
                    let mut number_of_negated = 0;
                    for (l, is_neg) in self.inputs.iter().copied().zip(is_negated.iter().copied()) {
                        if is_neg {
                            number_of_negated += 1;
                            clause.push(!l);
                        } else {
                            clause.push(l);
                        }
                    }
                    clause.push(if number_of_negated % 2 == 0 { !x } else { x });
                    cnf.push(Clause::from_sequence(clause));
                    let mut i = 0;
                    loop {
                        if i == self.inputs.len() {
                            break;
                        }
                        if is_negated[i] {
                            is_negated[i] = false;
                            i += 1;
                        } else {
                            is_negated[i] = true;
                            break;
                        }
                    }
                    if i == self.inputs.len() {
                        break;
                    }
                }
                cnf.sort_unstable();
                cnf
            }
        }
    }

    pub fn forward(&self, mut clause: Clause, to_remove: &mut Vec<Literal>) -> Option<Clause> {
        match self.function {
            DefinitionFunction::And => {
                if self.inputs.iter().any(|l| !clause.contains(&!*l)) {
                    return Some(clause);
                }
                debug_assert!(self.inputs.iter().all(|l| clause.contains(&!*l)));
                let x = self.variable.literal(false);
                for l in self.inputs.iter().copied() {
                    to_remove.push(!l);
                }
                match (clause.contains(&x), clause.contains(&!x)) {
                    (true, true) | (true, false) => {
                        // This clause is always true
                        return None;
                    }
                    (false, true) => {}
                    (false, false) => {
                        clause.insert(!x);
                    }
                }
            }
            DefinitionFunction::Xor => {
                // you can't go forward with XOR
            }
        }
        Some(clause)
    }

    pub fn backwards(&self, mut clause: Clause) -> Clause {
        match self.function {
            DefinitionFunction::And => {
                let x = self.variable.literal(false);
                if clause.contains(&!x) {
                    clause.remove(&!x);
                    for l in self.inputs.iter().copied() {
                        clause.insert(!l);
                    }
                }
            }
            DefinitionFunction::Xor => {
                // you can't go backward with XOR
            }
        }

        clause
    }

    /// Returns true if the definition is valid, false otherwise.
    /// A definition is valid if the variable is greater than the max variable in the inputs.
    pub fn is_valid(&self) -> bool {
        self.inputs.max_variable() < self.variable
    }
}
