// ************************************************************************************************
// use
// ************************************************************************************************

use super::CNF;
use crate::formulas::Clause;
use crate::formulas::Literal;
use crate::formulas::Variable;
use crate::models::UniqueSortedHashMap;
use crate::models::UniqueSortedVec;
use std::cmp::max;
use std::mem;

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

    /// Function that adds a clause to the CNF.
    /// If the clause already exists then it is not added.
    /// This function has a time complexity of O(n) where n is the number of clauses in the CNF.
    ///
    /// # Arguments
    ///
    /// * `self` - a mut reference to self.
    /// * `new_clause` - an immutable reference to a Clause you want to add.
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_formal_verification::formulas::{CNF, Clause, Literal, Variable};
    /// let l1 = Variable::new(1).literal(false);
    /// let l2 = Variable::new(2).literal(false);
    /// let l3 = Variable::new(3).literal(false);
    /// let mut cnf1 = CNF::from_sequence(vec![]);
    /// cnf1.add_clause(Clause::from_sequence(vec![l1, l2, l3]));
    /// cnf1.add_clause(Clause::from_sequence(vec![!l1, l2, l3]));
    /// cnf1.add_clause(Clause::from_sequence(vec![l1, !l2, l3]));
    /// cnf1.add_clause(Clause::from_sequence(vec![l1, l2, !l3]));
    /// assert_eq!(cnf1.to_string(), "p cnf 3 4\n1 2 3 0\n1 -2 3 0\n-1 2 3 0\n1 2 -3 0");
    /// ```
    pub fn add_clause(&mut self, new_clause: Clause) {
        self.max_variable_number = max(self.max_variable_number, new_clause.max_variable());
        self.clauses.insert(new_clause);
    }

    pub fn contains(&self, clause: &Clause) -> bool {
        self.clauses.contains(clause)
    }

    pub fn get_max_variable(&self) -> Variable {
        self.max_variable_number
    }

    /// Function that returns the number of clauses that are currently in the CNF.
    ///
    /// # Arguments
    ///
    /// * `self` - an immutable reference to self.
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_formal_verification::formulas::{CNF, Clause, Literal, Variable};
    /// let l1 = Variable::new(1).literal(false);
    /// let l2 = Variable::new(2).literal(false);
    /// let l3 = Variable::new(3).literal(false);
    /// let mut cnf1 = CNF::from_sequence(vec![]);
    /// assert_eq!(cnf1.len(), 0);
    /// cnf1.add_clause(Clause::from_sequence(vec![l1, l2, l3]));
    /// assert_eq!(cnf1.len(), 1);
    /// cnf1.add_clause(Clause::from_sequence(vec![!l1, l2, l3]));
    /// assert_eq!(cnf1.len(), 2);
    /// cnf1.add_clause(Clause::from_sequence(vec![l1, !l2, l3]));
    /// assert_eq!(cnf1.len(), 3);
    /// cnf1.add_clause(Clause::from_sequence(vec![l1, l2, !l3]));
    /// assert_eq!(cnf1.len(), 4);
    /// ```
    pub fn len(&self) -> usize {
        self.clauses.len()
    }

    /// Function that returns if the cnf is empty or not.
    ///
    /// # Arguments
    ///
    /// * `self` - an immutable reference to self.
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_formal_verification::formulas::{CNF, Clause, Literal, Variable};
    /// let l1 = Variable::new(1).literal(false);
    /// let l2 = Variable::new(2).literal(false);
    /// let l3 = Variable::new(3).literal(false);
    /// let mut cnf1 = CNF::from_sequence(vec![]);
    /// assert!(cnf1.is_empty());
    /// cnf1.add_clause(Clause::from_sequence(vec![l1, l2, l3]));
    /// assert!(!cnf1.is_empty());
    /// cnf1.add_clause(Clause::from_sequence(vec![!l1, l2, l3]));
    /// assert!(!cnf1.is_empty());
    /// cnf1.add_clause(Clause::from_sequence(vec![l1, !l2, l3]));
    /// assert!(!cnf1.is_empty());
    /// cnf1.add_clause(Clause::from_sequence(vec![l1, l2, !l3]));
    /// assert!(!cnf1.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.clauses.is_empty()
    }

    /// Function that returns an iterator over the clauses of the cnf.
    ///
    /// # Arguments
    ///
    /// * `self` - an immutable reference to self.
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_formal_verification::formulas::{CNF, Clause, Literal, Variable};
    /// let l1 = Variable::new(1).literal(false);
    /// let l2 = Variable::new(2).literal(false);
    /// let l3 = Variable::new(3).literal(false);
    /// let mut cnf1 = CNF::from_sequence(vec![
    ///     Clause::from_sequence(vec![l1, l2, l3]),
    ///     Clause::from_sequence(vec![!l1, l2, l3]),
    ///     Clause::from_sequence(vec![l1, !l2, l3]),
    ///     Clause::from_sequence(vec![l1, l2, !l3]),
    /// ]);
    /// for c in cnf1.iter() {
    ///     assert_eq!(c.len(), 3);
    /// }
    /// ```
    pub fn iter(&self) -> impl Iterator<Item = &Clause> {
        self.clauses.iter()
    }

    /// Function that appends another CNF to self.
    ///
    /// # Arguments
    ///
    /// * `self` - an immutable reference to self.
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_formal_verification::formulas::{CNF, Clause, Literal, Variable};
    /// let l1 = Variable::new(1).literal(false);
    /// let l2 = Variable::new(2).literal(false);
    /// let l3 = Variable::new(3).literal(false);
    /// let mut cnf1 = CNF::from_sequence(vec![
    ///     Clause::from_sequence(vec![l1, l2, l3]),
    ///     Clause::from_sequence(vec![!l1, l2, l3]),
    /// ]);
    /// let mut cnf2 = CNF::from_sequence(vec![
    ///     Clause::from_sequence(vec![l1, !l2, l3]),
    ///     Clause::from_sequence(vec![l1, l2, !l3]),
    ///     Clause::from_sequence(vec![l1, l2, l3]),
    /// ]);
    ///
    /// assert_eq!(cnf1.len(), 2);
    /// cnf1.append(cnf2);
    /// assert_eq!(cnf1.len(), 4);
    /// ```
    pub fn append(&mut self, cnf: CNF) {
        self.max_variable_number = max(self.max_variable_number, cnf.max_variable_number);
        let old_clauses = mem::take(&mut self.clauses);
        self.clauses = UniqueSortedVec::merge_consuming(old_clauses, cnf.clauses)
    }

    pub fn bump_all_literals(&mut self, delta: i32) {
        self.max_variable_number = if self.max_variable_number > Variable::new(0) {
            let mut l = Literal::new(self.max_variable_number);
            l.bump(delta);
            l.variable()
        } else {
            Variable::new(0)
        };
        self.clauses
            .perform_operation_on_each_value(|c| c.bump_all_literals(delta))
    }

    pub fn unpack(self) -> UniqueSortedVec<Clause> {
        self.clauses
    }

    pub fn peek(&self) -> &UniqueSortedVec<Clause> {
        &self.clauses
    }

    /// Function that gets all the literals used in the CNF.
    ///
    /// # Arguments
    ///
    /// * `self` - an immutable reference to self.
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_formal_verification::formulas::{CNF, Clause, Literal, Variable};
    /// let l1 = Variable::new(1).literal(false);
    /// let l2 = Variable::new(2).literal(false);
    /// let l3 = Variable::new(3).literal(false);
    /// let l4 = Variable::new(4).literal(false);
    /// let cnf1 = CNF::from_sequence(vec![
    ///     Clause::from_sequence(vec![l1, l2, l3]),
    ///     Clause::from_sequence(vec![!l1, l2, l4]),
    ///     Clause::from_sequence(vec![!l1, l2, l3, l4]),
    /// ]);
    ///
    /// assert_eq!(cnf1.get_literals().unpack(), vec![l1, !l1, l2, l3, l4]);
    ///
    /// ```
    pub fn get_literals(&self) -> UniqueSortedVec<Literal> {
        let mut literals = UniqueSortedVec::new();
        for clause in self.iter() {
            for literal in clause.iter() {
                literals.insert(*literal);
            }
        }
        literals
    }

    /// Function that gets all the variables used in the CNF.
    ///
    /// # Arguments
    ///
    /// * `self` - an immutable reference to self.
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_formal_verification::formulas::{CNF, Clause, Literal, Variable};
    /// let l1 = Variable::new(1).literal(false);
    /// let l2 = Variable::new(2).literal(false);
    /// let l3 = Variable::new(3).literal(false);
    /// let l4 = Variable::new(4).literal(false);
    /// let cnf1 = CNF::from_sequence(vec![
    ///     Clause::from_sequence(vec![l1, l2, l3]),
    ///     Clause::from_sequence(vec![!l1, l2, l4]),
    ///     Clause::from_sequence(vec![!l1, l2, l3, l4]),
    /// ]);
    ///
    /// assert_eq!(cnf1.get_variables().unpack(), vec![l1.variable(), l2.variable(), l3.variable(), l4.variable()]);
    ///
    /// ```
    pub fn get_variables(&self) -> UniqueSortedVec<Variable> {
        let mut vars: Vec<Variable> = self
            .get_literals()
            .unpack()
            .into_iter()
            .map(|l| l.variable())
            .collect();
        vars.dedup();
        UniqueSortedVec::from_ordered_set(vars)
    }

    /// Gets the number of occurences of each literal in the CNF.
    /// ```
    /// use rust_formal_verification::formulas::{CNF, Clause, Literal, Variable};
    /// let l1 = Variable::new(1).literal(false);
    /// let l2 = Variable::new(2).literal(false);
    /// let l3 = Variable::new(3).literal(false);
    /// let l4 = Variable::new(4).literal(false);
    /// let cnf1 = CNF::from_sequence(vec![
    ///     Clause::from_sequence(vec![l1, l2, l3]),
    ///     Clause::from_sequence(vec![!l1, l2, l4]),
    ///     Clause::from_sequence(vec![!l1, l2, !l3, l4]),
    /// ]);
    /// let occ = cnf1.count_literal_occurences();
    /// assert_eq!(*occ.get(&l1).unwrap(), 1);
    /// assert_eq!(*occ.get(&l2).unwrap(), 3);
    /// assert_eq!(*occ.get(&l3).unwrap(), 1);
    /// assert_eq!(*occ.get(&l4).unwrap(), 2);
    /// assert_eq!(*occ.get(&!l1).unwrap(), 2);
    /// assert_eq!(*occ.get(&!l3).unwrap(), 1);
    /// assert_eq!(occ.get(&!l2), None);
    /// assert_eq!(occ.get(&!l4), None);
    /// ```
    pub fn count_literal_occurences(&self) -> UniqueSortedHashMap<Literal, usize> {
        let max_lit = std::cmp::max(
            self.max_variable_number.literal(true),
            self.max_variable_number.literal(false),
        );
        let mut literal_to_priority: UniqueSortedHashMap<Literal, usize> =
            UniqueSortedHashMap::new(max_lit);
        for clause in self.iter() {
            for literal in clause.iter() {
                match literal_to_priority.get_mut(literal) {
                    Some(l) => *l += 1,
                    None => {
                        literal_to_priority.insert(*literal, 1);
                    }
                }
            }
        }
        literal_to_priority
    }

    /// Gets the number of occurences of each literal in the CNF. Returns None if the CNF is empty.
    /// ```
    /// use rust_formal_verification::formulas::{CNF, Clause, Literal, Variable};
    /// use rust_formal_verification::models::{UniqueSortedVec};
    /// let l1 = Variable::new(1).literal(false);
    /// let l2 = Variable::new(2).literal(false);
    /// let l3 = Variable::new(3).literal(false);
    /// let l4 = Variable::new(4).literal(false);
    /// let cnf1 = CNF::from_sequence(vec![
    ///     Clause::from_sequence(vec![l1, l2, l3]),
    ///     Clause::from_sequence(vec![!l1, l2, l4]),
    ///     Clause::from_sequence(vec![!l1, l2, !l3, l4]),
    /// ]);
    /// let occ = cnf1.get_literal_occurences();
    /// assert_eq!(*occ.get(&l1).unwrap(), UniqueSortedVec::from_ordered_set(vec![0]));
    /// assert_eq!(*occ.get(&l2).unwrap(), UniqueSortedVec::from_ordered_set(vec![0, 1, 2]));
    /// assert_eq!(*occ.get(&l3).unwrap(), UniqueSortedVec::from_ordered_set(vec![0]));
    /// assert_eq!(*occ.get(&l4).unwrap(), UniqueSortedVec::from_ordered_set(vec![1, 2]));
    /// assert_eq!(*occ.get(&!l1).unwrap(), UniqueSortedVec::from_ordered_set(vec![1, 2]));
    /// assert_eq!(*occ.get(&!l3).unwrap(), UniqueSortedVec::from_ordered_set(vec![2]));
    /// assert_eq!(occ.get(&!l2), None);
    /// assert_eq!(occ.get(&!l4), None);
    /// ```
    pub fn get_literal_occurences(&self) -> UniqueSortedHashMap<Literal, UniqueSortedVec<usize>> {
        Self::static_get_literal_occurences(self.clauses.peek())
    }

    /// Gets the number of occurences of each literal in the CNF. Returns None if the CNF is empty.
    pub fn static_get_literal_occurences(
        clauses: &[Clause],
    ) -> UniqueSortedHashMap<Literal, UniqueSortedVec<usize>> {
        let max_var = match clauses.iter().map(|c| c.max_variable()).max() {
            Some(x) => x,
            None => return UniqueSortedHashMap::new(Literal::new(Variable::new(1))),
        };
        let max_lit = std::cmp::max(max_var.literal(false), max_var.literal(true));
        let mut literal_to_occurences: UniqueSortedHashMap<Literal, UniqueSortedVec<usize>> =
            UniqueSortedHashMap::new(max_lit);
        for (i, clause) in clauses.iter().enumerate() {
            for literal in clause.iter() {
                match literal_to_occurences.get_mut(literal) {
                    Some(usv) => usv.push(i),
                    None => {
                        literal_to_occurences
                            .insert(*literal, UniqueSortedVec::from_ordered_set(vec![i]));
                    }
                }
            }
        }
        literal_to_occurences
    }
}
