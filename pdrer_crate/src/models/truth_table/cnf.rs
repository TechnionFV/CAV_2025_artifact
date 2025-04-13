// ************************************************************************************************
// use
// ************************************************************************************************

use fxhash::FxHashMap;
use rand::{rngs::StdRng, Rng, SeedableRng};

use super::{TruthTable, TruthTableEntry};
use crate::{
    formulas::{Clause, Literal, Variable, CNF},
    models::{Signal, Wire},
};

pub type CNFCache = FxHashMap<(usize, TruthTableEntry), Vec<Vec<i32>>>;

// ************************************************************************************************
// impl
// ************************************************************************************************

const fn calculate_if_then_else_tts() -> [(TruthTableEntry, u32, i32, i32); 24] {
    let mut tts = [(0, 0, 0, 0); 24];

    const fn index_to_var(i: i32) -> TruthTableEntry {
        match i {
            3 => 0b11110000,
            -3 => 0b00001111,
            2 => 0b11001100,
            -2 => 0b00110011,
            1 => 0b10101010,
            -1 => 0b01010101,
            _ => unreachable!(),
        }
    }

    const fn get_if_then_else(i: u32, t: i32, e: i32) -> (TruthTableEntry, u32, i32, i32) {
        (
            ((index_to_var(i as i32) & index_to_var(t))
                | (index_to_var(-(i as i32)) & index_to_var(e))),
            i,
            t,
            e,
        )
    }

    let mut write_index = 0;
    let mut i: u32 = 1;
    while i <= 3 {
        let mut j: i32 = 1;
        while j <= 3 {
            if j as u32 == i {
                j += 1;
                continue;
            }

            let mut k: i32 = 1;
            while k <= 3 {
                if k as u32 == i || k == j {
                    k += 1;
                    continue;
                }

                tts[write_index] = get_if_then_else(i, j, k);
                write_index += 1;
                tts[write_index] = get_if_then_else(i, -j, k);
                write_index += 1;
                tts[write_index] = get_if_then_else(i, j, -k);
                write_index += 1;
                tts[write_index] = get_if_then_else(i, -j, -k);
                write_index += 1;

                k += 1;
            }
            j += 1;
        }

        i += 1;
    }

    tts
}

const IF_THEN_ELSE_TRUTH_TABLES: [(TruthTableEntry, u32, i32, i32); 24] =
    calculate_if_then_else_tts();

impl TruthTable {
    // ********************************************************************************************
    // helper functions
    // ********************************************************************************************

    fn get_value_as_literal<F>(&self, index: usize, w2l: F, output_signal: Signal) -> Literal
    where
        F: Fn(&Wire) -> Literal,
    {
        let literal = w2l(&output_signal.wire(false));
        if self.get_value(index) {
            literal
        } else {
            !literal
        }
    }

    /// creates a cube that represents the inputs of the current line in the truth table.
    /// For example if the truth table has 2 inputs 'a' and 'b', then
    /// 1. for index 0 the function returns (!a ^ !b), if 'a' is constant (assumed zero) then it will return (!b)
    /// 2. for index 1 the function returns (!a ^ b), if 'a' is constant (assumed zero) then it will return (b)
    /// 3. for index 2 the function returns (a ^ !b), if 'a' is constant (assumed zero) then it will return None
    /// 4. for index 3 the function returns (a ^ b), if 'a' is constant (assumed zero) then it will return None
    fn get_cube_from_index<F>(&self, index: usize, w2l: F) -> Option<Vec<Literal>>
    where
        F: Fn(&Wire) -> Literal,
    {
        let mut literals = Vec::new();
        let mut current_index = index;
        for input in self.input_names.iter() {
            let bit = current_index & 1;

            // first check if input signals is constant
            if input.is_constant() {
                if bit == 1 {
                    return None;
                } else {
                    current_index >>= 1;
                    continue;
                }
            }

            let literal = w2l(&input.wire(false));
            if bit == 1 {
                literals.push(literal);
            } else {
                literals.push(!literal);
            }
            current_index >>= 1;
        }
        // for signal in self.input_names.iter() {
        //     let mut literal = Literal::new(fin_state.get_state_variable(i));
        //     if self.get_value(i) {
        //         literal.negate();
        //     }
        //     literals.push(literal);
        // }
        Some(literals)
    }

    fn check_if_then_else<F>(&self, w2l: F, output_signal: Signal) -> Option<Vec<Clause>>
    where
        F: Fn(&Wire) -> Literal,
    {
        if self.input_names.len() != 3 {
            return None;
        }

        let (_, i, t, e) = IF_THEN_ELSE_TRUTH_TABLES
            .iter()
            .find(|x| x.0 == self.truth_table)?;

        let lhs = w2l(&output_signal.wire(false));

        let li = w2l(&self.input_names.peek()[(i - 1) as usize].wire(false));
        // li = li.negate_if_true(*i < 0);

        let mut lt = w2l(&self.input_names.peek()[(t.abs() - 1) as usize].wire(false));
        lt = lt.negate_if_true(*t < 0);

        let mut le = w2l(&self.input_names.peek()[(e.abs() - 1) as usize].wire(false));
        le = le.negate_if_true(*e < 0);

        Some(vec![
            Clause::from_sequence(vec![lt, !li, !lhs]),
            Clause::from_sequence(vec![!lt, !li, lhs]),
            Clause::from_sequence(vec![le, li, !lhs]),
            Clause::from_sequence(vec![!le, li, lhs]),
        ])
    }

