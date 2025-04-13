// ************************************************************************************************
// use
// ************************************************************************************************

use super::UniqueSortedVec;

// ************************************************************************************************
// impl
// ************************************************************************************************

impl<V: Ord> UniqueSortedVec<V> {
    /// Checks if the current UniqueSortedVec is a subset of another UniqueSortedVec.
    pub fn is_subset_of(&self, other: &UniqueSortedVec<V>) -> bool {
        if self.len() > other.len() {
            return false;
        }

        let mut c1_literals = self.iter();
        let mut c2_literals = other.iter();
        let mut c1_i = c1_literals.next();
        let mut c2_i = c2_literals.next();
        loop {
            match (c1_i, c2_i) {
                (None, None) => return true,
                (None, Some(_)) => return true,
                (Some(_), None) => return false,
                (Some(a), Some(b)) => match a.cmp(b) {
                    std::cmp::Ordering::Less => {
                        return false;
                    }
                    std::cmp::Ordering::Equal => {
                        c1_i = c1_literals.next();
                        c2_i = c2_literals.next();
                    }
                    std::cmp::Ordering::Greater => {
                        c2_i = c2_literals.next();
                    }
                },
            }
        }
    }
}

// ************************************************************************************************
// test
// ************************************************************************************************

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_subset_of() {
        let mut usv1 = UniqueSortedVec::new();
        let mut usv2 = UniqueSortedVec::new();

        usv1.push(1);
        usv1.push(2);
        usv1.push(3);

        usv2.push(1);
        usv2.push(2);
        usv2.push(3);

        assert!(usv1.is_subset_of(&usv2));
        assert!(usv2.is_subset_of(&usv1));

        usv2.push(4);
        assert!(usv1.is_subset_of(&usv2));
        assert!(!usv2.is_subset_of(&usv1));

        usv1.push(4);
        assert!(usv1.is_subset_of(&usv2));
        assert!(usv2.is_subset_of(&usv1));

        usv1.push(5);
        assert!(!usv1.is_subset_of(&usv2));
        assert!(usv2.is_subset_of(&usv1));

        usv2.push(10);
        assert!(!usv1.is_subset_of(&usv2));
        assert!(!usv2.is_subset_of(&usv1));
    }
}
