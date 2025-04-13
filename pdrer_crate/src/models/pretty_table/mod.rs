/// Table for printing tables in a pretty way.
///
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct PrettyTable {
    header: Vec<String>,
    rows: Vec<Vec<String>>,
}

// ************************************************************************************************
// impl
// ************************************************************************************************

impl PrettyTable {
    pub fn new(header: Vec<String>) -> Self {
        Self {
            header,
            rows: vec![],
        }
    }

    pub fn add_row(&mut self, row: Vec<String>) -> Result<(), String> {
        if row.len() != self.header.len() {
            return Err(
                "The row does not have the same number of columns as the header.".to_string(),
            );
        }
        self.rows.push(row);
        Ok(())
    }

    pub fn get_column_widths(&self) -> Vec<usize> {
        let mut column_widths = vec![0; self.header.len()];
        for row in self.rows.iter().chain(std::iter::once(&self.header)) {
            for (i, cell) in row.iter().enumerate() {
                column_widths[i] = column_widths[i].max(cell.len());
            }
        }
        column_widths
    }
}

// ************************************************************************************************
// fmt
// ************************************************************************************************

impl fmt::Display for PrettyTable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        const LEFT_SEPERATOR: &str = "| ";
        const RIGHT_SEPERATOR: &str = " |";
        const MIDDLE_SEPERATOR: &str = " | ";

        let column_widths = self.get_column_widths();
        let total_width = {
            let a = column_widths.iter().sum::<usize>();
            let b = LEFT_SEPERATOR.len();
            let c = MIDDLE_SEPERATOR.len() * (column_widths.len() - 1);
            let d = RIGHT_SEPERATOR.len();
            a + b + c + d
        };

        writeln!(f, "{}", "-".repeat(total_width))?;
        write!(f, "{}", LEFT_SEPERATOR)?;
        for (i, cell) in self.header.iter().enumerate() {
            write!(f, "{:width$}", cell, width = column_widths[i])?;
            if i < self.header.len() - 1 {
                write!(f, "{}", MIDDLE_SEPERATOR)?;
            }
        }
        writeln!(f, "{}", RIGHT_SEPERATOR)?;
        writeln!(f, "{}", "-".repeat(total_width))?;

        for row in self.rows.iter() {
            write!(f, "{}", LEFT_SEPERATOR)?;
            for (i, cell) in row.iter().enumerate() {
                write!(f, "{:width$}", cell, width = column_widths[i])?;
                if i < row.len() - 1 {
                    write!(f, "{}", MIDDLE_SEPERATOR)?;
                }
            }
            writeln!(f, "{}", RIGHT_SEPERATOR)?;
        }
        write!(f, "{}", "-".repeat(total_width))

        // Ok(())
    }
}
