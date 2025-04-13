// ************************************************************************************************
// use
// ************************************************************************************************

use crate::models::PrettyTable;

use super::TimeStats;
use std::fmt;
use std::time::{Duration, Instant};

// ************************************************************************************************
// impl
// ************************************************************************************************

impl TimeStats {
    // ********************************************************************************************
    // API
    // ********************************************************************************************

    pub fn new() -> Self {
        Self {
            times: Default::default(),
            start_time: Instant::now(),
        }
    }

    pub fn update_time(&mut self, name: &'static str, duration: Duration) {
        let t = self
            .times
            .entry(super::RefEquality(name))
            .or_insert((Duration::ZERO, 0));
        t.0 += duration;
        t.1 += 1;
    }
}

// ************************************************************************************************
// impl Default
// ************************************************************************************************

impl Default for TimeStats {
    fn default() -> Self {
        Self::new()
    }
}

// ************************************************************************************************
// impl Default
// ************************************************************************************************

impl fmt::Display for TimeStats {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let total = self.start_time.elapsed().as_secs_f32();
        let mut v: Vec<_> = self.times.iter().collect();
        v.sort_by(|a, b| a.0.cmp(b.0));

        let mut table = PrettyTable::new(vec![
            "Name".to_string(),
            "Total".to_string(),
            "Count".to_string(),
            "Average".to_string(),
            "Percentage".to_string(),
        ]);
        for (name, (time, number_of_updates)) in v {
            let time_as_f32 = time.as_secs_f32();
            let percentage = (time_as_f32 / total) * 100.0;
            let average = time_as_f32 / *number_of_updates as f32;
            table
                .add_row(vec![
                    format!("{}", name.0.strip_suffix("::f").unwrap_or(name.0)),
                    format!("{:.3}", time_as_f32),
                    number_of_updates.to_string(),
                    format!("{:.5}", average),
                    format!("{:.3}", percentage),
                ])
                .unwrap();
        }
        writeln!(f, "{}", table)?;
        writeln!(f, "Total time = {:.3}", total)?;

        Ok(())
    }
}
