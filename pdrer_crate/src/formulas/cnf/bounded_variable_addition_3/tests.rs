// ************************************************************************************************
// use
// ************************************************************************************************

#[allow(unused_imports)]
use crate::{
    formulas::cnf::bounded_variable_addition_3::{BVA3Parameters, BVA3Pattern},
    models::Definition,
};
#[allow(unused_imports)]
use crate::{
    formulas::{cnf::bounded_variable_addition_2::BVA2Pattern, Clause, Literal, Variable, CNF},
    models::Utils,
    solvers::sat::incremental::CaDiCalSolver,
};
#[allow(unused_imports)]
use fxhash::FxHashMap;

use super::BVA3PatternMatch;

// ************************************************************************************************
// tests
// ************************************************************************************************

fn _check_pattern_match_correctness(matches: &BVA3PatternMatch) {
    let clauses_before: Vec<Clause> = matches
        .get_clauses_in_pattern()
        .iter()
        .copied()
        .cloned()
        .collect();

    let mut max_variable = clauses_before
        .iter()
        .map(|x| x.max_variable())
        .max()
        .unwrap();

    let mut defs = vec![];

    let (clauses_after, _) = matches.get_resulting_clauses(|f, i| {
        max_variable.bump(1);
        let r = max_variable.literal(false);
        defs.push(Definition {
            variable: r.variable(),
            function: f,
            inputs: i,
        });
        r
    });

    let defs: Vec<Clause> = defs.iter().flat_map(|d| d.to_cnf()).collect();

    // println!("Checking that two clause sets are equivalent under definitions:");
    // println!(
    //     "Before: {}",
    //     clauses_before
    //         .iter()
    //         .map(|x| x.to_string())
    //         .collect::<Vec<String>>()
    //         .join(", ")
    // );
    // println!(
    //     "After: {}",
    //     clauses_after
    //         .iter()
    //         .map(|x| x.to_string())
    //         .collect::<Vec<String>>()
    //         .join(", ")
    // );
    // println!(
    //     "Defs: {}",
    //     defs.iter()
    //         .map(|x| x.to_string())
    //         .collect::<Vec<String>>()
    //         .join(", ")
    // );

    assert!(Utils::does_a_imply_b::<CaDiCalSolver>(
        {
            let mut a = clauses_before.clone();
            a.extend(defs.clone());
            &CNF::from_sequence(a)
        },
        &CNF::from_sequence(clauses_after.clone()),
    )
    .unwrap_or(true));

    assert!(Utils::does_a_imply_b::<CaDiCalSolver>(
        {
            let mut a = clauses_after.clone();
            a.extend(defs);
            &CNF::from_sequence(a)
        },
        &CNF::from_sequence(clauses_before.clone()),
    )
    .unwrap_or(true));
}

// cargo test --package rust-formal-verification --lib -- formulas::cnf::bounded_variable_addition::advanced::test_bva_1 --exact --nocapture
#[test]
fn test_bva_1() {
    let a = Literal::new(Variable::new(10));
    let b = Literal::new(Variable::new(11));
    let c = Literal::new(Variable::new(12));
    let d = Literal::new(Variable::new(13));
    let e = Literal::new(Variable::new(14));
    // let f = Literal::new(Variable::new(15));
    // let g = Literal::new(Variable::new(16));

    let cnf = vec![
        // and pattern
        Clause::from_ordered_set(vec![a, c, d, e]),
        Clause::from_ordered_set(vec![b, c, d, e]),
        // xor pattern
        Clause::from_ordered_set(vec![!a, b]),
        Clause::from_ordered_set(vec![a, !b]),
        // fake xor pattern
        Clause::from_ordered_set(vec![!a, !b]),
        Clause::from_ordered_set(vec![a, c]),
        // xor pattern 2
        Clause::from_ordered_set(vec![a, b, c]),
        Clause::from_ordered_set(vec![a, b, d]),
        Clause::from_ordered_set(vec![!a, !b, c, d]),
    ];
    let cnfs = vec![cnf];

    let r = CNF::bva3_match_patterns_on_cnfs(
        &cnfs,
        BVA3Parameters {
            and_pattern: true,
            xor_pattern: true,
            half_adder_pattern: true,
        },
    );

    for x in r.iter() {
        println!(
            "Pattern match: cnf_i = {}, clauses = {:?}, pattern = {:?}",
            x.cnf_index,
            x.clause_indices
                .iter()
                .map(|x| x.to_string())
                .collect::<Vec<String>>(),
            x.pattern
        );

        _check_pattern_match_correctness(x);
    }
}

