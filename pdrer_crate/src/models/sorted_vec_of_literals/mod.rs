// ************************************************************************************************
// use
// ************************************************************************************************

use super::unique_sorted_hash_map::UniqueSortedHash;
use super::UniqueSortedVec;
use crate::formulas::Literal;
use crate::formulas::Variable;

// ************************************************************************************************
// struct
// ************************************************************************************************

#[derive(Eq, PartialEq, Clone, Hash, Debug, Default)]
pub struct SortedVecOfLiterals {
    usv: UniqueSortedVec<Literal>,
}

// ************************************************************************************************
// constants
// ************************************************************************************************

const MAKE_VARIABLE_NUMBER_UNIQUE: bool = true;

// ************************************************************************************************
// impl
// ************************************************************************************************

impl SortedVecOfLiterals {
    // ********************************************************************************************
    // helper function
    // ********************************************************************************************

    pub fn are_variables_sorted_and_unique(literals: &[Literal]) -> bool {
        if MAKE_VARIABLE_NUMBER_UNIQUE {
            literals
                .windows(2)
                .all(|w| w[0].variable() < w[1].variable())
        } else {
            true
        }
    }

    // ********************************************************************************************
    // API
    // ********************************************************************************************

    pub fn new() -> Self {
        Self {
            usv: UniqueSortedVec::new(),
        }
    }

    pub fn from_ordered_set(literals: Vec<Literal>) -> Self {
        assert!(
            Self::are_variables_sorted_and_unique(&literals),
            "Literal variables are not sorted and unique"
        );
        Self {
            usv: UniqueSortedVec::from_ordered_set(literals),
        }
    }

    pub fn from_sequence(mut literals: Vec<Literal>) -> Self {
        literals.sort_unstable();
        literals.dedup();
        assert!(Self::are_variables_sorted_and_unique(&literals));
        Self {
            usv: UniqueSortedVec::from_sequence(literals),
        }
    }

    pub fn len(&self) -> usize {
        self.usv.len()
    }

    pub fn is_empty(&self) -> bool {
        self.usv.is_empty()
    }

    pub fn iter(&self) -> impl DoubleEndedIterator<Item = &Literal> + ExactSizeIterator {
        self.usv.iter()
    }

    pub fn max_variable(&self) -> Variable {
        match self.usv.max() {
            Some(l) => l.variable(),
            None => Variable::new(0),
        }
    }

    pub fn max_literal(&self) -> Option<Literal> {
        self.usv.max().copied()
    }

    pub fn min_variable(&self) -> Variable {
        match self.usv.min() {
            Some(l) => l.variable(),
            None => Variable::new(0),
        }
    }

    pub fn min_literal(&self) -> Option<Literal> {
        self.usv.min().copied()
    }

    pub fn negate_literals(&mut self) {
        self.usv.perform_operation_on_each_value(|lit| {
            *lit = !(*lit);
        });
        debug_assert!(Self::are_variables_sorted_and_unique(self.usv.peek()));
    }

    pub fn unpack(self) -> UniqueSortedVec<Literal> {
        self.usv
    }

    pub fn peek(&self) -> &UniqueSortedVec<Literal> {
        &self.usv
    }

    /// This function is problematic as it can break the invariant that variables are sorted and unique.
    pub fn peek_mut(&mut self) -> &mut UniqueSortedVec<Literal> {
        &mut self.usv
    }

    pub fn bump_all_literals(&mut self, delta: i32) {
        self.usv.perform_operation_on_each_value(|lit| {
            (*lit).bump(delta);
        });
        debug_assert!(Self::are_variables_sorted_and_unique(self.usv.peek()));
    }

    pub fn contains(&self, lit: &Literal) -> bool {
        self.usv.contains(lit)
    }

    pub fn contains_variable(&self, var: &Variable) -> bool {
        let l = var.literal(false);
        self.usv.contains(&l) || self.usv.contains(&!l)
    }

    pub fn insert(&mut self, lit: Literal) -> bool {
        if MAKE_VARIABLE_NUMBER_UNIQUE {
            assert!(!self.usv.contains(&(!lit)));
        }
        let r = self.usv.insert(lit);
        debug_assert!(Self::are_variables_sorted_and_unique(self.usv.peek()));
        r
    }

    pub fn remove(&mut self, lit: &Literal) -> bool {
        self.usv.remove(lit)
    }

    pub fn remove_index(&mut self, index: usize) -> Literal {
        self.usv.remove_index(index)
    }

    pub fn retain<F>(&mut self, f: F)
    where
        F: FnMut(&Literal) -> bool,
    {
        self.usv.retain(f)
    }

    pub fn position(&self, lit: &Literal) -> Option<usize> {
        self.usv.position(lit)
    }

    pub fn shrink_to_fit(&mut self) {
        self.usv.shrink_to_fit();
    }
}

// ************************************************************************************************
// Ordering
// ************************************************************************************************

/// This is meant to be a more efficient way to compare two vectors of literals.
impl PartialOrd for SortedVecOfLiterals {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for SortedVecOfLiterals {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        let self_literal_number = self.max_literal().map(|l| l.hash()).unwrap_or(0);
        let other_literal_number = other.max_literal().map(|l| l.hash()).unwrap_or(0);
        let a = self_literal_number.cmp(&other_literal_number);
        let b = a.then(self.len().cmp(&other.len()));
        b.then(self.usv.peek().cmp(other.usv.peek()))
    }
}
