// ************************************************************************************************
// use
// ************************************************************************************************

use crate::models::and_inverter_graph::AndInverterGraph;
use crate::models::Wire;

// ************************************************************************************************
// impl
// ************************************************************************************************

impl AndInverterGraph {
    /// Function that converts an AndInverterGraph into '.aag' format as described in:
    /// The '.aag' file is in accordance to <http://fmv.jku.at/aiger/>
    ///
    /// # Arguments
    ///
    /// * `&self` - the AndInverterGraph desired for conversion.
    ///
    /// # Examples
    /// ```
    /// use rust_formal_verification::models::AndInverterGraph;
    /// use rust_formal_verification::models::Signal;
    /// use rust_formal_verification::models::TernaryValue;
    ///
    /// let aig = AndInverterGraph::new(
    ///     Signal::new(5),
    ///     0,
    ///     &[
    ///         (Signal::new(5).wire(false), TernaryValue::False),
    ///         (Signal::new(1).wire(false), TernaryValue::False),
    ///         (Signal::new(2).wire(false), TernaryValue::False),
    ///     ],
    ///     vec![Signal::new(5).wire(false)],
    ///     vec![],
    ///     vec![],
    ///     &[
    ///         (Signal::new(2).wire(true), Signal::new(3).wire(true)),
    ///         (
    ///             Signal::new(1).wire(true),
    ///             Signal::new(4).wire(false),
    ///         ),
    ///     ],
    ///     String::new(),
    /// )
    /// .unwrap();
    /// assert_eq!("aag 5 0 3 1 2\n2 10\n4 2\n6 4\n10\n8 7 5\n10 8 3\n", aig.get_aag_string());
    /// ```
    pub fn get_aag_string(&self) -> String {
        let mut result: Vec<String> = Vec::new();
        let mut symbol_table: Vec<String> = Vec::new();
        let mut first_line = vec![String::from("aag")];
        first_line.push(self.maximum_variable_index.to_string());
        first_line.push(self.number_of_inputs.to_string());
        first_line.push(self.number_of_latches.to_string());
        first_line.push(self.number_of_outputs.to_string());
        first_line.push(self.number_of_and_gates.to_string());
        assert!(self.number_of_justice_constraints + self.number_of_fairness_constraints == 0);
        if self.number_of_bad_state_constraints + self.number_of_invariant_constraints > 0 {
            first_line.push(self.number_of_bad_state_constraints.to_string());
        }
        if self.number_of_invariant_constraints > 0 {
            first_line.push(self.number_of_invariant_constraints.to_string());
        }
        result.push(first_line.join(" "));
        for input_index in self.get_input_signals().iter() {
            result.push(input_index.wire(false).number().to_string());
        }
        for latch_index in self.get_latch_information().iter().map(|x| x.output) {
            let mut line = Vec::new();
            let node = &self.nodes.get(&latch_index).unwrap();
            line.push(latch_index.wire(false).number().to_string());
            line.push(node.get_latch_input().number().to_string());
            if node.get_latch_reset() != Wire::new(0) {
                line.push(node.get_latch_reset().number().to_string());
            }
            result.push(line.join(" "));
        }
        for output_literal in self.outputs.iter() {
            result.push(output_literal.number().to_string());
        }
        for bad_literal in self.bad.iter() {
            result.push(bad_literal.number().to_string());
        }
        for constraint_literal in self.constraints.iter() {
            result.push(constraint_literal.number().to_string());
        }

        let a = self.input_symbols.iter().map(|(x, y)| ('i', x, y));
        let b = self.latch_symbols.iter().map(|(x, y)| ('l', x, y));
        let c = self.output_symbols.iter().map(|(x, y)| ('o', x, y));
        let d = self.bad_symbols.iter().map(|(x, y)| ('b', x, y));
        let e = self.constraint_symbols.iter().map(|(x, y)| ('c', x, y));
        for (x, i, symbol) in a.chain(b).chain(c).chain(d).chain(e) {
            symbol_table.push(format!("{x}{} {}", i, symbol));
        }

        for and_index in self.get_all_and_gates().iter().map(|x| &x.out) {
            let node = &self.nodes.get(and_index).unwrap();
            let lhs = and_index.wire(false).number();
            let rhs0 = node.get_and_rhs0().number();
            let rhs1 = node.get_and_rhs1().number();
            result.push(format!("{lhs} {rhs0} {rhs1}"));
        }
        result.append(&mut symbol_table);
        result.append(&mut vec![self.comments.to_owned()]);
        let mut final_res = result.join("\n");
        if final_res.ends_with('\n') {
            // do nothing
        } else {
            // add new line at aag end.
            final_res.push('\n');
        }
        final_res
    }