/// This tests that every pattern that is found is correct and is indeed SAT equivalent.
#[test]
fn test_bva_2() {
    use crate::{
        formulas::{Variable, CNF},
        models::UniqueSortedVec,
    };
    use rand::seq::IteratorRandom;
    use rand::{
        rngs::{StdRng, ThreadRng},
        Rng, SeedableRng,
    };

    let and_pattern = BVA2Pattern::and_pattern();
    let xor_pattern = BVA2Pattern::xor_pattern();
    let xor_pattern_2 = BVA2Pattern::xor_pattern_2();
    let patterns = vec![
        and_pattern.clone(),
        xor_pattern.clone(),
        xor_pattern_2.clone(),
    ];

    const N: usize = 1000;
    const CNF_SIZE: usize = 20;
    const MAX_CLAUSE_LENGTH: usize = 5;
    let mut counters = [0, 0, 0];
    for i in 0..=N {
        let seed: u64 = ThreadRng::default().gen();
        println!("i = {}, seed = {seed}\t, counters = {:?}", i, counters);
        let rng = &mut StdRng::seed_from_u64(seed);
        let variables = UniqueSortedVec::from_ordered_set(vec![
            Variable::new(10),
            Variable::new(11),
            Variable::new(12),
            Variable::new(13),
            Variable::new(14),
            Variable::new(15),
            // Variable::new(16),
        ]);
        let cnf_1 = CNF::custom_random(rng, &variables, 0.5, CNF_SIZE, |rng, _| {
            (1..=MAX_CLAUSE_LENGTH).choose(rng).unwrap()
        })
        .unpack()
        .unpack();
        let cnf_2 = CNF::custom_random(rng, &variables, 0.5, CNF_SIZE, |rng, _| {
            (1..=MAX_CLAUSE_LENGTH).choose(rng).unwrap()
        })
        .unpack()
        .unpack();
        let cnfs = [cnf_1, cnf_2];

        let r = CNF::bva3_match_patterns_on_cnfs(
            &cnfs,
            BVA3Parameters {
                and_pattern: true,
                xor_pattern: true,
                half_adder_pattern: true,
            },
        );
        let _ = CNF::bva3_get_best_set_of_patterns_to_add(&r, true);

        for x in r.iter() {
            // println!(
            //     "Pattern match: pid = {}, cid = {}, clauses = {:?}",
            //     x.pattern_id,
            //     x.cnf_index,
            //     x.clause_indices
            //         .iter()
            //         .map(|i| cnfs[x.cnf_index][*i].to_string())
            //         .collect::<Vec<String>>()
            // );
            _check_pattern_match_correctness(x);
            match x.pattern {
                BVA3Pattern::AndPattern { .. } => counters[0] += 1,
                BVA3Pattern::XorPattern { .. } => counters[1] += 1,
                BVA3Pattern::HalfAdderPattern { .. } => counters[2] += 1,
            }
        }

        // now check that every pattern that can be found is indeed found
        if patterns.contains(&xor_pattern_2) {
            for (cnf_i, cnf) in cnfs.iter().enumerate() {
                for (i, ci) in cnf.iter().enumerate() {
                    for (j, cj) in cnf.iter().enumerate() {
                        if ci.len() != cj.len() {
                            continue;
                        }

                        let (diff_a, diff_b) =
                            ci.peek().peek().symmetric_difference(cj.peek().peek());
                        if diff_a.len() != 1 || diff_b.len() != 1 {
                            continue;
                        }

                        let c = diff_a.peek()[0];
                        let d = diff_b.peek()[0];

                        for (k, ck) in cnf.iter().enumerate() {
                            if ck.len() != ci.len() + 1 {
                                continue;
                            }

                            let (diff_a, diff_b) =
                                ci.peek().peek().symmetric_difference(ck.peek().peek());

                            if diff_a.len() != 2 || diff_b.len() != 3 {
                                continue;
                            }

                            let a = diff_a.peek()[0];
                            let b = diff_a.peek()[1];

                            if (ci.contains(&a) && ci.contains(&b) && ci.contains(&c))
                                && (cj.contains(&a) && cj.contains(&b) && cj.contains(&d))
                                && (ck.contains(&!a)
                                    && ck.contains(&!b)
                                    && ck.contains(&c)
                                    && ck.contains(&d))
                            {
                                assert!(
                                    r.iter().any(|x| matches!(
                                        x.pattern,
                                        BVA3Pattern::HalfAdderPattern { .. }
                                    ) && x.cnf_index == cnf_i
                                        && x.clause_indices.len() == 3
                                        && x.clause_indices.contains(&ci)
                                        && x.clause_indices.contains(&cj)
                                        && x.clause_indices.contains(&ck)),
                                    "i = {}, ci = {}, j = {}, cj = {}, k = {}, ck = {}",
                                    i,
                                    ci,
                                    j,
                                    cj,
                                    k,
                                    ck
                                );
                            }
                        }
                    }
                }
            }
        }

        // check that all and patterns are found
        if patterns.contains(&and_pattern) {
            for (cnf_i, cnf) in cnfs.iter().enumerate() {
                for (i, ci) in cnf.iter().enumerate() {
                    for (j, cj) in cnf.iter().enumerate() {
                        if ci.len() != cj.len() {
                            continue;
                        }

                        let (diff_a, diff_b) =
                            ci.peek().peek().symmetric_difference(cj.peek().peek());
                        if diff_a.len() != 1 || diff_b.len() != 1 {
                            continue;
                        }

                        let a = diff_a.peek()[0];
                        let b = diff_b.peek()[0];

                        if a.variable() == b.variable() {
                            continue;
                        }

                        assert!(
                            r.iter()
                                .any(|x| matches!(x.pattern, BVA3Pattern::AndPattern { .. })
                                    && x.cnf_index == cnf_i
                                    && x.clause_indices.len() == 2
                                    && x.clause_indices.contains(&ci)
                                    && x.clause_indices.contains(&cj)),
                            "i = {}, ci = {}, j = {}, cj = {}",
                            i,
                            ci,
                            j,
                            cj
                        );
                    }
                }
            }
        }

        // check that all XOR variables are found:
        if patterns.contains(&xor_pattern) {
            for (cnf_i, cnf) in cnfs.iter().enumerate() {
                for (i, ci) in cnf.iter().enumerate() {
                    for (j, cj) in cnf.iter().enumerate() {
                        if ci.len() != cj.len() {
                            continue;
                        }

                        let (diff_a, diff_b) =
                            ci.peek().peek().symmetric_difference(cj.peek().peek());
                        if diff_a.len() != 2 || diff_b.len() != 2 {
                            continue;
                        }

                        let a = diff_a.peek()[0];
                        let b = diff_a.peek()[1];

                        if diff_b.contains(&!a) && diff_b.contains(&!b) {
                            assert!(
                                r.iter().any(|x| matches!(
                                    x.pattern,
                                    BVA3Pattern::XorPattern { .. }
                                ) && x.cnf_index == cnf_i
                                    && x.clause_indices.len() == 2
                                    && x.clause_indices.contains(&ci)
                                    && x.clause_indices.contains(&cj)),
                                "i = {}, ci = {}, j = {}, cj = {}",
                                i,
                                ci,
                                j,
                                cj
                            );
                        }
                    }
                }
            }
        }
    }
}

