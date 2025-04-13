// ************************************************************************************************
// use
// ************************************************************************************************

use super::Frames;
use crate::{
    engines::pdr::PropertyDirectedReachabilitySolver,
    formulas::{
        cnf::bounded_variable_addition_3::{BVA3Parameters, BVA3Pattern},
        Clause, Literal, CNF,
    },
    function,
    models::{
        time_stats::function_timer::FunctionTimer, UniqueSortedHashMap, UniqueSortedVec, Utils,
    },
    solvers::{dd::DecisionDiagramManager, sat::incremental::CaDiCalSolver},
};
// use fxhash::{FxBuildHasher, FxHashMap};

// ************************************************************************************************
// type
// ************************************************************************************************

// type Diff = (Vec<Literal>, Vec<Literal>);
// type FrameIndex = usize;
// type ClauseDiffs = FxHashMap<Diff, Vec<(FrameIndex, usize, usize)>>;
type ToRemove = Vec<(usize, usize)>;

// ************************************************************************************************
// impl
// ************************************************************************************************

impl<T: PropertyDirectedReachabilitySolver, D: DecisionDiagramManager> Frames<T, D> {
    // ********************************************************************************************
    // helper functions
    // ********************************************************************************************

    fn _print_frame(&self, title: &str, k: usize) {
        if self.s.parameters.should_print_bva_debug_information {
            println!("{} (k = {}):", title, k);
            println!(
                "{}",
                CNF::from_sequence(self.frames[k].get_delta_clauses_cloned().to_owned())
            );
            println!("{}", self.definition_library);
        }
    }

    fn check_bva_equivalence_under_definitions(
        &mut self,
        k: usize,
        delta_frame_before: &[Clause],
    ) -> bool {
        let delta_frame_after = self.frames[k].get_delta_clauses_cloned().to_owned();
        let add_definitions = |mut frame: Vec<Clause>| {
            for d in self.definition_library.get_definitions() {
                frame.append(&mut d.to_cnf());
            }
            CNF::from_sequence(frame)
        };

        if delta_frame_after != delta_frame_before {
            debug_assert!(Utils::does_a_imply_b::<CaDiCalSolver>(
                &add_definitions(delta_frame_before.to_vec()),
                &CNF::from_sequence(delta_frame_after.clone())
            )
            .unwrap_or(true));
            debug_assert!(Utils::does_a_imply_b::<CaDiCalSolver>(
                &add_definitions(delta_frame_after),
                &CNF::from_sequence(delta_frame_before.to_vec())
            )
            .unwrap_or(true));
        }

        true
    }

    // fn same_vars(a: &[Literal], b: &[Literal]) -> bool {
    //     debug_assert_eq!(a.len(), b.len());
    //     a.iter()
    //         .zip(b.iter())
    //         .all(|(x, y)| x.variable() == y.variable())
    // }

    // fn almost_same_vars(a: &[Literal], b: &[Literal]) -> bool {
    //     // debug_assert_eq!(a.len());
    //     debug_assert_eq!(a.len(), 3);
    //     debug_assert_eq!(b.len(), 2);

    //     b.iter().all(|vb| a.contains(&!*vb))
    // }

    // fn diff_pass(&self) -> ClauseDiffs {
    //     const ALLOW_UNIT_IN_BVA: bool = false;
    //     const MAX_DIFF: usize = 3;
    //     let mut subtraction_vec_1 = Vec::with_capacity(MAX_DIFF);
    //     let mut subtraction_vec_2 = Vec::with_capacity(MAX_DIFF);
    //     let cap = 2 * self.frames.iter().map(|f| f.len()).sum::<usize>();
    //     let hasher = FxBuildHasher::default();
    //     let mut result: ClauseDiffs = FxHashMap::with_capacity_and_hasher(cap, hasher);

    //     for (f, frame) in self.frames.iter().enumerate() {
    //         let clauses = frame.get_delta_clauses();
    //         for (i, c1) in clauses.iter().enumerate() {
    //             for (j, c2) in clauses.iter().enumerate().skip(i + 1) {
    //                 if c1.len().abs_diff(c2.len()) > 1 {
    //                     continue;
    //                 }

    //                 if (!ALLOW_UNIT_IN_BVA) && c1.len() <= 1 {
    //                     continue;
    //                 }

    //                 subtraction_vec_1.clear();
    //                 subtraction_vec_2.clear();

    //                 c1.peek().peek().symmetric_difference_custom(
    //                     c2.peek().peek(),
    //                     MAX_DIFF,
    //                     MAX_DIFF,
    //                     &mut subtraction_vec_1,
    //                     &mut subtraction_vec_2,
    //                 );