    fn write_header_segment(&self, result: &mut Vec<u8>) {
        let mut first_line = vec![String::from("aig")];
        first_line.push(self.maximum_variable_index.to_string());
        first_line.push(self.number_of_inputs.to_string());
        first_line.push(self.number_of_latches.to_string());
        first_line.push(self.number_of_outputs.to_string());
        first_line.push(self.number_of_and_gates.to_string());
        assert!(self.number_of_justice_constraints + self.number_of_fairness_constraints == 0);
        if self.number_of_bad_state_constraints + self.number_of_invariant_constraints > 0 {
            first_line.push(self.number_of_bad_state_constraints.to_string());
        }
        if self.number_of_invariant_constraints > 0 {
            first_line.push(self.number_of_invariant_constraints.to_string());
        }
        result.extend(first_line.join(" ").as_bytes());
        result.push(b'\n');
    }

    fn write_latch_segment(&self, result: &mut Vec<u8>) {
        for latch in self.get_latch_information().iter().map(|x| &x.output) {
            let latch_input = self.nodes.get(latch).unwrap().get_latch_input();
            let latch_reset = self.nodes.get(latch).unwrap().get_latch_reset();
            let line = if latch_reset == Wire::CONSTANT_ZERO {
                format!("{}\n", latch_input.number())
            } else {
                format!("{} {}\n", latch_input.number(), latch_reset.number())
            };
            result.extend(line.as_bytes());
        }
    }

    fn write_output_segment(&self, result: &mut Vec<u8>) {
        for output in self.outputs.iter() {
            let line = format!("{}\n", output.number());
            result.extend(line.as_bytes());
        }
    }

    fn write_bad_segment(&self, result: &mut Vec<u8>) {
        for output in self.bad.iter() {
            let line = format!("{}\n", output.number());
            result.extend(line.as_bytes());
        }
    }

    fn write_constraint_segment(&self, result: &mut Vec<u8>) {
        for output in self.constraints.iter() {
            let line = format!("{}\n", output.number());
            result.extend(line.as_bytes());
        }
    }

    fn write_delta(mut delta: u32, result: &mut Vec<u8>) {
        loop {
            let byte = (delta & 0x7f) as u8;
            delta >>= 7;
            if delta > 0 {
                result.push(byte | 0x80);
            } else {
                result.push(byte);
            }
            if delta == 0 {
                break;
            }
        }
    }

    fn write_and_segment(&self, result: &mut Vec<u8>) {
        for and_index in self.get_all_and_gates().iter().map(|x| &x.out) {
            let node = &self.nodes.get(and_index).unwrap();
            let lhs = and_index.wire(false).number();
            let rhs0 = node.get_and_rhs0().number();
            let rhs1 = node.get_and_rhs1().number();
            let delta_1 = lhs - rhs0;
            let delta_2 = rhs0 - rhs1;
            Self::write_delta(delta_1, result);
            Self::write_delta(delta_2, result);
        }
    }

    fn write_symbol_table(&self, result: &mut Vec<u8>) {
        let a = self.input_symbols.iter().map(|(x, y)| ('i', x, y));
        let b = self.latch_symbols.iter().map(|(x, y)| ('l', x, y));
        let c = self.output_symbols.iter().map(|(x, y)| ('o', x, y));
        let d = self.bad_symbols.iter().map(|(x, y)| ('b', x, y));
        let e = self.constraint_symbols.iter().map(|(x, y)| ('c', x, y));
        for (x, i, symbol) in a.chain(b).chain(c).chain(d).chain(e) {
            result.extend(format!("{x}{} {}\n", i, symbol).as_bytes());
        }
    }

    fn write_comment_section(&self, result: &mut Vec<u8>) {
        result.extend(self.comments.as_bytes());
        if (!self.comments.is_empty()) && (!self.comments.ends_with('\n')) {
            result.push(b'\n');
        }
    }

    pub fn get_aig(&self) -> Vec<u8> {
        let mut result: Vec<u8> = Vec::new();
        self.write_header_segment(&mut result);
        self.write_latch_segment(&mut result);
        self.write_output_segment(&mut result);
        self.write_bad_segment(&mut result);
        self.write_constraint_segment(&mut result);
        self.write_and_segment(&mut result);
        self.write_symbol_table(&mut result);
        self.write_comment_section(&mut result);

        result
    }
}

#[test]
fn check_same_delta() {
    use rand::{thread_rng, Rng};
    let deltas: Vec<u32> = (0..1_000_000).map(|_| thread_rng().gen()).collect();

    let mut result: Vec<u8> = Vec::new();
    for delta in deltas.iter().copied() {
        AndInverterGraph::write_delta(delta, &mut result);
    }

    let mut reader = crate::models::OnePassReader::new(&result);

    for delta in deltas.into_iter() {
        // print!("{}, ", delta);
        let delta_: u32 = AndInverterGraph::read_delta(&mut reader).unwrap();
        assert_eq!(delta, delta_);
    }
}
