// ************************************************************************************************
// use
// ************************************************************************************************

use crate::models::and_inverter_graph::aig_node::{AIGNode, AIGNodeType};
use crate::models::and_inverter_graph::AndInverterGraph;
use std::fs;

use crate::models::{OnePassReader, Signal, TernaryValue, Utils};
use crate::models::{UniqueSortedHashMap, Wire};

// ************************************************************************************************
// impl
// ************************************************************************************************

impl AndInverterGraph {
    // ********************************************************************************************
    // helper functions
    // ********************************************************************************************

    // fn split_vector_by_newline(vec_of_bytes: &[u8]) -> Vec<Vec<u8>> {
    //     let mut result: Vec<Vec<u8>> = Vec::new();
    //     let mut current_line: Vec<u8> = Vec::new();
    //     for byte in vec_of_bytes {
    //         if byte == &b'\n' {
    //             result.push(current_line);
    //             current_line = Vec::new();
    //         } else {
    //             current_line.push(byte.to_owned());
    //         }
    //     }
    //     if !current_line.is_empty() {
    //         result.push(current_line);
    //     }
    //     result
    // }

    // // Function is private to not allow accidental creation of some random AIG.
    // fn new() -> Self {
    //     Self {
    //         /// these fields must be changed later, set them to max to notice if there is a bug
    //         maximum_variable_index: usize::MAX,
    //         number_of_inputs: usize::MAX,
    //         number_of_latches: usize::MAX,
    //         number_of_outputs: usize::MAX,
    //         number_of_and_gates: usize::MAX,
    //         number_of_bad_state_constraints: usize::MAX,
    //         number_of_invariant_constraints: usize::MAX,
    //         number_of_justice_constraints: usize::MAX,
    //         number_of_fairness_constraints: usize::MAX,

    //         /// the following vectors have default lengths.
    //         nodes: SignalHashMap::new(&Signal::new(0)),
    //         inputs: Vec::new(),
    //         latches: Vec::new(),
    //         outputs: Vec::new(),
    //         ands: Vec::new(),
    //         bad: Vec::new(),
    //         constraints: Vec::new(),
    //         comments: String::from(""),
    //         // justice: Vec::new(),
    //         // fairness: Vec::new(),
    //     }
    // }

    fn convert_string_to_number(str1: &str) -> Result<u32, String> {
        str1.parse::<u32>()
            .map_err(|e| format!("Could not convert string '{}' to number: {}", str1, e))
    }

    fn check_first_line_of_aig_and_load_it(
        &mut self,
        reader: &mut OnePassReader<'_>,
    ) -> Result<(), String> {
        let does_line_exist = reader.read_line_as_string();
        let is_line_utf8 =
            does_line_exist.ok_or("The parameter line (first line in aig file) does not exist.")?;
        let first_line_as_str = is_line_utf8?;
        // println!("first line: {}", first_line_as_str);
        let params: Vec<&str> = first_line_as_str.split_whitespace().collect();

        if params.is_empty() {
            return Err("The parameter line (first line in aig file) is empty.".to_string());
        }

        // check if the input file format is correct (starts with aig)
        Utils::ensure(
            params[0] == "aig",
            "The parameter line (first line in aig file) must start with the word 'aig'.",
        )?;
        Utils::ensure(
            params.len() > 5,
            "The parameter line (first line in aig file) has too few arguments.",
        )?;

        Utils::ensure(
            params.len() <= 10,
            "The parameter line (first line in aig file) has too many arguments.",
        )?;

        // first 5 fields always exist
        self.maximum_variable_index = Self::convert_string_to_number(params[1])?;
        self.number_of_inputs = Self::convert_string_to_number(params[2])?;
        self.number_of_latches = Self::convert_string_to_number(params[3])?;
        self.number_of_outputs = Self::convert_string_to_number(params[4])?;
        self.number_of_and_gates = Self::convert_string_to_number(params[5])?;

        // these fields do not always exist
        self.number_of_bad_state_constraints =
            Self::convert_string_to_number(params.get(6).unwrap_or(&"0"))?;
        self.number_of_invariant_constraints =
            Self::convert_string_to_number(params.get(7).unwrap_or(&"0"))?;
        self.number_of_justice_constraints =
            Self::convert_string_to_number(params.get(8).unwrap_or(&"0"))?;
        self.number_of_fairness_constraints =
            Self::convert_string_to_number(params.get(9).unwrap_or(&"0"))?;

        let highest_nodes = self
            .number_of_inputs
            .checked_add(self.number_of_latches)
            .and_then(|x| x.checked_add(self.number_of_and_gates))
            .ok_or("The number of inputs + latches + and gates is too large.".to_string())?;

        Utils::ensure(
            self.maximum_variable_index == highest_nodes,
            "The number of inputs + latches + and gates does not equal max signal in AIG.",
        )?;
        Utils::ensure(
            self.number_of_fairness_constraints == 0,
            "Fairness is currently unsupported.",
        )?;
        Utils::ensure(
            self.number_of_justice_constraints == 0,
            "Justice is currently unsupported.",
        )?;
        Ok(())
    }

