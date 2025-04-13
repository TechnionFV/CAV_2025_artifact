// ************************************************************************************************
// use
// ************************************************************************************************

use super::{BVA3Parameters, BVA3Pattern, BVA3PatternMatch};
use crate::formulas::{Clause, CNF};
use fxhash::{FxHashMap, FxHashSet};

// ************************************************************************************************
// helper functions
// ************************************************************************************************

fn find_most_pattern_that_can_coexist(
    indices_of_matches_to_consider: &[usize],
    matches: &[BVA3PatternMatch],
) -> Vec<usize> {
    // for each clause, count the patterns that use it
    let mut clause_index_histogram = FxHashMap::with_capacity_and_hasher(
        indices_of_matches_to_consider.len(),
        Default::default(),
    );
    for i in indices_of_matches_to_consider.iter().copied() {
        for j in matches[i].clause_indices.iter().copied() {
            *clause_index_histogram.entry(j).or_insert(0) += 1;
        }
    }

    // for each pattern, count the max clause index that it uses
    let mut patterns: Vec<_> = indices_of_matches_to_consider
        .iter()
        .map(|i| (*i, &matches[*i]))
        .map(|(i, m)| {
            (
                m,
                i,
                m.clause_indices
                    .iter()
                    .map(|ci| *clause_index_histogram.get(ci).unwrap())
                    .max()
                    .unwrap(),
            )
        })
        .collect();

    // sort patterns by least max used clause
    patterns.sort_unstable_by_key(|x| x.1);

    // keep track of which clauses are used
    let mut used_clauses =
        FxHashSet::with_capacity_and_hasher(clause_index_histogram.len(), Default::default());
    let mut result = Vec::with_capacity(patterns.len());

    for (m, i, _) in patterns {
        let mut can_coexist = true;
        for ci in m.clause_indices.iter() {
            if used_clauses.contains(ci) {
                can_coexist = false;
                break;
            }
        }

        if can_coexist {
            for ci in m.clause_indices.iter().copied() {
                used_clauses.insert(ci);
            }
            result.push(i);
        }
    }

    // iterate clauses in order of least used
    result.shrink_to_fit();
    result
}

fn make_clause_length_index(cnf: &[Clause]) -> Vec<Vec<&Clause>> {
    let max_clause_length = cnf.iter().map(|x| x.len()).max().unwrap();
    let mut length_index = vec![vec![]; max_clause_length + 1];
    for c in cnf.iter() {
        length_index[c.len()].push(c);
    }
    length_index
}

// ************************************************************************************************
// API
// ************************************************************************************************

impl CNF {
    /// Match a set of patterns on a set of CNFs.
    ///
    pub fn bva3_match_patterns_on_cnfs(
        cnfs: &[Vec<Clause>],
        params: BVA3Parameters,
    ) -> Vec<BVA3PatternMatch<'_>> {
        let mut m = Vec::new();
        let mut sub_vec_1 = Vec::with_capacity(4);
        let mut sub_vec_2 = Vec::with_capacity(4);