    //                 let mut should_add = false;

    //                 should_add |= subtraction_vec_1.len() == 1
    //                     && subtraction_vec_2.len() == 1
    //                     && ((!subtraction_vec_1[0]) != subtraction_vec_2[0]);

    //                 should_add |= subtraction_vec_1.len() == 2
    //                     && subtraction_vec_2.len() == 2
    //                     && Self::same_vars(&subtraction_vec_1, &subtraction_vec_2);

    //                 // should_add |= subtraction_vec_1.len() == 3
    //                 //     && subtraction_vec_2.len() == 2
    //                 //     && Self::almost_same_vars(&subtraction_vec_1, &subtraction_vec_2);

    //                 // should_add |= subtraction_vec_1.len() == 2
    //                 //     && subtraction_vec_2.len() == 3
    //                 //     && Self::almost_same_vars(&subtraction_vec_2, &subtraction_vec_1);

    //                 if should_add {
    //                     let (entry, value) = if subtraction_vec_1 < subtraction_vec_2 {
    //                         (
    //                             (subtraction_vec_1.to_vec(), subtraction_vec_2.to_vec()),
    //                             (f, i, j),
    //                         )
    //                     } else {
    //                         (
    //                             (subtraction_vec_2.to_vec(), subtraction_vec_1.to_vec()),
    //                             (f, j, i),
    //                         )
    //                     };
    //                     result.entry(entry).or_default().push(value);
    //                 }
    //             }
    //         }
    //     }
    //     result
    // }

    // fn get_new_clause(d: &Definition, old_clause_a: &Clause, old_clause_b: &Clause) -> Clause {
    //     let a = d.inputs.peek().peek()[0];
    //     let b = d.inputs.peek().peek()[1];
    //     match d.function {
    //         DefinitionFunction::And => {
    //             debug_assert!(old_clause_a.contains(&a));
    //             debug_assert!(old_clause_b.contains(&b));
    //             let mut new_clause = old_clause_a.peek().peek().clone();
    //             new_clause.remove(&a);
    //             new_clause.insert(d.variable.literal(false));
    //             Clause::from_ordered_set(new_clause.unpack())
    //         }
    //         DefinitionFunction::Xor => {
    //             let x = if old_clause_a.contains(&a) { a } else { !a };
    //             let y = if old_clause_a.contains(&b) { b } else { !b };
    //             debug_assert!(old_clause_a.contains(&x));
    //             debug_assert!(old_clause_a.contains(&y));
    //             debug_assert!(old_clause_b.contains(&!x));
    //             debug_assert!(old_clause_b.contains(&!y));
    //             let mut new_clause = old_clause_a.peek().peek().clone();
    //             new_clause.remove(&x);
    //             new_clause.remove(&y);
    //             let is_negated = (x == a) ^ (y == b);
    //             new_clause.insert(d.variable.literal(is_negated));
    //             Clause::from_ordered_set(new_clause.unpack())
    //         }
    //     }
    // }

    // fn apply_definition(&mut self, i: usize, diffs: &ClauseDiffs) -> (ToRemove, Vec<usize>) {
    //     let d = &self.definition_library.get_definitions()[i];
    //     let mut to_remove = vec![];
    //     let mut to_add = vec![];
    //     let mut added = vec![0; self.frames.len()];
    //     let keys = match d.function {
    //         DefinitionFunction::And => {
    //             let a = d.inputs.peek().peek()[0];
    //             let b = d.inputs.peek().peek()[1];
    //             let diff = (vec![a], vec![b]);
    //             let not_diff = (vec![b], vec![a]);
    //             debug_assert!(!diffs.contains_key(&not_diff));
    //             vec![diff]
    //         }
    //         DefinitionFunction::Xor => {
    //             let a = d.inputs.peek().peek()[0];
    //             let b = d.inputs.peek().peek()[1];
    //             let diff_1 = (vec![a, !b], vec![!a, b]);
    //             let diff_2 = (vec![a, b], vec![!a, !b]);
    //             let not_diff_1 = (vec![!a, b], vec![a, !b]);
    //             let not_diff_2 = (vec![!a, !b], vec![a, b]);
    //             debug_assert!(!diffs.contains_key(&not_diff_1));
    //             debug_assert!(!diffs.contains_key(&not_diff_2));
    //             vec![diff_1, diff_2]
    //         }
    //     };

    //     for diff in keys.iter() {
    //         let set = if let Some(x) = diffs.get(diff) {
    //             x
    //         } else {
    //             continue;
    //         };