    fn allocate_vectors(&mut self) {
        self.nodes = UniqueSortedHashMap::new(Signal::new(self.maximum_variable_index));
        self.nodes
            .insert(Signal::new(0), AIGNode::new(AIGNodeType::ConstantZero));

        self.outputs = Vec::with_capacity(self.number_of_outputs as usize);
        self.bad = Vec::with_capacity(self.number_of_bad_state_constraints as usize);
        self.constraints = Vec::with_capacity(self.number_of_invariant_constraints as usize);
        // self.justice = Vec::with_capacity(self.number_of_justice_constraints);
        // self.fairness = Vec::with_capacity(self.number_of_fairness_constraints);
    }

    /// notice that this function does not need to read from the file since in AIG
    /// the input literals are known (2, 4, .., 2 * number_of_inputs)
    fn create_input_nodes_of_aig(&mut self) {
        // assert!(self.aig_nodes.len() == 0);
        for i in 1..(self.number_of_inputs + 1) {
            let lit = 2 * i;
            let wire: Wire = Wire::new(lit);
            self.nodes
                .insert(wire.signal(), AIGNode::new(AIGNodeType::Input));
        }
    }

    fn check_literal(&self, literal_number: u32, line_num: usize) -> Result<(), String> {
        let var_number = literal_number >> 1;
        // assert!(2 <= literal_number, "Line {line_num}: '.aig' file contains literal {literal_number} which is reserved for constants.");
        Utils::ensure(var_number <= self.maximum_variable_index, format!("Line {line_num}: '.aig' file contains literal {literal_number} which is higher than maximum variable index.").as_str())?;
        Ok(())
    }

    fn create_latch_nodes_of_aig(&mut self, reader: &mut OnePassReader<'_>) -> Result<(), String> {
        for i in 0..self.number_of_latches {
            // latch literal is known because this is the binary AIGER format.
            let lit = 2 * (i + self.number_of_inputs + 1);
            let wire = Wire::new(lit);
            self.nodes.insert(
                wire.signal(),
                AIGNode::new(AIGNodeType::Latch {
                    input: Wire::new(u32::MAX),
                    reset: Wire::new(u32::MAX),
                }),
            );
            let does_line_exist = reader.read_line_as_string();
            let is_line_utf8 = does_line_exist.ok_or(format!(
                "Line {}: Latch line does not exist.",
                reader.get_line_number()
            ))?;
            let line_as_string = is_line_utf8?;

            let parsed_line: Vec<&str> = line_as_string.split_whitespace().collect();
            Utils::ensure(
                parsed_line.len() == 1 || parsed_line.len() == 2,
                format!(
                    "Line {}: Wrong number of arguments for latch line.",
                    reader.get_line_number()
                )
                .as_str(),
            )?;

            let next_lit = Self::convert_string_to_number(parsed_line[0])?;
            self.check_literal(next_lit, reader.get_line_number())?;
            self.nodes
                .get_mut(&wire.signal())
                .unwrap()
                .set_input_of_latch(Wire::new(next_lit));

            if parsed_line.len() == 2 {
                // latch has a reset literal
                let reset = Self::convert_string_to_number(parsed_line[1])?;
                Utils::ensure(
                    reset == 0 || reset == 1 || reset == lit,
                    format!("Line {}: Latch reset may be 0, 1, or equal to literal designated for latch.",reader.get_line_number()).as_str()
                )?;
                self.nodes
                    .get_mut(&wire.signal())
                    .unwrap()
                    .set_reset_of_latch(Wire::new(reset));
            } else {
                // latch does not have a reset literal (defaults to 0)
                // https://epub.jku.at/obvulioa/content/titleinfo/5973560/full.pdf
                self.nodes
                    .get_mut(&wire.signal())
                    .unwrap()
                    .set_reset_of_latch(Wire::new(0));
            }
        }
        Ok(())
    }

