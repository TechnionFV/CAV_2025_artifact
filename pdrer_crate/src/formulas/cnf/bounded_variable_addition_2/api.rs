// ************************************************************************************************
// use
// ************************************************************************************************

use fxhash::{FxHashMap, FxHashSet};

use crate::{
    formulas::{Clause, Literal, Variable, CNF},
    models::{definition::DefinitionFunction, Definition, SortedVecOfLiterals, UniqueSortedVec},
};
use std::fmt::{self, Debug};

use super::variable_mapping::VariableMapping;

// ************************************************************************************************
// BVA pattern object
// ************************************************************************************************

/// Pattern to find in the clauses
#[derive(Clone, PartialEq, Eq)]
pub struct BVA2Pattern {
    pub inputs: Vec<SortedVecOfLiterals>,
    pub result: Vec<SortedVecOfLiterals>,
    pub definitions: Vec<(Variable, DefinitionFunction, SortedVecOfLiterals)>,
    pub score: usize,
    /// Selects which variables in the pattern must be different
    pub neq: Vec<(Variable, Variable)>,
}

impl BVA2Pattern {
    fn get_n_lits(n: usize) -> Vec<Literal> {
        let mut lits = vec![];
        for i in 1..=n {
            lits.push(Variable::new(i as u32).literal(false));
        }
        lits
    }

    /// The classic AND pattern found in the original paper
    pub fn and_pattern() -> Self {
        let lits = Self::get_n_lits(3);
        let a = lits[0];
        let b = lits[1];
        let x = lits[2];

        Self {
            inputs: vec![
                SortedVecOfLiterals::from_ordered_set(vec![a]),
                SortedVecOfLiterals::from_ordered_set(vec![b]),
            ],
            result: vec![SortedVecOfLiterals::from_ordered_set(vec![x])],
            definitions: vec![(
                x.variable(),
                DefinitionFunction::And,
                SortedVecOfLiterals::from_ordered_set(vec![a, b]),
            )],
            score: 1,
            neq: vec![(a.variable(), b.variable())],
        }
    }

    /// A pattern that can indicate a XOR gate
    pub fn xor_pattern() -> Self {
        let lits = Self::get_n_lits(3);
        let a = lits[0];
        let b = lits[1];
        let x = lits[2];

        Self {
            inputs: vec![
                SortedVecOfLiterals::from_ordered_set(vec![a, b]),
                SortedVecOfLiterals::from_ordered_set(vec![!a, !b]),
            ],
            result: vec![SortedVecOfLiterals::from_ordered_set(vec![x])],
            definitions: vec![(
                x.variable(),
                DefinitionFunction::Xor,
                SortedVecOfLiterals::from_ordered_set(vec![a, b]),
            )],
            score: 1,
            neq: vec![],
        }
    }

    pub fn xor_pattern_2() -> Self {
        let lits = Self::get_n_lits(8);

        let a = lits[0];
        let b = lits[1];
        let c = lits[2];
        let d = lits[3];

        let x = lits[4];
        // let w = lits[5];
        let y = lits[6];
        let z = lits[7];

        Self {
            inputs: vec![
                SortedVecOfLiterals::from_ordered_set(vec![a, b, c]),
                SortedVecOfLiterals::from_ordered_set(vec![a, b, d]),
                SortedVecOfLiterals::from_ordered_set(vec![!a, !b, c, d]),
            ],
            result: vec![
                SortedVecOfLiterals::from_sequence(vec![x, y, d]),
                SortedVecOfLiterals::from_sequence(vec![x, z, c]),
            ],
            definitions: vec![
                (
                    x.variable(),
                    DefinitionFunction::Xor,
                    SortedVecOfLiterals::from_sequence(vec![a, b]),
                ),
                // (
                //     w.variable(),
                //     DefinitionFunction::And,
                //     SortedVecOfLiterals::from_sequence(vec![a, b]),
                // ),
                (
                    y.variable(),
                    DefinitionFunction::And,
                    SortedVecOfLiterals::from_sequence(vec![a, b]),
                ),
                (
                    z.variable(),
                    DefinitionFunction::And,
                    SortedVecOfLiterals::from_sequence(vec![a, b, d]),
                ),
            ],
            score: 1,
            neq: vec![],
        }
    }
}

// ************************************************************************************************
// BVA pattern object
// ************************************************************************************************