/// This tests the performance of the BVA algorithm on very large CNFs
///
/// cargo test --release --package rust-formal-verification --lib -- formulas::cnf::bounded_variable_addition_3::tests::test_bva_3 --exact --nocapture
#[test]
fn test_bva_3() {
    use crate::{
        formulas::{Variable, CNF},
        models::UniqueSortedVec,
    };
    use rand::seq::IteratorRandom;
    use rand::{
        rngs::{StdRng, ThreadRng},
        Rng, SeedableRng,
    };

    const N: usize = 1;
    const CNF_SIZE: usize = 10_000;
    const MAX_CLAUSE_LENGTH: usize = 10;
    let mut counters = [0, 0, 0];
    for i in 0..N {
        let seed: u64 = ThreadRng::default().gen();
        println!("i = {}, counters = {:?}", i, counters);
        let rng = &mut StdRng::seed_from_u64(seed);
        let variables = UniqueSortedVec::from_ordered_set((10..=40).map(Variable::new).collect());

        let cnf_1 = CNF::custom_random(rng, &variables, 0.5, CNF_SIZE, |rng, _| {
            (1..=MAX_CLAUSE_LENGTH).choose(rng).unwrap()
        })
        .unpack()
        .unpack();
        let cnfs = [cnf_1];

        println!("CNF size = {}", cnfs[0].len());

        let start_time = std::time::Instant::now();
        let r = CNF::bva3_match_patterns_on_cnfs(
            &cnfs,
            BVA3Parameters {
                and_pattern: true,
                xor_pattern: true,
                half_adder_pattern: true,
            },
        );
        let elapsed = start_time.elapsed();

        println!("Matched patterns = {}", r.len());

        let matches = CNF::bva3_get_best_set_of_patterns_to_add(&r, true);

        println!("Best set to add = {}", matches.1.len());

        for x in r.iter() {
            _check_pattern_match_correctness(x);
            match x.pattern {
                BVA3Pattern::AndPattern { .. } => counters[0] += 1,
                BVA3Pattern::XorPattern { .. } => counters[1] += 1,
                BVA3Pattern::HalfAdderPattern { .. } => counters[2] += 1,
            }
        }

        println!("Elapsed time = {}", elapsed.as_secs_f32());
    }
}

