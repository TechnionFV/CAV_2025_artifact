//! This structure makes making performant parsers easier by only reading the file once.
//! Design decisions:
//! - The file should be already in memory. While this differs from the standard library's
//!   BufReader, it is more performant since it makes less system calls.
//! - The file is read in one pass, so the file is only traversed once.
//! - The line number and column in that line are saved for easier error printing.
//! - The structure does not take ownership over the input file since there is no need to
//!   modify the vector.
//! - The file is expected as u8 since some formats (like AIGER) contain binary data for
//!   compactness.

// ************************************************************************************************
// use
// ************************************************************************************************

// ************************************************************************************************
// struct
// ************************************************************************************************

pub struct OnePassReader<'a> {
    slice: &'a [u8],
    line: usize,
    column: usize,
    i: usize,
    should_line_be_incremented_in_next_read: bool,
}

// ************************************************************************************************
// impl
// ************************************************************************************************

impl<'a> OnePassReader<'a> {
    // ********************************************************************************************
    // helper functions
    // ********************************************************************************************

    // ********************************************************************************************
    // API
    // ********************************************************************************************

    pub fn new(literals: &'a [u8]) -> Self {
        Self {
            slice: literals,
            line: 1,
            column: 1,
            i: 0,
            should_line_be_incremented_in_next_read: false,
        }
    }

    pub fn read_char(&mut self) -> Option<u8> {
        // increment if this is the start of a new line
        if self.should_line_be_incremented_in_next_read {
            self.should_line_be_incremented_in_next_read = false;
            self.line += 1;
            self.column = 1;
        }

        // read char if possible
        let c = self.slice.get(self.i);
        if let Some(cc) = c {
            let ccc = *cc;
            // increment read ptr
            self.i += 1;
            self.column += 1;
            // increment line counter
            if ccc == b'\n' {
                self.should_line_be_incremented_in_next_read = true;
            }
            Some(ccc)
        } else {
            // out of bounds
            None
        }
    }

    pub fn read_line_as_slice(&mut self) -> Option<&[u8]> {
        // let mut result = Vec::new();
        let start_index = self.i;
        let mut end_index = start_index;
        while let Some(c) = self.read_char() {
            if c == b'\n' {
                break;
            }
            // push after so as not to add newline
            end_index += 1;
            // result.push(c);
        }
        if start_index == end_index {
            None
        } else {
            Some(&self.slice[start_index..end_index])
        }
    }

    pub fn read_line_as_string_slice(&mut self) -> Option<Result<&str, String>> {
        let line = self.read_line_as_slice();
        line.map(|line| {
            let r1 = std::str::from_utf8(line);
            r1.map_err(|e| e.to_string())
            // r2.map(|str| str.to_string())
        })
    }

    pub fn read_line(&mut self) -> Option<Vec<u8>> {
        // get slice
        let line = self.read_line_as_slice();
        // copy on success
        line.map(|line| line.to_vec())
    }

    pub fn read_line_as_string(&mut self) -> Option<Result<String, String>> {
        // get slice
        let line = self.read_line_as_string_slice();
        // copy on success
        line.map(|r| r.map(|str| str.to_owned()))
    }

    pub fn read_rest(&mut self) -> Vec<u8> {
        let mut result = Vec::new();
        while let Some(c) = self.read_char() {
            result.push(c);
        }
        result
    }

    pub fn get_line_number(&self) -> usize {
        self.line
    }

    pub fn get_column(&self) -> usize {
        self.column
    }
}
