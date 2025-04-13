// ************************************************************************************************
// use
// ************************************************************************************************

use super::BVA3Pattern;
use crate::{
    formulas::{Clause, Literal},
    models::{definition::DefinitionFunction, SortedVecOfLiterals, UniqueSortedVec},
};

// ************************************************************************************************
// parameters
// ************************************************************************************************

pub struct BVA3PatternMatch<'a> {
    pub cnf_index: usize,
    pub pattern: BVA3Pattern,
    pub clause_indices: Vec<&'a Clause>,
}

// ************************************************************************************************
// impl
// ************************************************************************************************

impl BVA3PatternMatch<'_> {
    fn get_common_clause(&self) -> SortedVecOfLiterals {
        let common = self
            .clause_indices
            .iter()
            .fold(None, |acc: Option<UniqueSortedVec<Literal>>, x| {
                if let Some(acc) = acc {
                    Some(acc.intersect(x.peek().peek()))
                } else {
                    Some(x.peek().peek().clone())
                }
            })
            .unwrap();
        SortedVecOfLiterals::from_ordered_set(common.unpack())
    }

    pub fn get_clauses_in_pattern(&self) -> &Vec<&'_ Clause> {
        &self.clause_indices
    }

    pub fn get_resulting_clauses<F: FnMut(DefinitionFunction, SortedVecOfLiterals) -> Literal>(
        &self,
        mut get_ev: F,
    ) -> (Vec<Clause>, Vec<Literal>) {
        let mut common = self.get_common_clause();
        match self.pattern {
            BVA3Pattern::AndPattern(a, b) => {
                let x = get_ev(
                    DefinitionFunction::And,
                    SortedVecOfLiterals::from_sequence(vec![a, b]),
                );
                common.insert(x);
                let clauses_after = vec![Clause::from_ordered_set(common.unpack().unpack())];
                (clauses_after, vec![x])
            }
            BVA3Pattern::XorPattern(a, b) => {
                let (a, b) = (a.literal(false), b.literal(false));
                let x = get_ev(
                    DefinitionFunction::Xor,
                    SortedVecOfLiterals::from_sequence(vec![a, b]),
                );
                let exists_a = self.clause_indices[0].contains(&a);
                let exists_b = self.clause_indices[0].contains(&b);
                debug_assert!(exists_a || self.clause_indices[0].contains(&!a));
                debug_assert!(exists_b || self.clause_indices[0].contains(&!b));
                let negated = (exists_a && !exists_b) || (!exists_a && exists_b);
                if negated {
                    common.insert(!x);
                } else {
                    common.insert(x);
                }
                let clauses_after = vec![Clause::from_ordered_set(common.unpack().unpack())];
                (clauses_after, vec![x])
            }
            BVA3Pattern::HalfAdderPattern(a, b, c, d) => {
                let x = get_ev(
                    DefinitionFunction::Xor,
                    SortedVecOfLiterals::from_sequence(vec![a, b]),
                );
                let y = get_ev(
                    DefinitionFunction::And,
                    SortedVecOfLiterals::from_sequence(vec![a, b]),
                );
                let z = get_ev(
                    DefinitionFunction::And,
                    SortedVecOfLiterals::from_sequence(vec![d, y]),
                );

                let mut c1 = common.clone();
                c1.insert(x);
                c1.insert(y);
                c1.insert(d);
                let mut c2 = common.clone();
                c2.insert(x);
                c2.insert(z);
                c2.insert(c);

                let clauses_after = vec![
                    Clause::from_ordered_set(c1.unpack().unpack()),
                    Clause::from_ordered_set(c2.unpack().unpack()),
                ];
                (clauses_after, vec![x, y, z])
            }
        }
    }
}
