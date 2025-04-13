// ************************************************************************************************
// use
// ************************************************************************************************

use crate::models::PrettyTable;
use std::fmt;

// ************************************************************************************************
// struct
// ************************************************************************************************

#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct Vec3d<T: Clone> {
    len_x: usize,
    len_y: usize,
    len_z: usize,
    len_yz: usize,
    vec: Vec<T>,
}

// ************************************************************************************************
// impl
// ************************************************************************************************

impl<T: Clone> Vec3d<T> {
    pub fn new(len_x: usize, len_y: usize, len_z: usize, default: T) -> Self {
        Self {
            len_x,
            len_y,
            len_z,
            len_yz: len_y * len_z,
            vec: vec![default; len_x * len_y * len_z],
        }
    }

    fn check_x(&self, x: usize) {
        debug_assert!(x < self.len_x);
    }

    fn check_y(&self, y: usize) {
        debug_assert!(y < self.len_y);
    }

    fn check_z(&self, z: usize) {
        debug_assert!(z < self.len_z);
    }

    pub fn get(&self, x: usize, y: usize, z: usize) -> &T {
        self.check_x(x);
        self.check_y(y);
        self.check_z(z);
        &self.vec[x * self.len_yz + y * self.len_z + z]
    }

    pub fn get_mut(&mut self, x: usize, y: usize, z: usize) -> &mut T {
        self.check_x(x);
        self.check_y(y);
        self.check_z(z);
        &mut self.vec[x * self.len_yz + y * self.len_z + z]
    }

    pub fn set(&mut self, x: usize, y: usize, z: usize, value: T) {
        self.check_x(x);
        self.check_y(y);
        self.check_z(z);
        self.vec[x * self.len_yz + y * self.len_z + z] = value;
    }

    pub fn get_row(&self, x: usize, y: usize) -> &[T] {
        self.check_x(x);
        self.check_y(y);
        let from = x * self.len_yz + y * self.len_z;
        let to = from + self.len_z;
        &self.vec[from..to]
    }

    pub fn get_row_mut(&mut self, x: usize, y: usize) -> &mut [T] {
        self.check_x(x);
        self.check_y(y);
        let from: usize = x * self.len_yz + y * self.len_z;
        let to = from + self.len_z;
        &mut self.vec[from..to]
    }

    pub fn get_matrix(&self, x: usize) -> &[T] {
        self.check_x(x);
        let from = x * self.len_yz;
        let to = from + self.len_yz;
        &self.vec[from..to]
    }

    pub fn get_matrix_mut(&mut self, x: usize) -> &mut [T] {
        self.check_x(x);
        let from = x * self.len_yz;
        let to = from + self.len_yz;
        &mut self.vec[from..to]
    }
}

// ************************************************************************************************
// Copy from other
// ************************************************************************************************

impl<T: Clone + Copy> Vec3d<T> {
    pub fn copy_matrix(&mut self, x_src: usize, x_dest: usize) {
        self.check_x(x_src);
        self.check_x(x_dest);
        if x_src == x_dest {
            return;
        }
        let dest_start = x_dest * self.len_yz;

        let (before_dest, dest_and_after) = self.vec.split_at_mut(dest_start);
        let (dest_slice, after_dest) = dest_and_after.split_at_mut(self.len_yz);

        if x_src < x_dest {
            let i = x_src * self.len_yz;
            dest_slice.copy_from_slice(&before_dest[i..i + self.len_yz]);
        } else {
            let i = (x_src - (x_dest - 1)) * self.len_yz;
            dest_slice.copy_from_slice(&after_dest[i..i + self.len_yz]);
        }
    }
}

// ************************************************************************************************
// fmt
// ************************************************************************************************

impl<T: Clone + fmt::Display> fmt::Display for Vec3d<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for x in 0..self.len_x {
            let mut table = PrettyTable::new((0..self.len_z).map(|i| i.to_string()).collect());
            for y in 0..self.len_y {
                table
                    .add_row(
                        (0..self.len_z)
                            .map(|z: usize| self.get(x, y, z).to_string())
                            .collect(),
                    )
                    .unwrap();
            }
            write!(f, "{}", table)?;
        }

        Ok(())
    }
}