    //         for (f, i, j) in set.iter().copied() {
    //             to_remove.push((f, i));
    //             to_remove.push((f, j));

    //             let ca = self.frames[f].get_delta_at(i).clause();
    //             let cb = self.frames[f].get_delta_at(j).clause();
    //             let c = Self::get_new_clause(d, ca, cb);
    //             to_add.push((f, c));
    //         }
    //     }

    //     for (f, c) in to_add {
    //         let de = self.make_delta_element(c);
    //         self.frames[f].push_to_delta_without_solver(de);
    //         added[f] += 1;
    //     }

    //     (to_remove, added)
    // }

    // fn get_definition_input_from_diff(diff: &Diff) -> (Vec<Literal>, DefinitionFunction) {
    //     match diff.0.len() {
    //         1 => (vec![diff.0[0], diff.1[0]], DefinitionFunction::And),
    //         2 => {
    //             let l1 = diff.0[0];
    //             let l2 = diff.0[1];
    //             let f = DefinitionFunction::Xor;
    //             (vec![l1, l2], f)
    //         }
    //         _ => panic!("Unexpected diff size: {}", diff.0.len()),
    //     }
    // }

    // fn get_diff_from_definition(d: &Definition) -> Diff {
    //     let inputs = d.inputs.peek().peek();
    //     match d.function {
    //         DefinitionFunction::And => Diff,
    //         DefinitionFunction::Xor => todo!(),
    //         // 2 => (vec![inputs[0], inputs[1]], vec![]),
    //         // 1 => (vec![inputs[0]], vec![inputs[1]]),
    //         // _ => panic!("Unexpected number of inputs: {}", inputs.len()),
    //     }
    // }

    fn _get_largest_used_literal_in_all_frames(&self) -> Literal {
        let largest_var = self
            .definition_library
            .iter()
            .last()
            .map(|d| d.variable)
            .unwrap_or_else(|| {
                *self
                    .s
                    .fin_state
                    .borrow()
                    .get_state_variables()
                    .max()
                    .unwrap()
            });

        std::cmp::max(largest_var.literal(true), largest_var.literal(false))
    }

    fn _make_index_for_each_literal(
        &self,
        f: usize,
    ) -> UniqueSortedHashMap<Literal, UniqueSortedVec<usize>> {
        let largest_lit = self._get_largest_used_literal_in_all_frames();
        let mut map: UniqueSortedHashMap<Literal, UniqueSortedVec<usize>> =
            UniqueSortedHashMap::new(largest_lit);

        for (i, c) in self.frames[f].get_delta().iter().enumerate() {
            for l in c.clause().iter() {
                let e = if let Some(e) = map.get_mut(l) {
                    e
                } else {
                    map.insert(*l, UniqueSortedVec::new());
                    map.get_mut(l).unwrap()
                };
                e.push(i);
            }
        }

        map
    }

    // fn apply_definitions_that_already_exist(&mut self) {
    //     // for d in self.definition_library.iter() {
    //     //     for (f, frame) in self.frames.iter().enumerate() {
    //     //         let index = self.make_index_for_each_literal(f);

    //     //         // for (i, c1) in clauses.iter().enumerate() {
    //     //         //     for (j, c2) in clauses.iter().enumerate().skip(i + 1) {}
    //     //         // }
    //     //     }
    //     // }
    // }

    pub(super) fn remove(&mut self, to_remove: &ToRemove) {
        if to_remove.is_empty() {
            return;
        }

        let mut to_remove_per_frame = vec![vec![]; self.frames.len()];
        for (f, i) in to_remove {
            to_remove_per_frame[*f].push(*i);
        }
        for (f, indexes) in to_remove_per_frame.into_iter().enumerate() {
            let indexes = UniqueSortedVec::from_sequence(indexes);
            self.frames[f].remove_multiple_from_delta(&indexes);
        }
    }

    // fn choose_definition_from_diffs(
    //     &self,
    //     diffs: &ClauseDiffs,
    // ) -> Option<(SortedVecOfLiterals, DefinitionFunction)> {
    //     if self.s.parameters.should_print_bva_debug_information {
    //         println!("Diffs:");
    //         for (k, v) in diffs.iter() {
    //             println!("({:?}, {:?}) -> {:?}", k.0, k.1, v);
    //         }
    //     }

    //     let mut best = None;
    //     for (d, v) in diffs.iter() {
    //         let matched_count = match d.0.len() {
    //             1 => v.len(),
    //             2 => {
    //                 let mut count = v.len();
    //                 let inp1 = vec![d.0[0], !d.0[1]];
    //                 let inp2 = vec![!d.0[0], d.0[1]];
    //                 let inp3 = vec![!d.0[0], !d.0[1]];