    // ********************************************************************************************
    // API
    // ********************************************************************************************

    pub fn is_ite(&self) -> bool {
        if self.input_names.len() != 3 {
            return false;
        }

        IF_THEN_ELSE_TRUTH_TABLES
            .iter()
            .any(|x| x.0 == self.truth_table)
    }

    /// Get a cnf description of the truth table.
    pub fn calculate_cnf<F>(&self, w2l: F, output_signal: Signal) -> Vec<Clause>
    where
        F: Fn(&Wire) -> Literal,
    {
        const DEFAULT_CADICAL_SIMPLIFICATION_ROUNDS: i32 = 3;
        const USE_CADICAL_SIMPLIFICATION: bool = false;
        const SHOULD_SIMPLIFY: bool = true;

        if let Some(clauses) = self.check_if_then_else(&w2l, output_signal) {
            return clauses;
        }

        let lhs = w2l(&output_signal.wire(false));

        // println!("Calculating CNF for truth table: {:#b}", self.truth_table);

        debug_assert!(!(lhs.is_negated()));
        let mut clauses: Vec<Clause> = Vec::new();
        for i in 0..self.calculate_number_of_rows() {
            let optional_cube = self.get_cube_from_index(i, &w2l);
            match optional_cube {
                Some(mut cube) => {
                    for l in cube.iter_mut() {
                        *l = !(*l);
                    }
                    let mut clause = cube;
                    let literal_to_add_to_clause =
                        self.get_value_as_literal(i, &w2l, output_signal);
                    match clause.binary_search(&literal_to_add_to_clause) {
                        Ok(_) => unreachable!(),
                        Err(position) => clause.insert(position, literal_to_add_to_clause),
                    }
                    let new_clause = clause;
                    clauses.push(Clause::from_ordered_set(new_clause));
                }
                None => continue,
            }
        }

        // println!("Clauses before: {:?}", clauses);
        if SHOULD_SIMPLIFY {
            if USE_CADICAL_SIMPLIFICATION {
                let mut frozen: Vec<Variable> = self
                    .input_names
                    .iter()
                    .map(|x| w2l(&x.wire(false)).variable())
                    .collect();
                frozen.push(lhs.variable());

                let mut rng = StdRng::seed_from_u64(123456789101112);
                let mut possibilities = vec![];
                for _ in 0..10 {
                    let seed: u64 = rng.gen();
                    let (x, _) = CNF::simplify_using_cadical(
                        seed,
                        &clauses,
                        &frozen,
                        DEFAULT_CADICAL_SIMPLIFICATION_ROUNDS,
                    );
                    possibilities.push(x);
                }
                let best = possibilities.into_iter().max_by_key(|c| c.len()).unwrap();
                clauses = best;
            } else {
                CNF::static_simple_bounded_variable_elimination(&mut clauses);
            }
        }

        clauses

        // clauses.into_iter().map(Clause::new_when_sorted).collect()
    }

    pub fn calculate_area(&self) -> usize {
        let free_signal_num = self
            .input_names
            .iter()
            .copied()
            .max()
            .unwrap_or(Signal::GROUND)
            .number()
            + 1;
        let cnf = self.calculate_cnf(
            |w| Variable::new(w.signal().number()).literal(w.is_negated()),
            Signal::new(free_signal_num),
        );
        cnf.len()
    }

    pub fn calculate_cnf_with_cache<F>(
        &self,
        w2l: F,
        output_signal: Signal,
        cache: &mut CNFCache,
    ) -> Vec<Clause>
    where
        F: Fn(&Wire) -> Literal,
    {
        let k = (self.input_names.len(), self.truth_table);

        let mut converted_literals: Vec<Literal> = self
            .input_names
            .iter()
            .map(|x| w2l(&x.wire(false)))
            .collect();
        converted_literals.push(w2l(&output_signal.wire(false)));

        let encoded_cnf = cache.entry(k).or_insert_with(|| {
            let cnf = self.calculate_cnf(&w2l, output_signal);
            let literal_to_index = |l: &Literal| -> i32 {
                for (i, x) in converted_literals.iter().enumerate() {
                    if x == l {
                        return i as i32 + 1;
                    } else if *x == !*l {
                        return -(i as i32 + 1);
                    }
                }
                unreachable!()
            };
            let r: Vec<Vec<i32>> = cnf
                .iter()
                .map(|c| c.iter().map(literal_to_index).collect())
                .collect();

            // println!("Calculating CNF for truth table: {:#b}", self.truth_table);
            // println!("CNF:");
            // for c in r.iter() {
            //     println!("{:?}", c);
            // }
            r
        });

        let r = encoded_cnf
            .iter()
            .map(|c| {
                Clause::from_ordered_set(
                    c.iter()
                        .map(|i| {
                            if *i > 0 {
                                converted_literals[*i as usize - 1]
                            } else {
                                !converted_literals[(-i) as usize - 1]
                            }
                        })
                        .collect(),
                )
            })
            .collect();

        debug_assert_eq!(r, self.calculate_cnf(&w2l, output_signal));

        r
    }
}
