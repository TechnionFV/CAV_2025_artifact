// ************************************************************************************************
// use
// ************************************************************************************************

use super::{VariableWeight, VariableWeights, INITIAL_WEIGHT};
use crate::{formulas::Variable, models::unique_sorted_hash_map::UniqueSortedHash};

// ************************************************************************************************
// printing
// ************************************************************************************************

impl VariableWeights {
    // ********************************************************************************************
    // helper functions
    // ********************************************************************************************

    fn get_table_contents(&self, ignore_zeros: bool) -> Vec<(Variable, VariableWeight)> {
        let mut table_content = Vec::with_capacity(self.weight_per_variable.len());
        for (index, weight) in self.weight_per_variable.iter().enumerate() {
            if index == 0 || index == 1 || (ignore_zeros && *weight == INITIAL_WEIGHT) {
                continue;
            }
            let literal = Variable::un_hash(index);
            table_content.push((literal, *weight));
        }
        table_content
    }

    fn print_table_contents(&self, table_content: &[(Variable, VariableWeight)]) {
        // printing table
        for (i, (literal, variable_weight)) in table_content.iter().enumerate() {
            println!(
                "{}:\tvariable = {}\tweigth = {}",
                i, literal, variable_weight
            );
        }
    }

    // ********************************************************************************************
    // API
    // ********************************************************************************************

    pub fn print_sorted_by_literal_weights(&self, ignore_zeros: bool) {
        let mut table_content = self.get_table_contents(ignore_zeros);
        table_content.sort_unstable_by(|x, y| x.1.total_cmp(&y.1).then(x.0.cmp(&y.0)));
        self.print_table_contents(&table_content);
    }

    // pub fn print_sorted_by_variable_weight(&self, ignore_zeros: bool) {
    //     let mut table_content = self.get_table_contents(ignore_zeros);
    //     table_content.sort_unstable_by_key(|x| x.2);
    //     self.print_table_contents(&table_content);
    // }
}
