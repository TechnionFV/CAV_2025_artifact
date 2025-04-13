// ************************************************************************************************
// use
// ************************************************************************************************

use super::CNF;
use crate::formulas::{Clause, Literal};

// ************************************************************************************************
// impl
// ************************************************************************************************

impl CNF {
    // ********************************************************************************************
    // helper functions
    // ********************************************************************************************

    // ********************************************************************************************
    // API
    // ********************************************************************************************

    pub fn apply_function_on_literals<F>(clauses: &[Clause], mut function: F) -> Vec<Clause>
    where
        F: FnMut(Literal) -> Literal,
    {
        let mut new_clauses = Vec::with_capacity(clauses.len());
        for clause in clauses.iter() {
            let mut new_clause = Vec::with_capacity(clause.len());
            for l in clause.iter().copied() {
                let l = function(l);
                new_clause.push(l);
            }
            debug_assert_eq!(new_clause.len(), clause.len());
            new_clauses.push(Clause::from_sequence(new_clause));
        }
        new_clauses
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::formulas::{Literal, Variable};

    #[test]
    fn test_simple_bounded_variable_addition() {
        let a = Literal::new(Variable::new(1));
        let b = Literal::new(Variable::new(2));
        let c = Literal::new(Variable::new(3));
        let d = Literal::new(Variable::new(4));
        let e = Literal::new(Variable::new(5));
        let x = Literal::new(Variable::new(6));

        let clauses = vec![
            Clause::from_sequence(vec![a, c]),
            Clause::from_sequence(vec![a, d]),
            Clause::from_sequence(vec![a, e]),
            Clause::from_sequence(vec![b, !c]),
            Clause::from_sequence(vec![b, !d]),
            Clause::from_sequence(vec![b, x]),
        ];

        let c2 = CNF::apply_function_on_literals(&clauses, |l| {
            if l.variable() == a.variable() {
                x.variable().literal(l.is_negated())
            } else {
                l
            }
        });

        // let c3 = CNF::rename_variables(&clauses, &old_to_new, true);
        // assert_eq!(c2, c3);

        let expected_result = vec![
            Clause::from_sequence(vec![x, c]),
            Clause::from_sequence(vec![x, d]),
            Clause::from_sequence(vec![x, e]),
            Clause::from_sequence(vec![b, !c]),
            Clause::from_sequence(vec![b, !d]),
            Clause::from_sequence(vec![b, x]),
        ];

        assert_eq!(c2, expected_result);
    }
}