/// Holds pattern as well as data that can be calculated from the pattern
struct InternalPatternMetaData {
    input_diffs: Vec<Vec<(SortedVecOfLiterals, SortedVecOfLiterals)>>,
    _strides: Vec<usize>,
    vars_that_need_mapping: Vec<Variable>,
    max_diff: usize,
    _id: usize,
}

impl InternalPatternMetaData {
    fn new(pattern: &BVA2Pattern, id: usize) -> InternalPatternMetaData {
        assert_eq!(
            pattern.inputs,
            {
                let mut a = pattern.inputs.clone();
                a.sort_by(|a, b| a.len().cmp(&b.len()).then(a.cmp(b)));
                a.dedup();
                a
            },
            "The inputs of a pattern must be unique and sorted by length"
        );

        let n = pattern.inputs.len();
        let mut input_diffs =
            vec![vec![(SortedVecOfLiterals::new(), SortedVecOfLiterals::new()); n]; n];
        for (i, in1) in pattern.inputs.iter().enumerate() {
            for (j, in2) in pattern.inputs.iter().enumerate().skip(i + 1) {
                let (a, b) = in1.peek().symmetric_difference(in2.peek());
                let diff: (SortedVecOfLiterals, SortedVecOfLiterals) = (
                    SortedVecOfLiterals::from_ordered_set(a.unpack()),
                    SortedVecOfLiterals::from_ordered_set(b.unpack()),
                );
                input_diffs[i][j] = diff;
            }
        }

        let max_diff = pattern.inputs.iter().map(|x| x.len()).max().unwrap();

        let strides = {
            let input_lengths: Vec<usize> = pattern.inputs.iter().map(|x| x.len()).collect();
            debug_assert!(
                input_lengths.windows(2).all(|w| w[0] <= w[1]),
                "Inputs are not sorted by clause length"
            );
            pattern
                .inputs
                .windows(2)
                .map(|w| w[1].len() - w[0].len())
                .collect()
        };

        let vars_that_need_mapping = {
            let mut vars = vec![];
            for input in pattern.inputs.iter() {
                for lit in input.iter() {
                    vars.push(lit.variable());
                }
            }
            vars.sort();
            vars.dedup();
            let new_vars: Vec<Variable> = pattern.definitions.iter().map(|x| x.0).collect();
            vars.retain(|x| !new_vars.contains(x));
            vars
        };

        InternalPatternMetaData {
            input_diffs,
            _strides: strides,
            vars_that_need_mapping,
            max_diff,
            _id: id,
        }
    }
}

// ************************************************************************************************
// impl Diff
// ************************************************************************************************

#[derive(Clone)]
// struct that holds diff between two clauses
struct Diff<'a> {
    i: &'a SortedVecOfLiterals,
    j: &'a SortedVecOfLiterals,
}

impl fmt::Display for Diff<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({:?}, {:?})", self.i, self.j)
    }
}

impl Debug for Diff<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self)
    }
}

// ************************************************************************************************
// impl Diff Cache
// ************************************************************************************************

// #[derive(Clone)]
// enum CacheEntry {
//     Diff(Box<Diff>),
//     NotYetComputed,
//     ComputedTooLarge,
// }

// type Ptr = Option<Box<Diff>>;

struct DiffCache<'a> {
    subtraction_vec_1: SortedVecOfLiterals,
    subtraction_vec_2: SortedVecOfLiterals,
    cnf: &'a [Clause],
    // cache: Result<Vec2d<CacheEntry>, FxHashMap<(usize, usize), Ptr>>,
}

impl<'a> DiffCache<'a> {
    fn new(max_diff_1: usize, max_diff_2: usize, cnf: &'a [Clause]) -> Self {
        Self {
            subtraction_vec_1: SortedVecOfLiterals::from_ordered_set(Vec::with_capacity(
                max_diff_1 + 1,
            )),
            subtraction_vec_2: SortedVecOfLiterals::from_ordered_set(Vec::with_capacity(
                max_diff_2 + 1,
            )),
            cnf,
        }
    }

