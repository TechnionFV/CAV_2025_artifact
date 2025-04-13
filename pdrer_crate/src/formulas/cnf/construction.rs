// ************************************************************************************************
// use
// ************************************************************************************************

use rand::distributions::Bernoulli;
use rand::distributions::Distribution;
use rand::prelude::SliceRandom;
use rand::Rng;

use super::CNF;
use crate::formulas::Clause;
use crate::formulas::Literal;
use crate::formulas::Variable;
use crate::models::UniqueSortedVec;

// ************************************************************************************************
// re-exports of structs in these modules to simplify paths for other imports
// ************************************************************************************************

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

    pub fn new() -> Self {
        Self {
            max_variable_number: Variable::new(0),
            clauses: UniqueSortedVec::new(),
        }
    }

    pub fn from_sequence(sequence: Vec<Clause>) -> Self {
        let clauses = UniqueSortedVec::from_sequence(sequence);
        let max_variable_number = clauses
            .iter()
            .map(|x| x.max_variable())
            .max()
            .unwrap_or(Variable::new(0));
        Self {
            max_variable_number,
            clauses,
        }
    }

    pub fn from_ordered_set(set: UniqueSortedVec<Clause>) -> Self {
        // let clauses = UniqueSortedVec::from_sorted(clauses);
        let max_variable_number = set
            .iter()
            .map(|x| x.max_variable())
            .max()
            .unwrap_or(Variable::new(0));
        Self {
            max_variable_number,
            clauses: set,
        }
    }

    /// create cnf from a DIMACS string.
    /// The DIMACS format is a simple text-based format for representing CNF formulas.
    ///
    pub fn from_dimacs(dimacs: &str) -> Result<Self, String> {
        let mut clauses: Vec<Clause> = vec![];
        for line in dimacs.lines() {
            // skip comment
            if line.starts_with('c') {
                continue;
            }

            // skip header line
            if line.starts_with('p') {
                let number_of_clauses = line.split_whitespace().nth(3).unwrap().parse().unwrap();
                clauses = Vec::with_capacity(number_of_clauses);
                continue;
            }

            // clause line, looks something like this: "1 2 3 0"
            let mut clause = vec![];
            for literal in line.split_whitespace() {
                let literal = literal.parse::<i32>().unwrap();
                if literal == 0 {
                    // reached end of clause
                    break;
                }
                let internal_literal = Literal::new(Variable::new(literal.unsigned_abs()));
                if literal < 0 {
                    clause.push(!internal_literal);
                } else {
                    clause.push(internal_literal);
                }
            }

            // add clause to cnf
            // assert!(clauses.is_some(), "Reached clause before header line.");
            let new_clause = Clause::from_sequence(clause);
            clauses.push(new_clause);
        }
        Ok(CNF::from_sequence(clauses))
    }

    /// Generate a random CNF formula, with a given number of variables and clauses.
    /// The probability of negating a variable in a clause is given by `probability_of_negating_variable`.
    /// The length of each clause is determined by the closure `length_per_clause`.
    /// The closure `length_per_clause` is called with the index of the clause and should return the length of the clause.
    ///
    /// # Arguments
    ///
    /// * `rng` - The random number generator to use.
    /// * `variables` - The set of variables to use.
    /// * `probability_of_negating_variable` - The probability of negating a variable in a clause.
    /// * `max_number_of_clauses` - The number of clauses to generate.
    /// * `length_per_clause` - A closure that determines the length of each clause.
    ///
    /// # Example
    ///
    /// ```
    /// use rand::SeedableRng;
    /// use rand::rngs::StdRng;
    /// use rust_formal_verification::formulas::{Variable, CNF};
    /// use rust_formal_verification::models::UniqueSortedVec;
    ///
    /// let mut rng = StdRng::seed_from_u64(0);
    /// let variables = UniqueSortedVec::from_sequence((1..=10).map(Variable::new).collect());
    /// let cnf = CNF::custom_random(&mut rng, &variables, 0.5, 10, |_, _| 3);
    ///
    /// assert!(cnf.len() <= 10);
    ///
    /// ```
    pub fn custom_random<R: Rng, F>(
        rng: &mut R,
        variables: &UniqueSortedVec<Variable>,
        probability_of_negating_variable: f64,
        max_number_of_clauses: usize,
        length_per_clause: F,
    ) -> Self
    where
        F: Fn(&mut R, usize) -> usize,
    {
        let mut clauses = Vec::with_capacity(max_number_of_clauses);
        let d = Bernoulli::new(probability_of_negating_variable).unwrap();
        // let start_time = std::time::Instant::now();
        for i in 0..max_number_of_clauses {
            let length = length_per_clause(rng, i);
            let mut clause = Vec::with_capacity(length);
            for x in variables.peek().choose_multiple(rng, length) {
                let is_negated = d.sample(rng);
                let x = x.literal(is_negated);
                clause.push(x);
            }
            clause.sort_unstable();
            let clause = Clause::from_sequence(clause);
            clauses.push(clause);
        }
        // let elapsed = start_time.elapsed();
        // println!("Elapsed: {}", elapsed.as_secs_f32());
        Self::from_sequence(clauses)
    }
}

// ************************************************************************************************
// Default impl
// ************************************************************************************************

impl Default for CNF {
    fn default() -> Self {
        Self::new()
    }
}