    fn create_output_nodes_of_aig(&mut self, reader: &mut OnePassReader<'_>) -> Result<(), String> {
        for _ in 0..self.number_of_outputs {
            let does_line_exist = reader.read_line_as_string();
            let is_line_utf8 = does_line_exist.ok_or(format!(
                "Line {}: Output line does not exist.",
                reader.get_line_number()
            ))?;
            let line_as_string = is_line_utf8?;

            let output_literal = Self::convert_string_to_number(&line_as_string)?;
            self.check_literal(output_literal, reader.get_line_number())?;
            self.outputs.push(Wire::new(output_literal));
        }
        Ok(())
    }

    fn create_bad_nodes_of_aig(&mut self, reader: &mut OnePassReader<'_>) -> Result<(), String> {
        // println!("Bad:");
        for _ in 0..self.number_of_bad_state_constraints {
            let does_line_exist = reader.read_line_as_string();
            let is_line_utf8 = does_line_exist.ok_or(format!(
                "Line {}: Bad line does not exist.",
                reader.get_line_number()
            ))?;
            let line_as_string = is_line_utf8?;

            let bad_literal = Self::convert_string_to_number(&line_as_string)?;
            // println!("{bad_literal}");
            self.check_literal(bad_literal, reader.get_line_number())?;
            self.bad.push(Wire::new(bad_literal));
        }
        Ok(())
    }

    fn create_invariant_constraint_nodes_of_aig(
        &mut self,
        reader: &mut OnePassReader<'_>,
    ) -> Result<(), String> {
        for _ in 0..self.number_of_invariant_constraints {
            let does_line_exist = reader.read_line_as_string();
            let is_line_utf8 = does_line_exist.ok_or(format!(
                "Line {}: Invariant constraint line does not exist.",
                reader.get_line_number()
            ))?;
            let line_as_string = is_line_utf8?;

            let inv_const_literal = Self::convert_string_to_number(&line_as_string)?;
            self.check_literal(inv_const_literal, reader.get_line_number())?;
            self.constraints.push(Wire::new(inv_const_literal));
        }
        Ok(())
    }

    fn get_max_literal_of_input_or_latch(&self) -> u32 {
        2 * (self.number_of_inputs + self.number_of_latches)
    }

    // fn get_position_of_start_of_and_segment(&self, bytes: &[u8]) -> Result<usize, String> {
    //     let mut read_index: usize = 0;
    //     let amount_of_lines_to_skip: usize = 1
    //         + self.number_of_latches
    //         + self.number_of_outputs
    //         + self.number_of_bad_state_constraints
    //         + self.number_of_invariant_constraints;
    //     let mut new_lines_seen = 0;
    //     while new_lines_seen < amount_of_lines_to_skip {
    //         if bytes
    //             .get(read_index)
    //             .ok_or("Unexpected end of file at start of and segment.".to_string())?
    //             == &b'\n'
    //         {
    //             new_lines_seen += 1;
    //         }
    //         read_index += 1;
    //     }
    //     Ok(read_index)
    // }

