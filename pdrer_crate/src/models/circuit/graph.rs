// // ************************************************************************************************
// // use
// // ************************************************************************************************

// use core::fmt;
// use std::collections::{FxHashMap, FxHashSet};

// use crate::models::TruthTable;
// // use crate::models::and_inverter_graph::aig_node::AIGNodeType;
// // use std::collections::FxHashSet;
// use crate::models::ternary_value::TernaryValue;
// use crate::models::Signal;
// use crate::models::Wire;
// use petgraph::graph::Graph;

// use super::Circuit;

// // ************************************************************************************************
// // enum
// // ************************************************************************************************

// #[derive(PartialEq, Eq, Debug, Clone)]
// pub enum NodeType {
//     ConstantZero,
//     Input {
//         signal: Signal,
//     },
//     Latch {
//         initial: TernaryValue,
//         signal: Signal,
//     },
//     And {
//         signal: Signal,
//     },
//     Generic {
//         signal: Signal,
//         table: TruthTable,
//     },
// }

// impl fmt::Display for NodeType {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         match self {
//             NodeType::ConstantZero => write!(f, "Const S=0"),
//             NodeType::Input { signal } => write!(f, "I S={}", signal.get_number()),
//             NodeType::Latch { initial, signal } => {
//                 write!(f, "L({}) S={}", initial, signal.get_number())
//             }
//             NodeType::And { signal } => write!(f, "A S={}", signal.get_number()),
//             NodeType::Generic { signal, table } => {
//                 write!(f, "G S={} T={}", signal.get_number(), table)
//             }
//         }
//     }
// }

// #[derive(PartialEq, Eq, Debug, Clone, Copy)]
// pub enum EdgeType {
//     NonNegated,
//     Negated,
// }

// impl fmt::Display for EdgeType {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         match self {
//             EdgeType::NonNegated => write!(f, ""),
//             EdgeType::Negated => write!(f, "*"),
//         }
//     }
// }

// // ************************************************************************************************
// // impl
// // ************************************************************************************************

// impl Circuit {
//     // ********************************************************************************************
//     // helper
//     // ********************************************************************************************

//     fn get_edge_weight(&self, is_negated: bool) -> EdgeType {
//         if is_negated {
//             EdgeType::Negated
//         } else {
//             EdgeType::NonNegated
//         }
//     }

//     // ********************************************************************************************
//     // API
//     // ********************************************************************************************

//     pub fn to_graph(&self, wires_to_trace: Option<&[Wire]>) -> Graph<NodeType, EdgeType> {
//         let mut g: Graph<NodeType, EdgeType> = Graph::new();
//         let mut signal_to_index = FxHashMap::new();

//         // record important signals
//         let important = match wires_to_trace {
//             Some(v) => v.to_owned(),
//             None => {
//                 // start counting the important literals
//                 let mut important = self.get_bad_wires();
//                 important.append(&mut self.get_constraints_wires());
//                 important.append(&mut self.get_output_wires());
//                 self.get_latch_information()
//                     .iter()
//                     .for_each(|latch| match latch.node_type {
//                         super::node_types::CircuitNodeType::Latch { input, initial } => {
//                             important.push(input)
//                         }
//                         _ => unreachable!(),
//                     });
//                 important.sort_unstable();
//                 important.dedup();
//                 important
//             }
//         };

//         // get cone of influence
//         let mut gates = self.get_cone_of_influence(&important);
//         gates.sort_unstable_by_key(|a| a.id);

//         // signals in cone
//         let signals_in_coi: FxHashSet<Signal> = gates.iter().map(|w| w.id).collect();

//         // add constant 0
//         if signals_in_coi.contains(&Signal::new(0)) {
//             let index = g.add_node(NodeType::ConstantZero);
//             signal_to_index.insert(Signal::new(0), index);
//         }

//         // add all inputs in cone of influence
//         for i in self
//             .get_input_signals()
//             .into_iter()
//             .filter(|i| signals_in_coi.contains(i))
//         {
//             let index = g.add_node(NodeType::Input { signal: i });
//             signal_to_index.insert(i, index);
//         }

//         // add all latches in cone of influence
//         for latch in self
//             .get_latch_information()
//             .into_iter()
//             .filter(|i| signals_in_coi.contains(&i.id))
//         {
//             let index = g.add_node(NodeType::Latch {
//                 initial: latch.initial,
//                 signal: latch.output,
//             });
//             signal_to_index.insert(latch.output, index);
//         }

//         // add the and gates in the cone to the graph
//         for and in gates {
//             let index = g.add_node(NodeType::And { signal: and.out });
//             signal_to_index.insert(and.out, index);
//             {
//                 let from1 = signal_to_index.get(&and.in0.get_signal()).unwrap();
//                 g.add_edge(*from1, index, self.get_edge_weight(and.in0.is_negated()));
//             }
//             {
//                 let from2 = signal_to_index.get(&and.in1.get_signal()).unwrap();
//                 g.add_edge(*from2, index, self.get_edge_weight(and.in1.is_negated()));
//             }
//         }

//         // add the edges to the latches for wires that exist
//         for latch in self
//             .get_latch_information()
//             .into_iter()
//             .filter(|i| signals_in_coi.contains(&i.output))
//         {
//             // latch input might have not been in COI
//             let from = signal_to_index.get(&latch.input.get_signal());
//             match from {
//                 Some(fromm) => {
//                     // latch has definitely been inserted
//                     let to = signal_to_index.get(&latch.output).unwrap();
//                     g.add_edge(*fromm, *to, self.get_edge_weight(latch.input.is_negated()));
//                 }
//                 None => {
//                     // do nothing (don't add) edge
//                     // this should never happen if wires_to_trace is None
//                     debug_assert_ne!(wires_to_trace, None);
//                 }
//             }
//         }

//         g
//     }
// }
