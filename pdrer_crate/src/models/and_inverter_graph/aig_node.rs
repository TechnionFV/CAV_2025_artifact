// ************************************************************************************************
// use
// ************************************************************************************************

use crate::models::Wire;

// ************************************************************************************************
// enum
// ************************************************************************************************

#[derive(PartialEq, Eq, Debug, Clone)]
pub(super) enum AIGNodeType {
    ConstantZero,
    Input,
    Latch {
        input: Wire,
        reset: Wire,
    },
    And {
        input0: Wire, /* as literal [0..2*maxvar+1] */
        input1: Wire, /* as literal [0..2*maxvar+1] */
    },
}

// ************************************************************************************************
// struct
// ************************************************************************************************

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AIGNode {
    // wire: Wire,
    node_type: AIGNodeType,
}

// ************************************************************************************************
// impl
// ************************************************************************************************

impl AIGNode {
    // ********************************************************************************************
    // creation
    // ********************************************************************************************

    pub(super) fn new(node_type: AIGNodeType) -> Self {
        Self {
            // wire: lit,
            node_type,
        }
    }

    // ********************************************************************************************
    // universal
    // ********************************************************************************************

    pub(super) fn get_type(&self) -> &AIGNodeType {
        &self.node_type
    }

    // pub fn get_wire(&self) -> Wire {
    //     self.wire
    // }

    // pub fn get_level(&self) -> usize {
    //     self.level
    // }

    // ********************************************************************************************
    // latch
    // ********************************************************************************************

    pub fn set_input_of_latch(&mut self, input: Wire) {
        if let AIGNodeType::Latch { input: _, reset } = self.node_type {
            self.node_type = AIGNodeType::Latch { input, reset };
        } else {
            panic!("Node is not a latch.");
        }
    }

    pub fn set_reset_of_latch(&mut self, reset: Wire) {
        if let AIGNodeType::Latch { input, reset: _ } = self.node_type {
            self.node_type = AIGNodeType::Latch { input, reset };
        } else {
            panic!("Node is not a latch.");
        }
    }

    pub fn get_latch_input(&self) -> Wire {
        if let AIGNodeType::Latch { input, reset: _ } = self.node_type {
            input
        } else {
            panic!("Node is not a latch.");
        }
    }

    pub fn get_latch_reset(&self) -> Wire {
        if let AIGNodeType::Latch { input: _, reset } = self.node_type {
            reset
        } else {
            panic!("Node is not a latch.");
        }
    }

    // ********************************************************************************************
    // and
    // ********************************************************************************************

    // pub fn set_rhs0_of_and(&mut self, rhs0: Wire) {
    //     if let AIGNodeType::And { input0: _, input1 } = self.node_type {
    //         self.node_type = AIGNodeType::And {
    //             input0: rhs0,
    //             input1,
    //         };
    //     } else {
    //         panic!("Node is not an and.");
    //     }
    // }

    // pub fn set_rhs1_of_and(&mut self, rhs1: Wire) {
    //     if let AIGNodeType::And { input0, input1: _ } = self.node_type {
    //         self.node_type = AIGNodeType::And {
    //             input0,
    //             input1: rhs1,
    //         };
    //     } else {
    //         panic!("Node is not an and.");
    //     }
    // }

    pub fn get_and_rhs0(&self) -> Wire {
        if let AIGNodeType::And { input0, input1: _ } = self.node_type {
            input0
        } else {
            panic!("Node is not an and.");
        }
    }

    pub fn get_and_rhs1(&self) -> Wire {
        if let AIGNodeType::And { input0: _, input1 } = self.node_type {
            input1
        } else {
            panic!("Node is not an and.");
        }
    }

    // ********************************************************************************************
    // symbols
    // ********************************************************************************************

    // pub fn set_input_symbol(&mut self, symbol: &str) {
    //     assert_eq!(self.node_type, AIGNodeType::Input);
    //     self.input_symbol = symbol.to_string();
    // }

    // pub fn get_input_symbol(&self) -> &str {
    //     assert_eq!(self.node_type, AIGNodeType::Input);
    //     self.input_symbol.as_str()
    // }

    // pub fn set_latch_symbol(&mut self, symbol: &str) {
    //     assert_eq!(self.node_type, AIGNodeType::Latch);
    //     self.latch_symbol = symbol.to_string();
    // }

    // pub fn get_latch_symbol(&self) -> &str {
    //     assert_eq!(self.node_type, AIGNodeType::Latch);
    //     self.latch_symbol.as_str()
    // }

    // pub fn set_output_symbol(&mut self, symbol: &str) {
    //     self.output_symbol = symbol.to_string();
    // }

    // pub fn get_output_symbol(&self) -> &str {
    //     self.output_symbol.as_str()
    // }

    // pub fn set_bad_symbol(&mut self, symbol: &str) {
    //     self.bad_symbol = symbol.to_string();
    // }

    // pub fn get_bad_symbol(&self) -> &str {
    //     self.bad_symbol.as_str()
    // }

    // pub fn set_constraint_symbol(&mut self, symbol: &str) {
    //     self.constraint_symbol = symbol.to_string();
    // }

    // pub fn get_constraint_symbol(&self) -> &str {
    //     self.constraint_symbol.as_str()
    // }
}