    pub fn read_delta(reader: &mut OnePassReader<'_>) -> Result<u32, String> {
        let mut i: usize = 0;
        let mut delta: u32 = 0;

        loop {
            let ch: u8 = reader.read_char().ok_or(format!(
                "Unexpected end of file while reading delta at line {}, colum {}.",
                reader.get_line_number(),
                reader.get_column()
            ))?;

            Utils::ensure(
                i < 5 || ((i < 6) && (ch < 0x10)),
                "One of the and gate deltas is too big and uses more than 32 bits, and thus cannot fit in u32.",
            )?;

            delta |= (ch as u32 & 0x7f) << (7 * i);
            i += 1;

            if ch & 0x80 == 0 {
                // this was the last byte of the delta
                break;
            }
        }

        Ok(delta)
    }

    fn create_and_nodes_of_aig(&mut self, reader: &mut OnePassReader<'_>) -> Result<(), String> {
        let mut lhs = self.get_max_literal_of_input_or_latch();

        for _ in 0..self.number_of_and_gates {
            lhs += 2;
            let delta = Self::read_delta(reader)?;
            Utils::ensure(delta <= lhs, "Invalid delta.")?;
            let rhs0: u32 = lhs - delta;

            let delta = Self::read_delta(reader)?;
            Utils::ensure(delta <= rhs0, "Invalid delta.")?;
            let rhs1: u32 = rhs0 - delta;

            // the assert is from https://github.com/arminbiere/aiger/blob/master/FORMAT
            // line 456 as of writing this.
            Utils::ensure(
                lhs > rhs0 && rhs0 >= rhs1,
                format!("Error (lhs > rhs0 >= rhs1) does not hold for and gate {lhs}").as_str(),
            )?;

            let wire = Wire::new(lhs);
            let rhs0_wire = Wire::new(rhs0);
            let rhs1_wire = Wire::new(rhs1);

            // let l1 = self.nodes.get(&rhs0_wire.get_signal()).unwrap().get_level();
            // let l2 = self.nodes.get(&rhs1_wire.get_signal()).unwrap().get_level();
            // let level = 1 + max(l1, l2);
            let node = AIGNode::new(AIGNodeType::And {
                input0: rhs0_wire,
                input1: rhs1_wire,
            });
            // node.set_rhs0_of_and(rhs0_wire);
            // node.set_rhs1_of_and(rhs1_wire);
            self.nodes.insert(wire.signal(), node);
        }

        Ok(())
    }

    fn add_symbol_to_node(
        &mut self,
        symbol_type: &str,
        symbol_number: u32,
        symbol: &str,
    ) -> Result<(), String> {
        if symbol_type == "i" {
            if symbol_number >= self.number_of_inputs {
                return Err(format!(
                    "Symbol {symbol_type}{symbol_number} is out of bounds (the index does not exist)."
                ));
            }

            // let node_index = Signal::new(symbol_number as u32 + 1);
            self.input_symbols.push((symbol_number, symbol.to_string()));
        } else if symbol_type == "l" {
            if symbol_number >= self.number_of_latches {
                return Err(format!(
                    "Symbol {symbol_type}{symbol_number} is out of bounds (the index does not exist)."
                ));
            }
            // let node_index = Signal::new(symbol_number as u32 + 1 + self.number_of_inputs as u32);
            self.latch_symbols.push((symbol_number, symbol.to_string()));
        } else if symbol_type == "o" {
            if symbol_number >= self.number_of_outputs {
                return Err(format!(
                    "Symbol {symbol_type}{symbol_number} is out of bounds (the index does not exist)."
                ));
            }
            self.output_symbols
                .push((symbol_number, symbol.to_string()));
        } else if symbol_type == "b" {
            if symbol_number >= self.number_of_bad_state_constraints {
                return Err(format!(
                    "Symbol {symbol_type}{symbol_number} is out of bounds (the index does not exist)."
                ));
            }
            self.bad_symbols.push((symbol_number, symbol.to_string()));
        } else if symbol_type == "c" {
            if symbol_number >= self.number_of_invariant_constraints {
                return Err(format!(
                    "Symbol {symbol_type}{symbol_number} is out of bounds (the index does not exist)."
                ));
            }
            self.constraint_symbols
                .push((symbol_number, symbol.to_string()));
        } else {
            unreachable!();
        }
        Ok(())
    }

