// ************************************************************************************************
// use
// ************************************************************************************************

use super::Utils;
use crate::models::UniqueSortedVec;

// ************************************************************************************************
// impl
// ************************************************************************************************

impl Utils {
    // ********************************************************************************************
    // API
    // ********************************************************************************************

    /// Remove mutiple
    pub fn remove_indexes<T>(v: &mut Vec<T>, indexes: &UniqueSortedVec<usize>) {
        if indexes.is_empty() {
            return;
        }
        let mut indexes = indexes.iter();
        let mut i = indexes.next();
        let mut j = 0;
        let mut k = 0;
        while j < v.len() {
            if i.is_some() && i.unwrap() == &j {
                i = indexes.next();
            } else {
                v.swap(k, j);
                k += 1;
            }
            j += 1;
        }
        v.truncate(k);
    }
}

#[cfg(test)]
mod tests {
    use crate::models::{UniqueSortedVec, Utils};

    #[test]
    fn test_remove_mult() {
        let usv: Vec<i32> = vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10];

        let mut usv_c = usv.clone();
        let indexes: UniqueSortedVec<usize> =
            UniqueSortedVec::from_ordered_set(vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
        Utils::remove_indexes(&mut usv_c, &indexes);
        assert!(usv_c.is_empty());

        let mut usv_c = usv.clone();
        let indexes: UniqueSortedVec<usize> = UniqueSortedVec::from_ordered_set(vec![0, 1, 2]);
        Utils::remove_indexes(&mut usv_c, &indexes);
        assert_eq!(usv_c.len(), 8);
        assert_eq!(usv_c, vec![3, 4, 5, 6, 7, 8, 9, 10]);
    }
}