#[test]
fn test_bva_4() {
    use crate::formulas::CNF;

    let mut cnf = CNF::from_dimacs(_INVARIANT_OF_BUF_ALLOC_8)
        .unwrap()
        .unpack()
        .unpack();
    cnf.sort_by(|a, b| a.len().cmp(&b.len()).then(a.cmp(b)));

    let mut max_variable = cnf.iter().map(|x| x.max_variable()).max().unwrap();

    let mut definitions = vec![];
    let mut ev_map = FxHashMap::default();

    // println!("CNF:");
    // for x in cnf.iter() {
    //     println!("{}", x);
    // }

    let mut i = 0;
    loop {
        i += 1;
        println!("********************************************************\nCNF at iteration {i}:");
        for x in cnf.iter() {
            println!("{}", x);
        }
        println!("CNF size {}", cnf.len());

        let cnfs = [cnf.clone()];

        let mut r = CNF::bva3_match_patterns_on_cnfs(
            &cnfs,
            BVA3Parameters {
                and_pattern: false,
                xor_pattern: false,
                half_adder_pattern: true,
            },
        );
        if r.is_empty() {
            r = CNF::bva3_match_patterns_on_cnfs(
                &cnfs,
                BVA3Parameters {
                    and_pattern: true,
                    xor_pattern: false,
                    half_adder_pattern: false,
                },
            );
            if r.is_empty() {
                break;
            }
        }
        let best = CNF::bva3_get_best_set_of_patterns_to_add(&r, true);

        let chosen_matched_pattern = r
            .iter()
            .filter(|pm| pm.pattern == best.0)
            .next_back()
            .unwrap();
        // let chosen_pattern = chosen_matched_pattern.pattern;

        let defs = chosen_matched_pattern
            .get_resulting_clauses(|f, i| {
                let k = (f, i.to_owned());
                if let Some(r) = ev_map.get(&k) {
                    *r
                } else {
                    max_variable.bump(1);
                    let r = max_variable.literal(false);
                    ev_map.insert(k, r);
                    definitions.push(Definition {
                        variable: r.variable(),
                        function: f,
                        inputs: i,
                    });
                    r
                }
            })
            .1;

        let mut to_remove = Vec::with_capacity(cnf.len());
        let mut to_add = Vec::with_capacity(cnf.len());
        for m in best.1.iter().map(|x| &r[*x]) {
            let clauses_before: Vec<Clause> = m
                .get_clauses_in_pattern()
                .iter()
                .copied()
                .cloned()
                .collect();
            let (clauses_after, dd) = m.get_resulting_clauses(|f, i| {
                let k = (f, i);
                *ev_map.get(&k).unwrap()
            });
            assert_eq!(dd, defs);
            to_remove.extend(clauses_before);
            to_add.extend(clauses_after);
        }

        println!("Added definitions:");
        for d in definitions.iter() {
            println!(
                "var = {}\tfunction = {}\tinputs = {}",
                d.variable,
                d.function,
                d.inputs.peek()
            );
        }
        println!("Removing clauses:");
        for x in to_remove.iter() {
            println!("{}", x);
        }
        println!("Adding clauses:");
        for x in to_add.iter() {
            println!("{}", x);
        }

        for to_remove in to_remove {
            let a = cnf.len();
            cnf.retain(|x| x != &to_remove);
            assert!(a == (cnf.len() + 1));
        }

        for to_add in to_add {
            cnf.push(to_add);
        }
    }

    println!("Added definitions:");
    for d in definitions.iter() {
        println!(
            "var = {}\tfunction = {}\tinputs = {}",
            d.variable,
            d.function,
            d.inputs.peek()
        );
    }

    // let mut counters = [0, 0, 0];
    // for x in r.v.iter() {
    //     println!(
    //         "Pattern match: pid = {}, cid = {}, clauses = {:?}",
    //         x.pattern_id,
    //         x.cnf_index,
    //         x.clause_indices
    //             .iter()
    //             .map(|x| cnf[*x].to_string())
    //             .collect::<Vec<String>>()
    //     );
    //     counters[x.pattern_id] += 1;
    // }
}

