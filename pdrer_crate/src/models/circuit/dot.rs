// ************************************************************************************************
// use
// ************************************************************************************************

use super::{node_types::CircuitNodeType, Circuit};
use crate::models::{Signal, Wire};
use dot_writer::{Attributes, Color, DotWriter, Shape};
use std::path::Path;

// ************************************************************************************************
// impl
// ************************************************************************************************

impl Circuit {
    // ********************************************************************************************
    // aig getting node info
    // ********************************************************************************************

    pub fn get_dot(&self) -> String {
        let mut output_bytes = Vec::new();
        {
            let mut writer = DotWriter::from(&mut output_bytes);
            let mut digraph = writer.digraph();
            digraph.set_rank_direction(dot_writer::RankDirection::TopBottom);
            digraph.set("newrank", "true", false);

            digraph.set("overlap", "false", false);
            // digraph.set("splines", "true", false);

            // ground
            if self.nodes.contains_key(&Signal::new(0)) {
                let mut node = digraph.node_named("0");
                node.set_label("0 Ground");
                node.set_shape(Shape::Rectangle);
                node.set_color(Color::Red);
            }

            // inputs
            {
                let mut cluster = digraph.cluster();
                cluster.set_label("Inputs");
                // cluster.set_rank(dot_writer::Rank::Same);
                // cluster.node_attributes().set_style(Style::Filled);
                cluster.set_color(Color::Blue);
                for input in self.get_input_signals().iter() {
                    let mut node = cluster.node_named(input.number().to_string());
                    node.set_label(format!("{} Input", input.number()).as_str());
                    node.set_shape(Shape::Circle);
                    node.set_color(Color::Blue);
                }
            }

            // latches
            {
                let mut cluster = digraph.cluster();
                cluster.set_label("Latches");
                cluster.set_rank(dot_writer::Rank::Same);
                // cluster.node_attributes().set_style(Style::Filled);
                cluster.set_color(Color::PaleGreen);
                for latch in self.get_latch_signals().iter() {
                    let initial: crate::models::TernaryValue = {
                        let node = &self.get_node(latch).unwrap().node_type;
                        if let CircuitNodeType::Latch(l) = node {
                            l.initial
                        } else {
                            unreachable!()
                        }
                    };

                    let mut node = cluster.node_named(latch.number().to_string());
                    let string = format!("{} Latch\n{}", latch.number(), initial);
                    node.set_label(&string);
                    node.set_shape(Shape::Msquare);
                    node.set_color(Color::PaleGreen);
                }
            }

            // gates
            {
                // let mut cluster = digraph.cluster();
                // cluster.set_label("And Gates");
                // cluster.node_attributes().set_style(Style::Filled);
                // cluster.set_color(Color::Black);

                // level
                let levels = self.get_level_per_signal();
                let max_level = *levels.iter_items().max().unwrap();
                for level in 1..=max_level {
                    let mut cluster = digraph.cluster();
                    // cluster.set_label(format!("Level {}", level).as_str());
                    cluster.set_rank(dot_writer::Rank::Same);
                    cluster.set_color(Color::White);
                    // sub_graph.set_rank(dot_writer::Rank::Same);
                    let mut v: Vec<Signal> = self.gates.iter().copied().collect();
                    v.sort_unstable();
                    for gate_signal in v {
                        let gate = &self.nodes.get(&gate_signal).unwrap();
                        let gate_level = *levels.get(&gate_signal).unwrap();
                        if gate_level != level {
                            continue;
                        }
                        let mut node = cluster.node_named(gate_signal.number().to_string());
                        match &gate.node_type {
                            CircuitNodeType::And { .. } => {
                                node.set_label(format!("{} And", gate_signal.number()).as_str());
                                node.set_shape(Shape::Rectangle);
                            }
                            CircuitNodeType::GenericGate { .. } => {
                                node.set_label(
                                    format!("{} Generic", gate_signal.number()).as_str(),
                                );
                                node.set_shape(Shape::Record);
                            }
                            _ => unreachable!(),
                        }
                        node.set_rank(dot_writer::Rank::Min);
                        node.set_color(Color::Black);
                    }
                }
            }

            // output nodes
            let out_to_vertex_number = |w: &Wire| {
                let base = self.get_highest_signal().number() + 1;
                let offset = (w.get_signal_number() << 1) + if w.is_negated() { 1 } else { 0 };
                base + offset
            };
            {
                let mut cluster = digraph.cluster();
                cluster.set_label("Outputs");
                cluster.set_rank(dot_writer::Rank::Same);
                cluster.set_color(Color::PaleTurquoise);
                for output in self.get_output_wires().iter() {
                    let mut node = cluster.node_named(out_to_vertex_number(output).to_string());
                    node.set_label(format!("Output {}", output).as_str());
                    node.set_shape(Shape::Mrecord);
                    node.set_color(Color::PaleTurquoise);
                }
            }

            let bad_to_vertex_number = |w: &Wire| {
                let base = (self.get_highest_signal().number() + 1) << 2;
                let offset = (w.get_signal_number() << 1) + if w.is_negated() { 1 } else { 0 };
                base + offset
            };
            // bad nodes
            {
                let mut cluster = digraph.cluster();
                cluster.set_rank(dot_writer::Rank::Same);
                cluster.set_label("Bad Outputs");
                cluster.set_color(Color::PaleTurquoise);
                for bad in self.get_bad_wires().iter() {
                    let mut node = cluster.node_named(bad_to_vertex_number(bad).to_string());
                    node.set_label(format!("Bad {}", bad).as_str());
                    node.set_shape(Shape::Mrecord);
                    node.set_color(Color::PaleTurquoise);
                }
            }

            let constraint_to_vertex_number = |w: &Wire| {
                let base = (self.get_highest_signal().number() + 1) << 4;
                let offset = (w.get_signal_number() << 1) + if w.is_negated() { 1 } else { 0 };
                base + offset
            };
            // bad nodes
            {
                let mut cluster = digraph.cluster();
                cluster.set_rank(dot_writer::Rank::Same);
                cluster.set_label("Constraint Outputs");
                cluster.set_color(Color::PaleTurquoise);
                for bad in self.get_invariant_constraint_wires().iter() {
                    let mut node = cluster.node_named(constraint_to_vertex_number(bad).to_string());
                    node.set_label(format!("Constraint {}", bad).as_str());
                    node.set_shape(Shape::Mrecord);
                    node.set_color(Color::PaleTurquoise);
                }
            }

            // edges
            {
                let negate_edge_and_rev = |is_negated: bool, e: dot_writer::EdgeList<'_, '_>| {
                    if is_negated {
                        e.attributes().set_color(Color::Red);
                        // .set("headport", "n", false)
                        // .set("tailport", "s", false);
                    } else {
                        e.attributes().set_color(Color::Black);
                        // .set("headport", "n", false)
                        // .set("tailport", "s", false);
                    }
                };

                let negate_and_direct = |is_negated: bool, e: dot_writer::EdgeList<'_, '_>| {
                    if is_negated {
                        e.attributes().set_color(Color::Red);
                        // .set("headport", "n", false)
                        // .set("tailport", "s", false);
                    } else {
                        e.attributes().set_color(Color::Black);
                        // .set("headport", "n", false)
                        // .set("tailport", "s", false);
                    }
                };

                let mut v: Vec<Signal> = self.gates.iter().copied().collect();
                v.sort_unstable();
                for gate_signal in v {
                    let gate = &self.nodes.get(&gate_signal).unwrap();
                    let inputs = match &gate.node_type {
                        CircuitNodeType::And(a) => a.inputs.peek(),
                        CircuitNodeType::GenericGate(g) => {
                            &(g.truth_table
                                .get_signals()
                                .iter()
                                .map(|s| s.wire(false))
                                .collect::<Vec<Wire>>())
                        }
                        _ => unreachable!(),
                    };

                    let to = gate_signal.number().to_string();
                    for input in inputs.iter() {
                        let from = input.get_signal_number().to_string();
                        let e = digraph.edge(from.as_str(), to.as_str());
                        negate_and_direct(input.is_negated(), e);
                    }
                }

                for latch in self.get_latch_signals().iter() {
                    let node = self.nodes.get(latch).unwrap();
                    let input = match &node.node_type {
                        CircuitNodeType::Latch(l) => l.input,
                        _ => unreachable!(),
                    };
                    let to = latch.number().to_string();
                    let from = input.get_signal_number().to_string();
                    let e = digraph.edge(from.as_str(), to.as_str());
                    negate_edge_and_rev(input.is_negated(), e);
                }

                for output in self.get_output_wires().iter() {
                    let to = out_to_vertex_number(output).to_string();
                    let from = output.get_signal_number().to_string();
                    let e = digraph.edge(from.as_str(), to.as_str());
                    negate_and_direct(output.is_negated(), e);
                }
                for bad in self.get_bad_wires().iter() {
                    let to = bad_to_vertex_number(bad).to_string();
                    let from = bad.get_signal_number().to_string();
                    let e = digraph.edge(from.as_str(), to.as_str());
                    negate_and_direct(bad.is_negated(), e);
                }
                for constraint in self.get_invariant_constraint_wires().iter() {
                    let to = constraint_to_vertex_number(constraint).to_string();
                    let from = constraint.get_signal_number().to_string();
                    let e = digraph.edge(from.as_str(), to.as_str());
                    negate_and_direct(constraint.is_negated(), e);
                }
            }
        }
        String::from_utf8(output_bytes).unwrap()
    }

    /// Writes DOT file of the circuit to the given path.
    /// If the path does not exist, it will be created.
    /// If the path exists, it will be overwritten.
    pub fn write_dot(&self, path: &Path) -> std::io::Result<()> {
        let data = self.get_dot();
        let parent = if let Some(p) = path.parent() {
            p
        } else {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Invalid path",
            ));
        };
        if !path.exists() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(path, data)
    }
}
