// ************************************************************************************************
// use
// ************************************************************************************************

use crate::models::PrettyTable;
use std::fmt;

// ************************************************************************************************
// struct
// ************************************************************************************************

#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct Vec2d<T: Clone> {
    len_x: usize,
    len_y: usize,
    vec: Vec<T>,
}

// ************************************************************************************************
// impl
// ************************************************************************************************

impl<T: Clone> Vec2d<T> {
    pub fn new(len_x: usize, len_y: usize, default: T) -> Self {
        Self {
            len_x,
            len_y,
            vec: vec![default; len_x * len_y],
        }
    }

    fn check_x(&self, x: usize) {
        debug_assert!(x < self.len_x);
    }

    fn check_y(&self, y: usize) {
        debug_assert!(y < self.len_y);
    }

    pub fn get(&self, x: usize, y: usize) -> &T {
        self.check_x(x);
        self.check_y(y);
        &self.vec[x * self.len_y + y]
    }

    pub fn get_mut(&mut self, x: usize, y: usize) -> &mut T {
        self.check_x(x);
        self.check_y(y);
        &mut self.vec[x * self.len_y + y]
    }

    pub fn set(&mut self, x: usize, y: usize, value: T) {
        self.check_x(x);
        self.check_y(y);
        self.vec[x * self.len_y + y] = value;
    }

    pub fn get_row(&self, x: usize) -> &[T] {
        self.check_x(x);
        &self.vec[x * self.len_y..(x + 1) * self.len_y]
    }

    pub fn get_row_mut(&mut self, x: usize) -> &mut [T] {
        self.check_x(x);
        &mut self.vec[x * self.len_y..(x + 1) * self.len_y]
    }
}

// ************************************************************************************************
// Copy from other
// ************************************************************************************************

impl<T: Clone + Copy> Vec2d<T> {
    pub fn copy_row(&mut self, x_src: usize, x_dest: usize) {
        self.check_x(x_src);
        self.check_x(x_dest);
        if x_src == x_dest {
            return;
        }

        let dest_start = x_dest * self.len_y;

        let (before_dest, dest_and_after) = self.vec.split_at_mut(dest_start);
        let (dest_slice, after_dest) = dest_and_after.split_at_mut(self.len_y);

        if x_src < x_dest {
            let i = x_src * self.len_y;
            dest_slice.copy_from_slice(&before_dest[i..i + self.len_y]);
        } else {
            let i = (x_src - (x_dest - 1)) * self.len_y;
            dest_slice.copy_from_slice(&after_dest[i..i + self.len_y]);
        }
    }
}

// ************************************************************************************************
// fmt
// ************************************************************************************************

impl<T: Clone + fmt::Display> fmt::Display for Vec2d<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut table = PrettyTable::new((0..self.len_y).map(|i| i.to_string()).collect());
        for x in 0..self.len_x {
            table
                .add_row(
                    (0..self.len_y)
                        .map(|y| self.get(x, y).to_string())
                        .collect(),
                )
                .unwrap();
        }
        write!(f, "{}", table)
    }
}