        for (cnf_index, cnf) in cnfs.iter().enumerate() {
            if cnf.is_empty() {
                continue;
            }

            // make index to cnf clauses by clause length
            let index = make_clause_length_index(cnf);

            for (n, same_length_clauses) in index.iter().enumerate() {
                for (skip, ci) in same_length_clauses.iter().enumerate() {
                    debug_assert!(ci.len() == n);
                    for cj in same_length_clauses.iter().skip(skip + 1) {
                        sub_vec_1.clear();
                        sub_vec_2.clear();

                        ci.peek().peek().symmetric_difference_custom(
                            cj.peek().peek(),
                            if params.xor_pattern { 3 } else { 2 },
                            if params.xor_pattern { 3 } else { 2 },
                            &mut sub_vec_1,
                            &mut sub_vec_2,
                        );

                        if sub_vec_1.len() == 1
                            && sub_vec_2.len() == 1
                            && sub_vec_1[0].variable() != sub_vec_2[0].variable()
                        {
                            if params.and_pattern {
                                let (a, b) = (sub_vec_1[0], sub_vec_2[0]);
                                let (a, b, ci, cj) = if a < b {
                                    (a, b, ci, cj)
                                } else {
                                    (b, a, cj, ci)
                                };
                                m.push(BVA3PatternMatch {
                                    cnf_index,
                                    pattern: BVA3Pattern::AndPattern(a, b),
                                    clause_indices: vec![ci, cj],
                                });
                            }

                            if params.half_adder_pattern && n + 1 < index.len() {
                                // (a or b or c) and (a or b or d) or (!a or !b or c or d)
                                let c = sub_vec_1[0];
                                let d = sub_vec_2[0];
                                for ck in index[n + 1].iter() {
                                    if !ck.contains(&c) || !ck.contains(&d) {
                                        continue;
                                    }

                                    sub_vec_1.clear();
                                    sub_vec_2.clear();

                                    ci.peek().peek().symmetric_difference_custom(
                                        ck.peek().peek(),
                                        3,
                                        4,
                                        &mut sub_vec_1,
                                        &mut sub_vec_2,
                                    );

                                    if sub_vec_1.len() == 2
                                        && sub_vec_2.len() == 3
                                        && sub_vec_2.contains(&d)
                                        && sub_vec_2.contains(&!sub_vec_1[0])
                                        && sub_vec_2.contains(&!sub_vec_1[1])
                                    {
                                        let a = sub_vec_1[0];
                                        let b = sub_vec_1[1];

                                        let (a, b) = if a < b { (a, b) } else { (b, a) };
                                        let (c, d, ci, cj) = if c < d {
                                            (c, d, ci, cj)
                                        } else {
                                            (d, c, cj, ci)
                                        };

                                        m.push(BVA3PatternMatch {
                                            cnf_index,
                                            pattern: BVA3Pattern::HalfAdderPattern(a, b, c, d),
                                            clause_indices: vec![ci, cj, ck],
                                        });
                                    }
                                }
                            }
                        } else if params.xor_pattern
                            && sub_vec_1.len() == 2
                            && sub_vec_2.len() == 2
                            && sub_vec_1[0].variable() == sub_vec_2[0].variable()
                            && sub_vec_1[1].variable() == sub_vec_2[1].variable()
                        {
                            let a = sub_vec_1[0].variable();
                            let b = sub_vec_1[1].variable();
                            let (a, b, ci, cj) = if a < b {
                                (a, b, ci, cj)
                            } else {
                                (b, a, cj, ci)
                            };
                            m.push(BVA3PatternMatch {
                                cnf_index,
                                pattern: BVA3Pattern::XorPattern(a, b),
                                clause_indices: vec![*ci, *cj],
                            });
                        }
                    }
                }
            }
        }

        m
    }

    /// Returns a hashmap that maps each set of added variables to the indexes of the pattern matches that use them.
    pub fn bva3_cluster_pattern_matches(
        matches: &[BVA3PatternMatch],
    ) -> FxHashMap<BVA3Pattern, Vec<usize>> {
        let mut match_map = FxHashMap::with_capacity_and_hasher(matches.len(), Default::default());
        for (i, m) in matches.iter().enumerate() {
            match_map.entry(m.pattern).or_insert_with(Vec::new).push(i);
        }
        match_map
    }

    pub fn bva3_get_definition_candidates(
        matches: &[BVA3PatternMatch],
        remove_patterns_that_cannot_co_exist: bool,
    ) -> FxHashMap<BVA3Pattern, Vec<usize>> {
        if matches.is_empty() {
            return Default::default();
        }

        // cluster pattern matches that agree on the extension variables.
        let mut match_map = Self::bva3_cluster_pattern_matches(matches);

        // take each pattern and use a variant of the approximate algorithm for the
        // set cover problem to choose which pattern matches can co-exist together.
        if remove_patterns_that_cannot_co_exist {
            for (_, v) in match_map.iter_mut() {
                *v = find_most_pattern_that_can_coexist(v, matches);
            }
        }

        match_map
    }

    ///
    /// This may remove some pattern matches that cannot co-exist together.
    pub fn bva3_get_best_set_of_patterns_to_add(
        matches: &[BVA3PatternMatch],
        remove_patterns_that_cannot_co_exist: bool,
    ) -> (BVA3Pattern, Vec<usize>) {
        let match_map =
            Self::bva3_get_definition_candidates(matches, remove_patterns_that_cannot_co_exist);
        match_map.into_iter().max_by_key(|x| x.1.len()).unwrap()
    }
}
