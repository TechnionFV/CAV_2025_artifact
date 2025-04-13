// ************************************************************************************************
// use
// ************************************************************************************************

use crate::{formulas::Literal, models::unique_sorted_hash_map::UniqueSortedHash};

use super::LiteralWeights;

// ************************************************************************************************
// printing
// ************************************************************************************************

impl LiteralWeights {
    // ********************************************************************************************
    // helper functions
    // ********************************************************************************************

    fn get_table_contents(&self, ignore_zeros: bool) -> Vec<(Literal, &str, usize, usize)> {
        let mut table_content = Vec::with_capacity(self.literal_weights.len());
        for (index, weight) in self.literal_weights.iter().enumerate() {
            if index == 0 || index == 1 || (ignore_zeros && *weight == 0) {
                continue;
            }
            let literal = Literal::un_hash(index);
            // let literal_type = if self.state_variable_range.contains(literal.variable()) {
            //     "state"
            // } else if self.input_variable_range.contains(literal.variable()) {
            //     "input"
            // } else {
            //     "internal"
            // };
            let variable_weight = weight + self.literal_weights[(!literal).hash()];
            table_content.push((literal, "", *weight, variable_weight));
        }
        table_content
    }

    fn print_table_contents(&self, table_content: &[(Literal, &str, usize, usize)]) {
        // printing table

        for (i, (literal, literal_type, weight, variable_weight)) in
            table_content.iter().enumerate()
        {
            println!(
                "{}:\tliteral = {}\ttype = {}\tweigth = {}\tvariable weight = {}",
                i, literal, literal_type, weight, variable_weight
            );
        }
    }

    // ********************************************************************************************
    // API
    // ********************************************************************************************

    pub fn print_sorted_by_literal_weights(&self, ignore_zeros: bool) {
        let mut table_content = self.get_table_contents(ignore_zeros);
        table_content.sort_unstable_by_key(|x| x.2);
        self.print_table_contents(&table_content);
    }

    pub fn print_sorted_by_variable_weight(&self, ignore_zeros: bool) {
        let mut table_content = self.get_table_contents(ignore_zeros);
        table_content.sort_unstable_by_key(|x| x.3);
        self.print_table_contents(&table_content);
    }
}
