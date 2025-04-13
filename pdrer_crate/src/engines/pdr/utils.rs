// ************************************************************************************************
// use
// ************************************************************************************************

use super::{PropertyDirectedReachability, PropertyDirectedReachabilitySolver};
use crate::{
    models::{definition::DefinitionFunction, PrettyTable},
    solvers::dd::DecisionDiagramManager,
};

// ************************************************************************************************
// const
// ************************************************************************************************

// ************************************************************************************************
// impl
// ************************************************************************************************

impl<T: PropertyDirectedReachabilitySolver, D: DecisionDiagramManager>
    PropertyDirectedReachability<T, D>
{
    // ********************************************************************************************
    // printing API
    // ********************************************************************************************

    pub(super) fn print_start_message_if_verbose(&self) {
        if self.s.parameters.verbose {
            println!("Parameters:");
            println!("{}", self.s.parameters);
            self.print_progress_if_verbose("START");
        }
    }

    pub(super) fn print_progress_if_verbose(&self, title: &str) {
        if self.s.parameters.verbose {
            let mut stats = vec![("Title".to_string(), title.to_string())];
            stats.push(("Depth".to_string(), self.frames.depth().to_string()));
            stats.append(&mut self.s.pdr_stats.borrow().get_stats());
            stats.push((
                "Proof obligations".to_string(),
                self.proof_obligations.len().to_string(),
            ));
            stats.push((
                "Trace Tree size".to_string(),
                self.proof_obligations.get_trace_tree().len().to_string(),
            ));
            stats.push((
                "Extension Variables".to_string(),
                self.frames.get_definitions().len().to_string(),
            ));
            stats.push((
                "XOR Extension Variables".to_string(),
                self.frames
                    .get_definitions()
                    .iter()
                    .filter(|d| d.function == DefinitionFunction::Xor)
                    .count()
                    .to_string(),
            ));
            stats.push((
                "AND Extension Variables".to_string(),
                self.frames
                    .get_definitions()
                    .iter()
                    .filter(|d| d.function == DefinitionFunction::And)
                    .count()
                    .to_string(),
            ));
            stats.push((
                "Elapsed".to_string(),
                self.s
                    .parameters
                    .start_time
                    .unwrap()
                    .elapsed()
                    .as_secs_f32()
                    .to_string(),
            ));
            const AMOUNF_OF_FRAMES_TO_PRINT: usize = 10;
            for (i, f) in self
                .frames
                .iter()
                .enumerate()
                .rev()
                .take(AMOUNF_OF_FRAMES_TO_PRINT)
                .rev()
            {
                if i == self.frames.len() - 1 {
                    stats.push(("Delta of Infinite Frame".to_string(), f.len().to_string()));
                } else {
                    stats.push((format!("Delta of Frame {}", i), f.len().to_string()));
                }
            }
            stats.push((
                "Total Clauses".to_string(),
                self.frames
                    .iter()
                    .map(|f| f.len())
                    .sum::<usize>()
                    .to_string(),
            ));
            const HORIZONTAL_OR_VERTICAL: bool = false;
            let pretty_table = if HORIZONTAL_OR_VERTICAL {
                let mut t = PrettyTable::new(stats.iter().map(|(k, _)| k.clone()).collect());
                t.add_row(stats.iter().map(|(_, v)| v.clone()).collect())
                    .unwrap();
                t
            } else {
                let mut t = PrettyTable::new(vec!["Key".to_string(), "Value".to_string()]);
                for (k, v) in stats {
                    t.add_row(vec![k, v]).unwrap();
                }
                t
            };
            println!("{}", pretty_table);
            if self.s.parameters.should_print_time_stats_during_run {
                self.print_time_statistics_if_verbose();
            }
        }
    }

    pub(super) fn print_time_statistics_if_verbose(&self) {
        if self.s.parameters.verbose {
            println!("Time Statistics:");
            println!("{}", self.s.time_stats.borrow());
            self.s
                .fin_state
                .borrow()
                .print_ternary_simulation_time_stats();
        }
    }

    pub(super) fn print_final_message_if_verbose(&self, was_error: bool) {
        if self.s.parameters.verbose {
            if was_error {
                self.print_progress_if_verbose("ERROR");
            } else {
                self.print_progress_if_verbose("DONE");
            }
            if !self.s.parameters.should_print_time_stats_during_run {
                self.print_time_statistics_if_verbose();
            }
            println!("definition library:\n{}", self.frames.get_definitions_lib());
        }
    }
}