    fn calculate(&mut self, i: usize, j: usize, diff_1: usize, diff_2: usize) -> Option<Diff> {
        debug_assert!(diff_1 < self.subtraction_vec_1.peek().peek().capacity());
        debug_assert!(diff_2 < self.subtraction_vec_2.peek().peek().capacity());

        self.subtraction_vec_1.peek_mut().peek_mut().clear();
        self.subtraction_vec_2.peek_mut().peek_mut().clear();

        self.cnf[i].peek().peek().symmetric_difference_custom(
            self.cnf[j].peek().peek(),
            diff_1 + 1,
            diff_2 + 1,
            self.subtraction_vec_1.peek_mut().peek_mut(),
            self.subtraction_vec_2.peek_mut().peek_mut(),
        );

        if self.subtraction_vec_1.len() != diff_1 || self.subtraction_vec_2.len() != diff_2 {
            None
        } else {
            // let diff_from = SortedVecOfLiterals::from_ordered_set(self.subtraction_vec_1.clone());
            // let diff_to = SortedVecOfLiterals::from_ordered_set(self.subtraction_vec_2.clone());

            Some(Diff {
                i: &self.subtraction_vec_1,
                j: &self.subtraction_vec_2,
            })
        }
    }
}

// ************************************************************************************************
// struct for aggregating matched patterns
// ************************************************************************************************

pub struct BVA2MatchedPattern<'b> {
    pub pattern: &'b BVA2Pattern,
    pub cnf_index: usize,
    pub clause_indices: Vec<usize>,
    pub variable_mapping: Vec<(Variable, Literal)>,
}

impl BVA2MatchedPattern<'_> {
    pub fn convert_literal(&self, l: Literal) -> Option<Literal> {
        self.variable_mapping
            .iter()
            .find(|(v, _)| *v == l.variable())
            .map(|(_, l)| *l)
            .map(|x| if l.is_negated() { !x } else { x })
    }

    pub fn get_common_clause(&self, cnf: &[Clause]) -> SortedVecOfLiterals {
        let clauses = self.clause_indices.iter().map(|x| &cnf[*x]);
        let common = clauses
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

    pub fn get_clauses_in_pattern<'a>(
        &'a self,
        cnf: &'a [Clause],
    ) -> impl ExactSizeIterator<Item = &'a Clause> + DoubleEndedIterator {
        self.clause_indices.iter().map(|x| &cnf[*x])
    }

    pub fn get_resulting_clauses<
        F: FnMut(&DefinitionFunction, Option<&SortedVecOfLiterals>) -> Literal,
    >(
        &self,
        cnf: &[Clause],
        mut get_ev: F,
    ) -> (Vec<Clause>, Vec<Result<Definition, Literal>>) {
        let common = self.get_common_clause(cnf);
        type M = FxHashMap<Variable, Literal>;
        let mut seen_extension_variables: M = FxHashMap::default();

        // function to map a pattern literal to either a cnf literal or an extension literal
        let map_pattern_literal = |l: Literal, seen_extension_variables: &M| {
            self.convert_literal(l).unwrap_or_else(|| {
                seen_extension_variables
                    .get(&l.variable())
                    .unwrap()
                    .negate_if_true(l.is_negated())
            })
        };

        let defs: Vec<Result<Definition, Literal>> = self
            .pattern
            .definitions
            .iter()
            .map(|(og_v, f, i)| {
                let mut inputs: Vec<Literal> = i
                    .iter()
                    .map(|x| map_pattern_literal(*x, &seen_extension_variables))
                    .collect();
                inputs.sort_unstable();
                // println!("Defining inputs = {:?}", inputs);
                if !SortedVecOfLiterals::are_variables_sorted_and_unique(&inputs) {
                    let v = get_ev(f, None);
                    let l = match f {
                        DefinitionFunction::And => !v,
                        DefinitionFunction::Xor => v,
                    };
                    Err(l)
                } else {
                    let inputs = SortedVecOfLiterals::from_ordered_set(inputs);
                    let v = get_ev(f, Some(&inputs));
                    let d = Definition {
                        variable: v.variable(),
                        function: *f,
                        inputs,
                    };
                    seen_extension_variables.insert(*og_v, v);
                    Ok(d)
                }
            })
            .collect();

        let clauses_after: Vec<Clause> = self
            .pattern
            .result
            .iter()
            .map(|x| {
                let mut r = common.clone();
                // println!("result = {}", x.peek());
                // println!("common = {}", r);
                for l in x.iter() {
                    // println!("r = {}", r);
                    r.insert(map_pattern_literal(*l, &seen_extension_variables));
                    // println!("r = {}", r);
                }
                Clause::from_sequence(r.unpack().unpack())
            })
            .collect();

        (clauses_after, defs)
    }
}

pub struct PatternMatches<'b> {
    pub v: Vec<BVA2MatchedPattern<'b>>,
}

// ************************************************************************************************
// Variable Mapping
// ************************************************************************************************

// ************************************************************************************************
// helper functions
// ************************************************************************************************

