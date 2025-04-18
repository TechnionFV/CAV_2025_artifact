// ************************************************************************************************
// use
// ************************************************************************************************

use super::CNF;
use std::fmt;

// ************************************************************************************************
// Default
// ************************************************************************************************

// ************************************************************************************************
// PartialEq
// ************************************************************************************************

impl PartialEq for CNF {
    fn eq(&self, other: &Self) -> bool {
        self.to_string() == other.to_string()
    }
}

// ************************************************************************************************
// Eq
// ************************************************************************************************

impl Eq for CNF {}

// ************************************************************************************************
// printing
// ************************************************************************************************

/// Printing is done in a canonical way. This means that the clauses are sorted and that the
/// literals in the clauses are sorted.
impl fmt::Display for CNF {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let string_vec = self
            .clauses
            .iter()
            .map(|one_clause| one_clause.to_string())
            .collect::<Vec<String>>();
        write!(
            f,
            "p cnf {} {}\n{}",
            self.max_variable_number,
            self.len(),
            string_vec.join("\n")
        )
    }
}
