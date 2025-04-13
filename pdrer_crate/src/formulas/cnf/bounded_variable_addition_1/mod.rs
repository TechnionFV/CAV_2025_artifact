// ************************************************************************************************
// use
// ************************************************************************************************

use super::CNF;
use crate::formulas::{Clause, Variable};
use crate::{formulas::Literal, models::UniqueSortedVec};

// ************************************************************************************************
// structs used in API
// ************************************************************************************************

#[derive(Debug)]
pub struct VariableAddition {
    pub variable: Option<Variable>,
    pub matched_literals: UniqueSortedVec<Literal>,
    pub constraint_clauses: Vec<Clause>,
    pub definition_clauses: Vec<Clause>,
    pub removed_clauses: Vec<Clause>,
    pub removed_clauses_indexes: UniqueSortedVec<usize>,
}

#[derive(Debug, Eq, PartialEq)]
pub struct PElement {
    pub l_tag: Literal,
    pub i_c: usize,
    pub i_d: usize,
}

// ************************************************************************************************
// rust submodule declaration, they get searched in their respective file  names
// **************
pub mod api;
pub mod parameters;

// ************************************************************************************************
// re-exports of structs in these modules to simplify paths for other imports
// ************************************************************************************************

pub use parameters::BoundedVariableAdditionParameters;

// ************************************************************************************************
// Tests
// ************************************************************************************************

#[cfg(test)]
mod tests {

    use super::*;
    use crate::formulas::Variable;

    #[test]
    fn test_simple_bounded_variable_addition() {
        let a = Literal::new(Variable::new(1));
        let b = Literal::new(Variable::new(2));
        let c = Literal::new(Variable::new(3));
        let d = Literal::new(Variable::new(4));
        let e = Literal::new(Variable::new(5));
        let x = Literal::new(Variable::new(6));

        let mut clauses = CNF::from_sequence(vec![
            Clause::from_sequence(vec![a, c]),
            Clause::from_sequence(vec![a, d]),
            Clause::from_sequence(vec![a, e]),
            Clause::from_sequence(vec![b, c]),
            Clause::from_sequence(vec![b, d]),
            Clause::from_sequence(vec![b, e]),
        ]);

        let mut par = BoundedVariableAdditionParameters::new();
        let mut h = 5;
        par.new_variable_getter = Box::new(move |_| {
            h += 1;
            Variable::new(h)
        });
        clauses.simple_bounded_variable_addition(par);

        let expected_result = CNF::from_sequence(vec![
            Clause::from_sequence(vec![a, !x]),
            Clause::from_sequence(vec![b, !x]),
            Clause::from_sequence(vec![c, x]),
            Clause::from_sequence(vec![d, x]),
            Clause::from_sequence(vec![e, x]),
        ]);

        assert_eq!(clauses, expected_result);
    }

    #[test]
    fn test_simple_bounded_variable_addition_2() {
        let a = Literal::new(Variable::new(1));
        let b = Literal::new(Variable::new(2));
        let p = Literal::new(Variable::new(3));
        let q = Literal::new(Variable::new(4));
        let r = Literal::new(Variable::new(5));
        let s = Literal::new(Variable::new(6));
        let t = Literal::new(Variable::new(7));

        let mut clauses = CNF::from_sequence(vec![
            Clause::from_sequence(vec![a, p, q]),
            Clause::from_sequence(vec![a, p, r]),
            Clause::from_sequence(vec![a, r, s]),
            Clause::from_sequence(vec![a, t]),
            Clause::from_sequence(vec![b, p, q]),
            Clause::from_sequence(vec![b, p, r]),
            Clause::from_sequence(vec![b, r, s]),
            Clause::from_sequence(vec![b, t]),
        ]);

        let mut par = BoundedVariableAdditionParameters::new();
        let mut h = 7;
        par.new_variable_getter = Box::new(move |_| {
            h += 1;
            Variable::new(h)
        });
        clauses.simple_bounded_variable_addition(par);

        let x = Literal::new(Variable::new(8));
        let expected_result = CNF::from_sequence(vec![
            Clause::from_sequence(vec![x, p, q]),
            Clause::from_sequence(vec![x, p, r]),
            Clause::from_sequence(vec![x, r, s]),
            Clause::from_sequence(vec![x, t]),
            Clause::from_sequence(vec![!x, a]),
            Clause::from_sequence(vec![!x, b]),
        ]);
        assert_eq!(clauses, expected_result);
    }

    #[test]
    /// Test the case where a literal and its negation are in matched literals
    fn test_simple_bounded_variable_addition_3() {
        let a = Literal::new(Variable::new(1));
        // let b = Literal::new(Variable::new(2));
        let p = Literal::new(Variable::new(3));
        let q = Literal::new(Variable::new(4));
        let r = Literal::new(Variable::new(5));
        let s = Literal::new(Variable::new(6));
        let t = Literal::new(Variable::new(7));

        let mut clauses = CNF::from_sequence(vec![
            Clause::from_sequence(vec![a, p, q]),
            Clause::from_sequence(vec![a, p, r]),
            Clause::from_sequence(vec![a, r, s]),
            Clause::from_sequence(vec![a, t]),
            Clause::from_sequence(vec![!a, p, q]),
            Clause::from_sequence(vec![!a, p, r]),
            Clause::from_sequence(vec![!a, r, s]),
            Clause::from_sequence(vec![!a, t]),
        ]);

        let mut par = BoundedVariableAdditionParameters::new();
        let mut h = 7;
        par.new_variable_getter = Box::new(move |_| {
            h += 1;
            Variable::new(h)
        });
        clauses.simple_bounded_variable_addition(par);

        let expected_result = CNF::from_sequence(vec![
            Clause::from_sequence(vec![p, q]),
            Clause::from_sequence(vec![p, r]),
            Clause::from_sequence(vec![r, s]),
            Clause::from_sequence(vec![t]),
        ]);
        assert_eq!(clauses, expected_result);
    }

    #[test]
    /// Test the case where a literal and its negation are in matched literals and the
    /// clause that results is already in the CNF
    fn test_simple_bounded_variable_addition_4() {
        let a = Literal::new(Variable::new(1));
        // let b = Literal::new(Variable::new(2));
        let p = Literal::new(Variable::new(3));
        let q = Literal::new(Variable::new(4));
        let r = Literal::new(Variable::new(5));
        let s = Literal::new(Variable::new(6));
        let t = Literal::new(Variable::new(7));

        let mut clauses = CNF::from_sequence(vec![
            Clause::from_sequence(vec![p, q]),
            Clause::from_sequence(vec![a, p, q]),
            Clause::from_sequence(vec![a, p, r]),
            Clause::from_sequence(vec![a, r, s]),
            Clause::from_sequence(vec![a, t]),
            Clause::from_sequence(vec![!a, p, q]),
            Clause::from_sequence(vec![!a, p, r]),
            Clause::from_sequence(vec![!a, r, s]),
            Clause::from_sequence(vec![!a, t]),
        ]);

        let mut par = BoundedVariableAdditionParameters::new();
        let mut h = 7;
        par.new_variable_getter = Box::new(move |_| {
            h += 1;
            Variable::new(h)
        });
        clauses.simple_bounded_variable_addition(par);

        let expected_result = CNF::from_sequence(vec![
            Clause::from_sequence(vec![p, q]),
            Clause::from_sequence(vec![p, r]),
            Clause::from_sequence(vec![r, s]),
            Clause::from_sequence(vec![t]),
        ]);
        assert_eq!(clauses, expected_result);
    }
}