    fn read_symbols_and_comments(&mut self, reader: &mut OnePassReader<'_>) -> Result<(), String> {
        // position_of_end_of_and_segment_plus_one == position where symbol table might begin
        while let Some(line) = reader.read_line_as_string() {
            let line_as_string = line?;

            if line_as_string == "c" {
                // comment segment started, we can read this till the end and return
                let rest_of_comments = reader.read_rest();
                let comment_section_as_is = std::str::from_utf8(&rest_of_comments)
                    .map_err(|e| e.to_string())?
                    .to_string();
                // erase null characters and add c\n to the beginning
                self.comments = "c\n".to_string()
                    + &(comment_section_as_is.replace(&char::from(0).to_string(), ""));
                break;
            } else {
                let (a, b) = line_as_string
                    .split_once(char::is_whitespace)
                    .ok_or_else(|| {
                        format!(
                            "Line {} ({line_as_string}): Symbol line should contain a whitespace.",
                            reader.get_line_number()
                        )
                    })?;
                let b = b.trim_start();
                let parsed_line: Vec<&str> = vec![a, b];
                // Utils::ensure(
                //     parsed_line.len() == 2,
                //     format!("Line '{line_as_string}': Wrong number of arguments for symbol line.")
                //         .as_str(),
                // )?;
                let mut symbol_and_variable_split: Vec<&str> = parsed_line[0].split("").collect();
                // "i0" gets split into vec!["" , "i", "0", ""], let's drop start and end.
                symbol_and_variable_split =
                    symbol_and_variable_split[1..(symbol_and_variable_split.len() - 1)].to_vec();
                Utils::ensure(
                    symbol_and_variable_split.len() > 1,
                    format!("Line '{line_as_string}': Symbol line should start with [ilobc]<pos>.")
                        .as_str(),
                )?;

                let symbol_type = symbol_and_variable_split[0];
                Utils::ensure(
                    ["i", "l", "o", "b", "c"].contains(&symbol_type),
                    format!("Line '{line_as_string}': Symbol line should start with [ilobc]<pos>.")
                        .as_str(),
                )?;
                let var_as_vector_of_strings = symbol_and_variable_split[1..].to_vec();
                let symbol_number_as_string = var_as_vector_of_strings.join("");
                let symbol_number = Self::convert_string_to_number(&symbol_number_as_string)?;
                self.add_symbol_to_node(symbol_type, symbol_number, parsed_line[1])?;
            }
        }
        Ok(())
    }

    fn perform_pass(&mut self, reader: &mut OnePassReader) -> Result<(), String> {
        self.check_first_line_of_aig_and_load_it(reader)?;
        self.allocate_vectors();
        self.create_input_nodes_of_aig();
        self.create_latch_nodes_of_aig(reader)?;
        self.create_output_nodes_of_aig(reader)?;
        self.create_bad_nodes_of_aig(reader)?;
        self.create_invariant_constraint_nodes_of_aig(reader)?;
        self.create_and_nodes_of_aig(reader)?;
        self.read_symbols_and_comments(reader)?;
        self.check()?;
        Ok(())
    }

    // ********************************************************************************************
    // aig creator
    // ********************************************************************************************