#[allow(clippy::too_many_arguments)]
fn match_starting_at_recursive<'a>(
    length_index: &[Vec<usize>],
    s: &mut DiffCache,
    p: &'a BVA2Pattern,
    ip: &InternalPatternMetaData,
    cnf_index: usize,
    variable_mapping: &mut VariableMapping,
    indices_so_far: &mut Vec<(usize, usize)>,
    m: &mut PatternMatches<'a>,
) {
    if indices_so_far.len() == p.inputs.len() {
        // println!("Before Getting Mapping:\n{}", variable_mapping);
        let mp = BVA2MatchedPattern {
            pattern: p,
            cnf_index,
            clause_indices: indices_so_far
                .iter()
                .map(|(k, i)| length_index[*k][*i])
                .collect(),
            variable_mapping: variable_mapping.get_mapping(),
        };
        m.v.push(mp);
        // println!("Variable mapping: {:?}", variable_mapping.get_mapping());
        // println!("Matched pattern {} at indices {:?}", p.id, indices_so_far);
        return;
    }

    // get information about current clause
    let (clause_length_i, clause_length_index_i) = *indices_so_far.last().unwrap();
    let cnf_index_i = length_index[clause_length_i][clause_length_index_i];

    // get information about jump to next clause
    let (a, b) = &ip.input_diffs[indices_so_far.len() - 1][indices_so_far.len()];
    let stride = b.len() - a.len();
    let amount_to_skip = if stride == 0 {
        clause_length_index_i + 1
    } else {
        0
    };

    let clause_length_j = stride + clause_length_i;
    if clause_length_j >= length_index.len() {
        return;
    }

    // iterate over all possible next clauses
    for (clause_length_index_j, cnf_index_j) in length_index[clause_length_j]
        .iter()
        .copied()
        .enumerate()
        .skip(amount_to_skip)
    {
        // let swap = cnf_index_i >= cnf_index_j;
        // let (e, f, d1, d2) = if swap {
        //     (cnf_index_j, cnf_index_i, b.len(), a.len())
        // } else {
        //     (cnf_index_i, cnf_index_j)
        // };

        // if the diff is too large, skip
        let diff = if let Some(x) = s.calculate(cnf_index_i, cnf_index_j, a.len(), b.len()) {
            x
        } else {
            // diff too large
            continue;
        };

        // swap diff
        let (diff_i, diff_j) = (diff.i, diff.j);

        // if the diff is not exactly the diff we are looking for, skip
        if !(diff_i.len() == a.len() && diff_j.len() == b.len()) {
            continue;
        }

        // to do, there should be a check that the diff matches the current mapping of variables.
        if !variable_mapping.update_variable_mapping(diff_i, diff_j, a, b) {
            // println!("After Unsuccessful Update:\n{}", variable_mapping);
            continue;
        }
        // println!("CNF index i {}, CNF index j {}", cnf_index_i, cnf_index_j);
        // println!("After Successful Update:\n{}", variable_mapping);

        indices_so_far.push((clause_length_j, clause_length_index_j));
        match_starting_at_recursive(
            length_index,
            s,
            p,
            ip,
            cnf_index,
            variable_mapping,
            indices_so_far,
            m,
        );
        indices_so_far.pop();
        variable_mapping.undo();

        // println!("After Undo:\n{}", variable_mapping);
    }
}

fn match_pattern_on_cnf<'a>(
    length_index: &[Vec<usize>],
    s: &mut DiffCache,
    p: &'a BVA2Pattern,
    ip: &InternalPatternMetaData,
    cnf_index: usize,
    m: &mut PatternMatches<'a>,
) {
    let mut indices_so_far = Vec::with_capacity(p.inputs.len());
    let mut variable_mapping = VariableMapping::new(
        ip.vars_that_need_mapping.clone(),
        p.inputs.len(),
        ip.input_diffs
            .iter()
            .flat_map(|d| d.iter())
            .map(|(v1, v2)| std::cmp::max(v1.len(), v2.len()))
            .max()
            .unwrap(),
        p.neq.clone(),
    );

    for (clause_length, clauses_of_length_k) in length_index.iter().enumerate() {
        for clause_length_index in 0..clauses_of_length_k.len() {
            indices_so_far.push((clause_length, clause_length_index));
            // println!(
            //     "clause_length = {}/{}, clause_length_index = {}/{}",
            //     clause_length,
            //     length_index.len(),
            //     clause_length_index,
            //     clauses_of_length_k.len()
            // );
            match_starting_at_recursive(
                length_index,
                s,
                p,
                ip,
                cnf_index,
                &mut variable_mapping,
                &mut indices_so_far,
                m,
            );
            indices_so_far.pop();
            debug_assert!(variable_mapping.is_empty());
            debug_assert!(indices_so_far.is_empty());
        }
    }
}