    //                 // let mut matched_diffs = vec![];
    //                 for (a, possible) in [(inp1, true), (inp2, false), (inp3, false)] {
    //                     let not_a = vec![!a[0], !a[1]];
    //                     let e = diffs.get_key_value(&(a, not_a));
    //                     if let Some((_, x)) = e {
    //                         debug_assert!(possible);
    //                         count += x.len();
    //                         // matched_diffs.push(k);
    //                     }
    //                 }
    //                 count
    //             }
    //             _ => unreachable!(),
    //         };

    //         if let Some((_, best_count)) = best {
    //             if matched_count > best_count {
    //                 best = Some((d, matched_count));
    //             }
    //         } else {
    //             best = Some((d, matched_count));
    //         }
    //     }

    //     let (diff, count) = best?;

    //     if count < self.s.parameters.min_pair_count_to_add_definition {
    //         return None;
    //     }

    //     let (input, f) = Self::get_definition_input_from_diff(diff);
    //     let input = SortedVecOfLiterals::from_ordered_set(input);

    //     if self.s.parameters.should_print_bva_debug_information {
    //         println!("A new extension variable is chosen:");
    //         println!("Best diff: {:?}", diff);
    //         println!("Score: {}", count);
    //         println!("Inputs: {:?}", input);
    //         println!("Function: {}", f);
    //     }

    //     Some((input, f))
    // }

    fn perform_bva(&mut self) -> Vec<usize> {
        let _timer = FunctionTimer::start(function!(), self.s.time_stats.clone());

        // get all cnfs
        let cnfs: Vec<Vec<Clause>> = self
            .frames
            .iter()
            .map(|f| f.get_delta_clauses_cloned())
            .collect();

        let r = CNF::bva3_match_patterns_on_cnfs(
            &cnfs,
            BVA3Parameters {
                and_pattern: true,
                xor_pattern: true,
                half_adder_pattern: false,
            },
        );

        if r.is_empty() {
            return vec![];
        };

        let clustered_matches = CNF::bva3_get_definition_candidates(&r, false);

        let best = clustered_matches
            .into_iter()
            .max_by_key(|(p, i)| {
                i.len()
                    + match p {
                        BVA3Pattern::AndPattern(..) => 0,
                        BVA3Pattern::XorPattern(..) => r.len(),
                        BVA3Pattern::HalfAdderPattern(..) => r.len() * 2,
                    }
            })
            .unwrap();

        if best.1.len() < self.s.parameters.min_match_count_to_add_definition {
            return vec![];
        }

        // let chosen_matched_pattern = &r.v[*matches.first().unwrap()];
        // let chosen_pattern = chosen_matched_pattern.pattern;
        let mut to_remove = Vec::with_capacity(self.len());
        let mut to_add = Vec::with_capacity(self.len());

        for m in best.1.iter().map(|x| &r[*x]) {
            let cnf_i = m.cnf_index;
            let cnf = &cnfs[cnf_i];
            let indexes_of_clauses_before = m
                .clause_indices
                .iter()
                .map(|p1| cnf.iter().position(|p2| std::ptr::eq(*p1, p2)).unwrap());

            // make the resulting clauses
            let (clauses_after, _) = m.get_resulting_clauses(|f, inputs| {
                let already_exists = self.definition_library.position(f, &inputs);
                if let Some((i, is_negated)) = already_exists {
                    self.definition_library.at(i).variable.literal(is_negated)
                } else {
                    let (i, is_negated) = self
                        .definition_library
                        .add_definition(f, inputs.clone())
                        .unwrap();
                    self.definition_library.at(i).variable.literal(is_negated)
                }
            });

            to_remove.extend(indexes_of_clauses_before.into_iter().map(|x| (cnf_i, x)));
            to_add.extend(clauses_after.into_iter().map(|x| (cnf_i, x)));
        }

        // remove the old clauses
        self.remove(&to_remove);

        // add the new clauses
        let mut added = vec![0; self.len()];
        for (f, c) in to_add.iter() {
            let de = self.make_delta_element(c.clone());
            self.frames[*f].push_to_delta(de);
            added[*f] += 1;
        }

        added
    }