    /// Function that creates an vector of byte that represents an And Inverter Graph in .aig
    /// format and creates a memory representation of the aig.
    ///
    /// # Arguments
    ///
    /// * `vec_of_bytes` - the vector the contains an '.aig' file.
    ///
    /// # Examples
    ///
    pub fn from_vector_of_bytes(vec_of_bytes: &[u8]) -> Result<Self, String> {
        if vec_of_bytes.is_empty() {
            return Err(String::from(
                "Cannot create AIG from empty file or empty sequence of bytes.",
            ));
        }
        let mut reader: OnePassReader<'_> = OnePassReader::new(vec_of_bytes);
        let mut aig = Self {
            maximum_variable_index: u32::MAX,
            number_of_inputs: u32::MAX,
            number_of_latches: u32::MAX,
            number_of_outputs: u32::MAX,
            number_of_and_gates: u32::MAX,
            number_of_bad_state_constraints: u32::MAX,
            number_of_invariant_constraints: u32::MAX,
            number_of_justice_constraints: u32::MAX,
            number_of_fairness_constraints: u32::MAX,

            // the following vectors have default lengths.
            nodes: UniqueSortedHashMap::new(Signal::new(0)),
            outputs: Vec::new(),
            bad: Vec::new(),
            constraints: Vec::new(),
            comments: String::from(""),

            // symbols
            input_symbols: Vec::new(),
            latch_symbols: Vec::new(),
            output_symbols: Vec::new(),
            bad_symbols: Vec::new(),
            constraint_symbols: Vec::new(),
        };
        aig.perform_pass(&mut reader)?;
        Ok(aig)
    }

    /// Function that takes path to '.aig' file and creates a corresponding AndInverterGraph object.
    /// The '.aig' file is in accordance to <http://fmv.jku.at/aiger/>
    ///
    /// # Arguments
    ///
    /// * `file_path` - the path to the '.aig' file desired.
    ///
    pub fn from_aig_path(file_path: &str) -> Result<Self, String> {
        let file_as_vec_of_bytes = fs::read(file_path)
            .unwrap_or_else(|_| panic!("Unable to read the '.aig' file {file_path}"));
        Self::from_vector_of_bytes(&file_as_vec_of_bytes)
    }

    /// This is the way to create an AndInverterGraph object directly.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        max_signal: Signal,
        inputs: u32,
        latches: &[(Wire, TernaryValue)],
        output_wires: Vec<Wire>,
        bad_wires: Vec<Wire>,
        constraint_wires: Vec<Wire>,
        and_gates: &[(Wire, Wire)],
        comments: String,
    ) -> Result<Self, String> {
        let mut aig = Self {
            // these fields must be changed later, set them to max to notice if there is a bug
            maximum_variable_index: max_signal.number(),
            number_of_inputs: inputs,
            number_of_latches: latches.len() as u32,
            number_of_outputs: output_wires.len() as u32,
            number_of_and_gates: and_gates.len() as u32,
            number_of_bad_state_constraints: bad_wires.len() as u32,
            number_of_invariant_constraints: constraint_wires.len() as u32,
            number_of_justice_constraints: 0,
            number_of_fairness_constraints: 0,

            // the following vectors have default lengths.
            nodes: UniqueSortedHashMap::new(max_signal),
            outputs: output_wires,
            bad: bad_wires,
            constraints: constraint_wires,
            comments,

            // symbols
            input_symbols: Vec::new(),
            latch_symbols: Vec::new(),
            output_symbols: Vec::new(),
            bad_symbols: Vec::new(),
            constraint_symbols: Vec::new(),
        };

        aig.nodes
            .insert(Signal::new(0), AIGNode::new(AIGNodeType::ConstantZero));

        for signal in aig.get_input_signals().iter() {
            let node = AIGNode::new(AIGNodeType::Input);
            aig.nodes.insert(*signal, node);
        }

        for (signal, (input, reset)) in latches.iter().enumerate() {
            let signal = Signal::new(signal as u32 + 1 + aig.number_of_inputs);
            let node = AIGNode::new(AIGNodeType::Latch {
                input: *input,
                reset: match reset {
                    TernaryValue::True => Wire::new(1),
                    TernaryValue::False => Wire::new(0),
                    TernaryValue::X => signal.wire(false),
                },
            });
            aig.nodes.insert(signal, node);
        }

        for (signal, (input1, input2)) in and_gates.iter().enumerate() {
            let signal =
                Signal::new(signal as u32 + 1 + aig.number_of_inputs + aig.number_of_latches);
            let (a, b) = if input1 >= input2 {
                (input1, input2)
            } else {
                (input2, input1)
            };
            let node = AIGNode::new(AIGNodeType::And {
                input0: *a,
                input1: *b,
            });

            aig.nodes.insert(signal, node);
        }

        aig.check()?;
        Ok(aig)
    }
}