fn find_most_pattern_that_can_coexist(
    indices_of_matches_to_consider: &[usize],
    matches: &PatternMatches,
) -> Vec<usize> {
    // for each clause, count the patterns that use it
    let mut clause_index_histogram = FxHashMap::with_capacity_and_hasher(
        indices_of_matches_to_consider.len(),
        Default::default(),
    );
    for i in indices_of_matches_to_consider.iter().copied() {
        for j in matches.v[i].clause_indices.iter().copied() {
            *clause_index_histogram.entry(j).or_insert(0) += 1;
        }
    }

    // for each pattern, count the max clause index that it uses
    let mut patterns: Vec<_> = indices_of_matches_to_consider
        .iter()
        .map(|i| (*i, &matches.v[*i]))
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

// ************************************************************************************************
// API
// ************************************************************************************************

impl CNF {
    /// Match a set of patterns on a set of CNFs.
    ///
    pub fn bva2_match_patterns_on_cnfs<'a>(
        cnfs: &[Vec<Clause>],
        patterns: &'a [BVA2Pattern],
    ) -> PatternMatches<'a> {
        let internal_patterns: Vec<InternalPatternMetaData> = patterns
            .iter()
            .enumerate()
            .map(|(i, p)| InternalPatternMetaData::new(p, i))
            .collect();

        let mut m: PatternMatches<'a> = PatternMatches { v: vec![] };

        for (cnf_index, cnf) in cnfs.iter().enumerate() {
            if cnf.is_empty() {
                continue;
            }

            // make index to cnf clauses by clause length
            let n = cnf.len();
            let max_clause_length = cnf.iter().map(|x| x.len()).max().unwrap();
            let mut length_index = vec![Vec::with_capacity(n); max_clause_length + 1];
            for (i, c) in cnf.iter().enumerate() {
                length_index[c.len()].push(i);
            }

            let max_diff = internal_patterns.iter().map(|x| x.max_diff).max().unwrap();

            // make graph given parameters
            let mut d = DiffCache::new(max_diff, max_diff, cnf);
            for (p, ip) in patterns.iter().zip(internal_patterns.iter()) {
                match_pattern_on_cnf(&length_index, &mut d, p, ip, cnf_index, &mut m);
            }
        }

        m
    }

    /// Returns a hashmap that maps each set of added variables to the indexes of the pattern matches that use them.
    pub fn bva2_convert_pattern_matches_into_definition_histogram(
        matches: &PatternMatches,
    ) -> FxHashMap<Vec<(DefinitionFunction, Vec<Literal>)>, Vec<usize>> {
        let mut match_map =
            FxHashMap::with_capacity_and_hasher(matches.v.len(), Default::default());
        for (i, m) in matches.v.iter().enumerate() {
            let mut defs: Vec<(DefinitionFunction, Vec<Literal>)> = vec![];
            for (_, f, inputs) in m.pattern.definitions.iter() {
                let mut inputs: Vec<Literal> = inputs
                    .iter()
                    .map(|x| matches.v[i].convert_literal(*x).unwrap())
                    .collect();
                inputs.sort_unstable();
                defs.push((*f, inputs));
            }
            defs.sort_unstable();
            match_map.entry(defs).or_insert_with(Vec::new).push(i);
        }
        match_map
    }

    pub fn bva2_get_definition_candidates(
        matches: &PatternMatches,
        remove_patterns_that_cannot_co_exist: bool,
    ) -> FxHashMap<Vec<(DefinitionFunction, Vec<Literal>)>, Vec<usize>> {
        if matches.v.is_empty() {
            return Default::default();
        }

        // cluster pattern matches that agree on the extension variables.
        let mut match_map = Self::bva2_convert_pattern_matches_into_definition_histogram(matches);

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
    pub fn bva2_get_best_set_of_patterns_to_add(
        matches: &PatternMatches,
        remove_patterns_that_cannot_co_exist: bool,
    ) -> Vec<usize> {
        let match_map =
            Self::bva2_get_definition_candidates(matches, remove_patterns_that_cannot_co_exist);
        match_map.into_iter().max_by_key(|x| x.1.len()).unwrap().1
    }
}