    fn fix_redundancy(&mut self, added_clauses_per_frame: Vec<usize>) {
        let _timer = FunctionTimer::start(function!(), self.s.time_stats.clone());
        if added_clauses_per_frame.is_empty() {
            return;
        }

        debug_assert_eq!(added_clauses_per_frame.len(), self.frames.len());

        // println!("added_clauses_per_frame = {:?}", added_clauses_per_frame);

        let mut to_remove = vec![];
        // #[allow(clippy::needless_range_loop)]
        for (j, fj) in self.frames.iter().enumerate().skip(1) {
            for (y, cj) in fj
                .get_delta()
                .iter()
                .enumerate()
                .rev()
                .take(added_clauses_per_frame[j])
            {
                for (i, fi) in self.frames.iter().enumerate().take(j + 1) {
                    // #[allow(clippy::needless_range_loop)]
                    for (x, ci) in fi.get_delta().iter().enumerate() {
                        if i == j && x == y {
                            continue;
                        }
                        if Self::does_a_imply_b(cj, ci, &mut self.definition_library, &self.s) {
                            // fix
                            to_remove.push((i, x));
                        }
                    }
                }
            }
        }

        // println!("to_remove when fixing redundancy = {:?}", to_remove);
        self.remove(&to_remove);
    }

    pub(super) fn get_deltas(&self) -> Vec<Vec<Clause>> {
        self.frames
            .iter()
            .map(|f| f.get_delta_clauses_cloned())
            .collect()
    }

    fn get_deltas_sorted(&self) -> Vec<UniqueSortedVec<Clause>> {
        self.frames
            .iter()
            .map(|f| UniqueSortedVec::from_sequence(f.get_delta_clauses_cloned()))
            .collect()
    }

    fn print_before_bva(&self) {
        if self.s.parameters.should_print_bva_debug_information {
            println!("Before BVA:");
            for (k, f) in self.frames.iter().enumerate() {
                println!("Frame {}:", k);
                for (i, c) in f.get_delta_clauses().iter().enumerate() {
                    println!("{}: {}", i, c);
                }
            }
        }
    }

    fn print_after_bva(&self, deltas_before: &[UniqueSortedVec<Clause>]) {
        if self.s.parameters.should_print_bva_debug_information {
            let deltas_after = self.get_deltas_sorted();
            println!("Delta Change:");
            for (k, _) in self.frames.iter().enumerate() {
                let removed = deltas_before[k]
                    .clone()
                    .subtract_consuming(deltas_after[k].clone());
                let added = deltas_after[k]
                    .clone()
                    .subtract_consuming(deltas_before[k].clone());
                println!("Frame {k}:");
                println!("Removed:");
                for c in removed.iter() {
                    println!("{}", c);
                }
                println!("Added:");
                for c in added.iter() {
                    println!("{}", c);
                }
            }
            println!("Definitions:");
            println!("{}", self.definition_library);
        }
    }

    // ********************************************************************************************
    // API
    // ********************************************************************************************

    pub fn condense_frames_by_defining_new_variables(&mut self) {
        let _timer = FunctionTimer::start(function!(), self.s.time_stats.clone());
        debug_assert!(self.regression_check());
        // PDRStats::print_memory_usage("Before BVA");

        let lib_size_before = self.definition_library.len();

        let deltas_before = self.get_deltas_sorted();

        // PDRStats::print_memory_usage("after getting deltas");

        self.print_before_bva();

        let r = self.perform_bva();

        // PDRStats::print_memory_usage("after BVA");

        self.print_after_bva(&deltas_before);

        for (k, delta_before) in deltas_before.iter().enumerate() {
            debug_assert!(self.check_bva_equivalence_under_definitions(k, delta_before.peek()));
        }

        let lib_size_after = self.definition_library.len();
        for i in lib_size_before..lib_size_after {
            let d = &self.definition_library.get_definitions()[i];
            // Self::add_definition_to_all_frames(&mut self.frames, d);
            self.solvers.add_new_definition(d);
        }

        // PDRStats::print_memory_usage("After adding definitions to solvers");

        self.fix_redundancy(r);

        self.solvers.rest_solvers(self.get_deltas());

        // PDRStats::print_memory_usage("After fixing redundancy");

        debug_assert!(self.regression_check());
    }

    pub fn call_condense(&mut self) {
        let _timer = FunctionTimer::start(function!(), self.s.time_stats.clone());

        if self.s.parameters.er {
            self.extension_variables_counter += 1;
            let s = self.extension_variables_counter;
            let n = self.s.parameters.er_delta;
            if s % n == 0 {
                self.condense_frames_by_defining_new_variables();
            }
        }
        debug_assert!(self.sanity_check());
    }
}

// ************************************************************************************************
// tests
// ************************************************************************************************
