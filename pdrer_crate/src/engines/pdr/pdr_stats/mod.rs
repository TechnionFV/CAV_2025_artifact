// ************************************************************************************************
// use
// ************************************************************************************************

use std::{fmt, process::Command};

use fxhash::FxHashMap;

use crate::{models::PrettyTable, solvers::sat::incremental::SatResult};

// ************************************************************************************************
// struct
// ************************************************************************************************

#[derive(Debug, Clone)]
pub struct PDRStats {
    ternary_simulation_reductions: Vec<(usize, usize)>,
    generalization_reductions: Vec<(usize, usize)>,
    // number_of_sat_calls_sat: usize,
    // number_of_sat_calls_un_sat: usize,
    // number_of_ev_sat_calls_sat: usize,
    // number_of_ev_sat_calls_un_sat: usize,
    // total_proof_obligations_count: usize,
    lic_analysis_calls: usize,
    lic_analysis_successful_calls: usize,
    generic_counts: FxHashMap<&'static str, usize>,
}

// ************************************************************************************************
// impl
// ************************************************************************************************

impl PDRStats {
    pub fn new() -> Self {
        Self {
            ternary_simulation_reductions: vec![],
            generalization_reductions: vec![],
            // number_of_sat_calls_sat: 0,
            // number_of_sat_calls_un_sat: 0,
            // number_of_ev_sat_calls_sat: 0,
            // number_of_ev_sat_calls_un_sat: 0,
            // total_proof_obligations_count: 0,
            lic_analysis_calls: 0,
            lic_analysis_successful_calls: 0,
            generic_counts: Default::default(),
        }
    }

    pub fn note_ternary_simulation(&mut self, size_before: usize, size_after: usize) {
        self.ternary_simulation_reductions
            .push((size_before, size_after));
    }

    pub fn note_generalization(&mut self, size_before: usize, size_after: usize) {
        self.generalization_reductions
            .push((size_before, size_after));
    }

    pub fn note_sat_call(&mut self, r: &SatResult) {
        self.increment_generic_count("Number of SAT calls");
        match r {
            SatResult::Sat => self.increment_generic_count("Number of SAT calls (SAT)"),
            SatResult::UnSat => self.increment_generic_count("Number of SAT calls (UNSAT)"),
        }
    }

    pub fn note_ev_sat_call(&mut self, r: &SatResult) {
        self.increment_generic_count("Number of EV SAT calls");
        match r {
            SatResult::Sat => self.increment_generic_count("Number of EV SAT calls (SAT)"),
            SatResult::UnSat => self.increment_generic_count("Number of EV SAT calls (UNSAT)"),
        }
    }

    // pub fn update_proof_obligations_count(&mut self) {
    //     self.total_proof_obligations_count += 1;
    // }

    // pub fn get_proof_obligations_count(&self) -> usize {
    //     self.total_proof_obligations_count
    // }

    pub fn note_lic_analysis(&mut self, was_successful: bool) {
        self.lic_analysis_calls += 1;
        if was_successful {
            self.lic_analysis_successful_calls += 1;
        }
    }

    pub fn get_lic_stats(&self) -> (usize, usize) {
        (self.lic_analysis_calls, self.lic_analysis_successful_calls)
    }

    pub fn increment_generic_count(&mut self, name: &'static str) {
        *self.generic_counts.entry(name).or_insert(0) += 1;
    }

    fn get_memory_usage() -> Option<usize> {
        // This command will work on both Linux and macOS.
        let output = Command::new("ps")
            .args(["-o", "rss=", "-p", &std::process::id().to_string()])
            .output()
            .ok()?;

        let memory_kb = String::from_utf8_lossy(&output.stdout)
            .trim()
            .parse::<usize>()
            .ok()?;
        Some(memory_kb / 1024) // Convert from KB to MB
    }

    pub fn print_memory_usage(title: &str) {
        println!(
            "Memory usage ({title}): {} MB",
            Self::get_memory_usage().unwrap_or_default()
        );
    }

    pub fn get_stats(&self) -> Vec<(String, String)> {
        let t = self.ternary_simulation_reductions.len();
        let sum_before_ternary = self
            .ternary_simulation_reductions
            .iter()
            .map(|(a, _)| a)
            .sum::<usize>();
        let sum_after_ternary = self
            .ternary_simulation_reductions
            .iter()
            .map(|(_, a)| a)
            .sum::<usize>();

        let g = self.generalization_reductions.len();
        let sum_before_generalize = self
            .generalization_reductions
            .iter()
            .map(|(a, _)| a)
            .sum::<usize>();
        let sum_after_generalize = self
            .generalization_reductions
            .iter()
            .map(|(_, a)| a)
            .sum::<usize>();
        let x = vec![
            (
                "Average state size before ternary simulation".to_string(),
                format!("{:.4}", sum_before_ternary as f64 / t as f64),
            ),
            (
                "Average state size after ternary simulation".to_string(),
                format!("{:.4}", sum_after_ternary as f64 / t as f64),
            ),
            (
                "Number of ternary simulation calls".to_string(),
                t.to_string(),
            ),
            (
                "Average state size before generalization".to_string(),
                format!("{:.4}", sum_before_generalize as f64 / g as f64),
            ),
            (
                "Average state size after generalization".to_string(),
                format!("{:.4}", sum_after_generalize as f64 / g as f64),
            ),
            (
                "Number of generalize simulation calls".to_string(),
                g.to_string(),
            ),
            // (
            //     "Number of SAT calls".to_string(),
            //     (self.number_of_sat_calls_sat + self.number_of_sat_calls_un_sat).to_string(),
            // ),
            // (
            //     "Number of SAT calls (SAT)".to_string(),
            //     self.number_of_sat_calls_sat.to_string(),
            // ),
            // (
            //     "Number of SAT calls (UNSAT)".to_string(),
            //     self.number_of_sat_calls_un_sat.to_string(),
            // ),
            // (
            //     "Number of EV SAT calls".to_string(),
            //     (self.number_of_ev_sat_calls_sat + self.number_of_ev_sat_calls_un_sat).to_string(),
            // ),
            // (
            //     "Number of EV SAT calls (SAT)".to_string(),
            //     self.number_of_ev_sat_calls_sat.to_string(),
            // ),
            // (
            //     "Number of EV SAT calls (UNSAT)".to_string(),
            //     self.number_of_ev_sat_calls_un_sat.to_string(),
            // ),
            (
                "Number of LIC analysis calls".to_string(),
                self.lic_analysis_calls.to_string(),
            ),
            (
                "Number of LIC analysis calls (successful)".to_string(),
                self.lic_analysis_successful_calls.to_string(),
            ),
            // (
            //     "Total proof obligations count".to_string(),
            //     self.total_proof_obligations_count.to_string(),
            // ),
            (
                "Total memory used (MB)".to_string(),
                Self::get_memory_usage().unwrap_or_default().to_string(),
            ),
        ];
        let mut v: Vec<(String, String)> = self
            .generic_counts
            .iter()
            .map(|(a, b)| (a.to_string(), b.to_string()))
            .collect();
        v.sort_unstable();

        [x, v].concat()
    }
}

// ************************************************************************************************
// Deafult
// ************************************************************************************************

impl Default for PDRStats {
    fn default() -> Self {
        Self::new()
    }
}

// ************************************************************************************************
// FMT
// ************************************************************************************************

impl fmt::Display for PDRStats {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let stats = self.get_stats();
        let mut table = PrettyTable::new(vec!["Name".to_string(), "Value".to_string()]);
        for (name, value) in stats {
            table.add_row(vec![name, value]).unwrap();
        }
        write!(f, "{}", table)
    }
}