// ************************************************************************************************
// Large invariant
// ************************************************************************************************

const _INVARIANT_OF_BUF_ALLOC_8: &str = "p cnf 22 310
-6 -9 0
-7 -9 0
-8 -9 0
6 7 8 9 -15 0
6 7 8 9 -16 0
-6 7 8 -15 -16 0
6 7 8 9 -17 0
-6 7 8 -15 -17 0
-6 7 8 -16 -17 0
6 -7 8 -15 -16 -17 0
6 7 8 9 -18 0
-6 7 8 -15 -18 0
-6 7 8 -16 -18 0
-6 7 8 -17 -18 0
6 7 8 16 -17 -18 0
6 -7 8 -15 -16 -18 0
6 -7 8 -15 -17 -18 0
6 -7 8 -16 -17 -18 0
-7 8 -15 -16 -17 -18 0
6 7 8 9 -19 0
-6 7 8 -15 -19 0
-6 7 8 -16 -19 0
-6 7 8 -17 -19 0
-6 7 8 -18 -19 0
7 8 9 -15 -19 0
7 8 16 -18 -19 0
6 7 8 9 -15 -19 0
6 -7 8 -15 -16 -19 0
6 -7 8 -15 -17 -19 0
6 -7 8 -15 -18 -19 0
6 -7 8 -16 -17 -19 0
6 -7 8 -16 -18 -19 0
6 -7 8 -17 -18 -19 0
-6 8 -15 -16 -18 -19 0
-6 8 -15 -17 -18 -19 0
-6 8 -16 -17 -18 -19 0
-7 8 -15 -16 -17 -19 0
-7 8 -15 -16 -18 -19 0
-7 8 -15 -17 -18 -19 0
-7 8 -16 -17 -18 -19 0
6 7 -8 -15 -16 -17 -18 -19 0
6 7 8 9 -20 0
-6 7 8 -15 -20 0
-6 7 8 -16 -20 0
-6 7 8 -17 -20 0
-6 7 8 -18 -20 0
-6 7 8 -19 -20 0
7 8 9 -17 -20 0
7 8 16 -19 -20 0
6 7 8 16 -17 -20 0
6 -7 8 -15 -16 -20 0
6 -7 8 -15 -17 -20 0
6 -7 8 -15 -18 -20 0
6 -7 8 -15 -19 -20 0
6 -7 8 -16 -17 -20 0
6 -7 8 -16 -18 -20 0
6 -7 8 -16 -19 -20 0
6 -7 8 -17 -18 -20 0
6 -7 8 -17 -19 -20 0
6 -7 8 -18 -19 -20 0
7 8 16 -17 -18 -20 0
-7 8 -15 -16 -17 -20 0
-7 8 -15 -16 -18 -20 0
-7 8 -15 -16 -19 -20 0
-7 8 -15 -17 -18 -20 0
-7 8 -15 -17 -19 -20 0
-7 8 -15 -18 -19 -20 0
-7 8 -16 -17 -18 -20 0
-7 8 -16 -17 -19 -20 0
-7 8 -16 -18 -19 -20 0
-7 8 -17 -18 -19 -20 0
8 -15 16 -18 -19 -20 0
-6 8 -15 -16 -17 -19 -20 0
6 7 -8 -15 -16 -17 -18 -20 0
6 7 -8 -15 -16 -17 -19 -20 0
6 7 -8 -15 -16 -18 -19 -20 0
6 7 -8 -15 -17 -18 -19 -20 0
6 7 -8 -16 -17 -18 -19 -20 0
-6 7 -15 -16 -17 -18 -19 -20 0
7 -8 -15 -16 -17 -18 -19 -20 0
6 7 8 9 -21 0
-6 7 8 -15 -21 0
-6 7 8 -16 -21 0
-6 7 8 -17 -21 0
-6 7 8 -18 -21 0
-6 7 8 -19 -21 0
-6 7 8 -20 -21 0
7 8 9 -15 -21 0
7 8 9 -20 -21 0
7 8 16 -17 -21 0
7 8 16 -18 -21 0
6 7 8 16 -20 -21 0
6 -7 8 -15 -16 -21 0
6 -7 8 -15 -17 -21 0
6 -7 8 -15 -18 -21 0
6 -7 8 -15 -19 -21 0
6 -7 8 -15 -20 -21 0
6 -7 8 -16 -17 -21 0
6 -7 8 -16 -18 -21 0
6 -7 8 -16 -19 -21 0
6 -7 8 -16 -20 -21 0
6 -7 8 -17 -18 -21 0
6 -7 8 -17 -19 -21 0
6 -7 8 -17 -20 -21 0
6 -7 8 -18 -19 -21 0
6 -7 8 -18 -20 -21 0
6 -7 8 -19 -20 -21 0
-6 7 8 -17 -19 -21 0
-7 8 -15 -16 -17 -21 0
-7 8 -15 -16 -18 -21 0
-7 8 -15 -16 -19 -21 0
-7 8 -15 -16 -20 -21 0
-7 8 -15 -17 -18 -21 0
-7 8 -15 -17 -19 -21 0
-7 8 -15 -17 -20 -21 0
-7 8 -15 -18 -19 -21 0
-7 8 -15 -18 -20 -21 0
-7 8 -15 -19 -20 -21 0
-7 8 -16 -17 -18 -21 0
-7 8 -16 -17 -19 -21 0
-7 8 -16 -17 -20 -21 0
-7 8 -16 -18 -19 -21 0
-7 8 -16 -18 -20 -21 0
-7 8 -16 -19 -20 -21 0
-7 8 -17 -18 -19 -21 0
-7 8 -17 -18 -20 -21 0
-7 8 -17 -19 -20 -21 0
-7 8 -18 -19 -20 -21 0
8 16 -17 -18 -19 -21 0
-6 8 -15 -16 -17 -19 -21 0
-6 8 -16 -17 -19 -20 -21 0
6 7 -8 -15 -16 -17 -18 -21 0
6 7 -8 -15 -16 -17 -19 -21 0
6 7 -8 -15 -16 -17 -20 -21 0
6 7 -8 -15 -16 -18 -19 -21 0
6 7 -8 -15 -16 -18 -20 -21 0
6 7 -8 -15 -16 -19 -20 -21 0
6 7 -8 -15 -17 -18 -19 -21 0
6 7 -8 -15 -17 -18 -20 -21 0
6 7 -8 -15 -17 -19 -20 -21 0
6 7 -8 -16 -17 -18 -19 -21 0
6 7 -8 -16 -17 -18 -20 -21 0
6 7 -8 -16 -17 -19 -20 -21 0
6 7 -8 -16 -18 -19 -20 -21 0
6 7 -8 -17 -18 -19 -20 -21 0
6 7 -15 16 -17 -18 -19 -21 0
6 7 -15 16 -17 -19 -20 -21 0
6 7 -15 16 -18 -19 -20 -21 0
6 7 16 -17 -18 -19 -20 -21 0
-6 7 -15 -16 -17 -18 -19 -21 0
-6 7 -15 -16 -17 -18 -20 -21 0
-6 7 -15 -16 -17 -19 -20 -21 0
-6 7 -15 -16 -18 -19 -20 -21 0
-6 7 -15 -17 -18 -19 -20 -21 0
-6 7 -16 -17 -18 -19 -20 -21 0
7 -8 -15 -16 -17 -18 -19 -21 0
7 -15 16 -17 -18 -19 -20 -21 0
6 -7 -15 -16 -17 -18 -19 -20 -21 0
6 7 8 9 -22 0
-6 7 8 -15 -22 0
-6 7 8 -16 -22 0
-6 7 8 -17 -22 0
-6 7 8 -18 -22 0
-6 7 8 -19 -22 0
-6 7 8 -20 -22 0
-6 7 8 -21 -22 0
6 -7 8 -15 -16 -22 0
6 -7 8 -15 -17 -22 0
6 -7 8 -15 -18 -22 0
6 -7 8 -15 -19 -22 0
6 -7 8 -15 -20 -22 0
6 -7 8 -15 -21 -22 0
6 -7 8 -16 -17 -22 0
6 -7 8 -16 -18 -22 0
6 -7 8 -16 -19 -22 0
6 -7 8 -16 -20 -22 0
6 -7 8 -16 -21 -22 0
6 -7 8 -17 -18 -22 0
6 -7 8 -17 -19 -22 0
6 -7 8 -17 -20 -22 0
6 -7 8 -17 -21 -22 0
6 -7 8 -18 -19 -22 0
6 -7 8 -18 -20 -22 0
6 -7 8 -18 -21 -22 0
6 -7 8 -19 -20 -22 0
6 -7 8 -19 -21 -22 0
6 -7 8 -20 -21 -22 0
-7 8 -15 -16 -17 -22 0
-7 8 -15 -16 -18 -22 0
-7 8 -15 -16 -19 -22 0
-7 8 -15 -16 -20 -22 0
-7 8 -15 -16 -21 -22 0
-7 8 -15 -17 -18 -22 0
-7 8 -15 -17 -19 -22 0
-7 8 -15 -17 -20 -22 0
-7 8 -15 -17 -21 -22 0
-7 8 -15 -18 -19 -22 0
-7 8 -15 -18 -20 -22 0
-7 8 -15 -18 -21 -22 0
-7 8 -15 -19 -20 -22 0
-7 8 -15 -19 -21 -22 0
-7 8 -15 -20 -21 -22 0
-7 8 -16 -17 -18 -22 0
-7 8 -16 -17 -19 -22 0
-7 8 -16 -17 -20 -22 0
-7 8 -16 -17 -21 -22 0
-7 8 -16 -18 -19 -22 0
-7 8 -16 -18 -20 -22 0
-7 8 -16 -18 -21 -22 0
-7 8 -16 -19 -20 -22 0
-7 8 -16 -19 -21 -22 0
-7 8 -16 -20 -21 -22 0
-7 8 -17 -18 -19 -22 0
-7 8 -17 -18 -20 -22 0
-7 8 -17 -18 -21 -22 0
-7 8 -17 -19 -20 -22 0
-7 8 -17 -19 -21 -22 0
-7 8 -17 -20 -21 -22 0
-7 8 -18 -19 -20 -22 0
-7 8 -18 -19 -21 -22 0
-7 8 -18 -20 -21 -22 0
-7 8 -19 -20 -21 -22 0
6 7 -8 -15 -16 -17 -18 -22 0
6 7 -8 -15 -16 -17 -19 -22 0
6 7 -8 -15 -16 -17 -20 -22 0
6 7 -8 -15 -16 -17 -21 -22 0
6 7 -8 -15 -16 -18 -19 -22 0
6 7 -8 -15 -16 -18 -20 -22 0
6 7 -8 -15 -16 -18 -21 -22 0
6 7 -8 -15 -16 -19 -20 -22 0
6 7 -8 -15 -16 -19 -21 -22 0
6 7 -8 -15 -16 -20 -21 -22 0
6 7 -8 -15 -17 -18 -19 -22 0
6 7 -8 -15 -17 -18 -20 -22 0
6 7 -8 -15 -17 -18 -21 -22 0
6 7 -8 -15 -17 -19 -20 -22 0
6 7 -8 -15 -17 -19 -21 -22 0
6 7 -8 -15 -17 -20 -21 -22 0
6 7 -8 -15 -18 -19 -20 -22 0
6 7 -8 -15 -18 -19 -21 -22 0
6 7 -8 -15 -18 -20 -21 -22 0
6 7 -8 -16 -17 -18 -19 -22 0
6 7 -8 -16 -17 -18 -20 -22 0
6 7 -8 -16 -17 -18 -21 -22 0
6 7 -8 -16 -17 -19 -20 -22 0
6 7 -8 -16 -17 -19 -21 -22 0
6 7 -8 -16 -17 -20 -21 -22 0
6 7 -8 -16 -18 -19 -20 -22 0
6 7 -8 -16 -18 -19 -21 -22 0
6 7 -8 -16 -18 -20 -21 -22 0
6 7 -8 -16 -19 -20 -21 -22 0
6 7 -8 -17 -18 -19 -20 -22 0
6 7 -8 -17 -18 -19 -21 -22 0
6 7 -8 -17 -18 -20 -21 -22 0
6 7 -8 -18 -19 -20 -21 -22 0
6 7 -15 16 -17 -18 -19 -22 0
6 7 -15 16 -17 -20 -21 -22 0
6 7 -15 16 -18 -19 -20 -22 0
6 7 -15 16 -18 -20 -21 -22 0
6 7 -15 16 -19 -20 -21 -22 0
6 7 16 -17 -18 -19 -20 -22 0
6 7 16 -17 -18 -20 -21 -22 0
6 7 16 -17 -19 -20 -21 -22 0
6 8 15 16 -17 -18 -20 -22 0
-6 7 -15 -16 -17 -18 -19 -22 0
-6 7 -15 -16 -17 -18 -20 -22 0
-6 7 -15 -16 -17 -18 -21 -22 0
-6 7 -15 -16 -17 -19 -21 -22 0
-6 7 -15 -16 -18 -19 -20 -22 0
-6 7 -15 -16 -18 -19 -21 -22 0
-6 7 -15 -16 -18 -20 -21 -22 0
-6 7 -15 -18 -19 -20 -21 -22 0
-6 7 -16 -17 -18 -19 -21 -22 0
-6 7 -16 -17 -18 -20 -21 -22 0
-6 7 -16 -17 -19 -20 -21 -22 0
-6 7 -16 -18 -19 -20 -21 -22 0
7 8 9 -16 -18 -19 -21 -22 0
7 -8 -15 -16 -17 -18 -19 -22 0
7 -8 -15 -16 -17 -18 -20 -22 0
7 -8 -15 -16 -17 -18 -21 -22 0
7 -8 -15 -16 -17 -19 -20 -22 0
7 -8 -15 -16 -17 -19 -21 -22 0
7 -8 -15 -16 -17 -20 -21 -22 0
7 -8 -15 -16 -18 -19 -20 -22 0
7 -8 -15 -16 -18 -19 -21 -22 0
7 -8 -15 -16 -18 -20 -21 -22 0
7 -8 -15 -16 -19 -20 -21 -22 0
7 -8 -15 -17 -18 -19 -20 -22 0
7 -8 -15 -17 -18 -19 -21 -22 0
7 -8 -15 -17 -18 -20 -21 -22 0
7 -8 -15 -17 -19 -20 -21 -22 0
7 -8 -16 -17 -18 -19 -20 -22 0
7 -8 -16 -17 -18 -19 -21 -22 0
7 -8 -16 -17 -18 -20 -21 -22 0
7 -8 -16 -17 -19 -20 -21 -22 0
7 -8 -17 -18 -19 -20 -21 -22 0
7 -15 16 -17 -18 -19 -20 -22 0
7 -15 16 -17 -19 -20 -21 -22 0
-7 8 15 16 -17 -18 -19 -22 0
6 -7 8 15 16 -17 -18 -21 -22 0
6 -7 -15 -16 -17 -18 -19 -20 -22 0
6 -7 -15 -16 -17 -18 -19 -21 -22 0
6 -7 -15 -16 -17 -18 -20 -21 -22 0
6 -7 -15 -16 -17 -19 -20 -21 -22 0
6 -7 -15 -16 -18 -19 -20 -21 -22 0
6 -7 -16 -17 -18 -19 -20 -21 -22 0
6 -8 -15 -16 -17 -18 -19 -21 -22 0
6 -15 16 -17 -18 -19 -20 -21 -22 0
-6 -15 -16 -17 -18 -19 -20 -21 -22 0
-7 -15 -16 -17 -18 -19 -20 -21 -22 0";
