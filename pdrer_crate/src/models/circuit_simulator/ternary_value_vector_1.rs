// ************************************************************************************************
// use
// ************************************************************************************************

// ************************************************************************************************
// types
// ************************************************************************************************

use crate::models::TernaryValue;

type Element = usize;

#[derive(Clone, Debug)]
pub struct TernaryValueVector {
    vector: Vec<Element>,
}

// ************************************************************************************************
// constants
// ************************************************************************************************

const ATOM_SIZE_IN_BITS: usize = 2;
const ATOM_SIZE_IN_BITS_LOG2: usize = ATOM_SIZE_IN_BITS.ilog2() as usize;

const ATOMS_IN_ELEMENT: usize = (Element::BITS as usize) / ATOM_SIZE_IN_BITS;
const ATOMS_IN_ELEMENT_LOG2: usize = ATOMS_IN_ELEMENT.ilog2() as usize;
const ATOMS_IN_ELEMENT_MINUS_ONE: usize = ATOMS_IN_ELEMENT - 1;

// ************************************************************************************************
// impl
// ************************************************************************************************

impl TernaryValueVector {
    fn get_mask_location(index: usize) -> usize {
        (index & ATOMS_IN_ELEMENT_MINUS_ONE) << ATOM_SIZE_IN_BITS_LOG2
    }

    fn index_to_internal_index(index: usize) -> usize {
        index >> ATOMS_IN_ELEMENT_LOG2
    }

    fn ternary_value_to_element(value: TernaryValue) -> Element {
        value as Element
    }

    fn element_to_ternary_value(element: Element) -> TernaryValue {
        // unsafe { std::mem::transmute(element as u8) }
        match element {
            0b00 => TernaryValue::X,
            0b01 => TernaryValue::False,
            0b10 => TernaryValue::True,
            _ => panic!("Invalid element"),
        }
    }

    // ********************************************************************************************
    // API
    // ********************************************************************************************

    pub fn new(length: usize) -> Self {
        let len = (length >> ATOMS_IN_ELEMENT_LOG2) + 1;
        Self {
            vector: vec![0; len],
        }
    }

    pub fn set(&mut self, index: usize, value: TernaryValue) {
        let i = Self::index_to_internal_index(index);
        let m = Self::get_mask_location(index);
        let e = Self::ternary_value_to_element(value);
        self.vector[i] = self.vector[i] & !(0b11 << m) | (e << m);
    }

    pub fn get(&self, index: usize) -> TernaryValue {
        let i = Self::index_to_internal_index(index);
        let m = Self::get_mask_location(index);
        Self::element_to_ternary_value((self.vector[i] >> m) & 0b11)
    }

    pub fn clear(&mut self) {
        for x in self.vector.iter_mut() {
            *x = 0;
        }
    }

    pub fn clone_from_slice(&mut self, other: &Self) {
        self.vector.clone_from_slice(&other.vector);
    }
}
