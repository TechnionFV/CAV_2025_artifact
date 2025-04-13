use std::fmt::Display;

use crate::{
    formulas::{Literal, Variable},
    models::{vec3d::Vec3d, SortedVecOfLiterals, Vec2d},
};

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
struct MappingObjects {
    mappings: Vec3d<Option<Literal>>,
    mapping_counts: Vec2d<usize>,
    mapping_units: Vec2d<Option<Literal>>,
}

/// This struct solves the problem of mapping literals in the pattern to literals in the original
/// CNF. The struct expects a set of literals `{a, b, c, ...}`, where each literal is unique.
/// It also expects a set of inequalities between the literal variables (`a.var != b.var`).
/// And finally the struct expects a set of rules where a rule is an indication that a literal
/// belongs in a certain set:
///
/// For example:
/// 1. a is in {1, -2, 3}
/// 2. b is in {1, -2, 3}
/// 3. c is in {1, -2, 3}
/// 4. a.var != b.var
/// 5. b.var != c.var
/// 6. a.var != c.var
///
/// Is solvable by the following mapping: a -> 1, b -> -2, c -> 3
///
/// The following rules are not solvable:
/// 1. a is in {1, 2}
/// 2. a is in {1, 3}
/// 3. a is in {2, 3}
///
/// Furthermore, solving is done incrementally, where a `block` of rules are given
/// and can be undone. For example:
///
/// 1. a is in {1, 2, 3}
/// 2. b is in {1, 2, 3}
/// 3. c is in {1, 2, 3}
/// 4. a is in {1, 2}
///
///
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct VariableMapping {
    vars_to_be_mapped: Vec<Variable>,
    max_block_count: usize,
    max_set_length: usize,
    neq: Vec<(usize, usize)>,
    m: MappingObjects,
    stack_pointer: usize,
}

impl VariableMapping {
    pub fn new(
        vars_to_be_mapped: Vec<Variable>,
        max_block_count: usize,
        max_set_length: usize,
        neq: Vec<(Variable, Variable)>,
    ) -> Self {
        let mappings = Vec3d::new(
            max_block_count,
            vars_to_be_mapped.len(),
            max_set_length,
            None,
        );
        let mapping_counts = Vec2d::new(max_block_count, vars_to_be_mapped.len(), 0);
        let mapping_units = Vec2d::new(max_block_count, vars_to_be_mapped.len(), None);
        let neq = neq
            .iter()
            .map(|(a, b)| {
                (
                    vars_to_be_mapped.iter().position(|x| x == a).unwrap(),
                    vars_to_be_mapped.iter().position(|x| x == b).unwrap(),
                )
            })
            .collect();
        Self {
            vars_to_be_mapped,
            max_block_count,
            max_set_length,
            neq,
            m: MappingObjects {
                mappings,
                mapping_counts,
                mapping_units,
            },
            stack_pointer: 0,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.stack_pointer == 0
    }

    fn mark_new_unit(
        m: &mut MappingObjects,
        vars_to_be_mapped: &[Variable],
        x: usize,
        y: usize,
    ) -> Result<(), ()> {
        let map = m
            .mappings
            .get_row(x, y)
            .iter()
            .find(|x| x.is_some())
            .unwrap()
            .unwrap();

        m.mapping_units.set(x, y, Some(map));

        for y2 in 0..vars_to_be_mapped.len() {
            let mut some_count = Self::get_mapping_counts(&m.mapping_counts, x, y2);
            if some_count <= 1 {
                // already unit or uninitialized
                continue;
            }

            let row = m.mappings.get_row_mut(x, y2);
            for e in row.iter_mut() {
                if let Some(z) = e {
                    if *z == map {
                        Self::decrement_mapping_counts(&mut m.mapping_counts, x, y2);
                        some_count -= 1;
                        *e = None;
                        break; // the literals are unique
                    }
                }
            }

            if some_count == 0 {
                return Err(());
            }
            if some_count == 1 {
                Self::mark_new_unit(m, vars_to_be_mapped, x, y2)?;
            }
        }

        Ok(())
    }

    fn is_already_in_unit(
        m: &MappingObjects,
        // vars_to_be_mapped: &[Variable],
        x: usize,
        l: Literal,
    ) -> bool {
        m.mapping_units.get_row(x).contains(&Some(l))
        // for y in 0..vars_to_be_mapped.len() {
        // if let Some(u) = .get(x, y) {
        //     if *u == l {
        //         return true;
        //     }
        // }
        // }
        // false
    }

    fn increment_mapping_counts(mapping_counts: &mut Vec2d<usize>, x: usize, y: usize) {
        mapping_counts.set(x, y, mapping_counts.get(x, y) + 1);
    }

    fn decrement_mapping_counts(mapping_counts: &mut Vec2d<usize>, x: usize, y: usize) {
        mapping_counts.set(x, y, mapping_counts.get(x, y) - 1);
    }

    fn get_mapping_counts(mapping_counts: &Vec2d<usize>, x: usize, y: usize) -> usize {
        *mapping_counts.get(x, y)
    }

    fn indicate_var_in_set(
        m: &mut MappingObjects,
        vars_to_be_mapped: &[Variable],
        x: usize,
        var: Variable,
        set: &[Literal],
        negated: bool,
    ) -> Result<(), ()> {
        let y = vars_to_be_mapped.iter().position(|x| *x == var).unwrap();

        let is_first_indication = Self::get_mapping_counts(&m.mapping_counts, x, y) == 0;

        if is_first_indication {
            // todo set should remove units first.
            for (z, l) in set.iter().enumerate() {
                if Self::is_already_in_unit(m, x, *l) {
                    continue;
                }

                Self::increment_mapping_counts(&mut m.mapping_counts, x, y);
                m.mappings
                    .set(x, y, z, Some(if negated { !*l } else { *l }));
            }
        } else {
            let row = m.mappings.get_row_mut(x, y);
            for e in row.iter_mut() {
                if let Some(z) = e {
                    let exists = if negated {
                        set.contains(&!*z)
                    } else {
                        set.contains(z)
                    };
                    if !exists {
                        Self::decrement_mapping_counts(&mut m.mapping_counts, x, y);
                        *e = None;
                    }
                }
            }
        }

        // if entire row is None now then there is no way to map this variable
        let non_null = Self::get_mapping_counts(&m.mapping_counts, x, y);
        if non_null == 0 {
            return Err(());
        }

        // now must propagate units.
        if non_null == 1 {
            Self::mark_new_unit(m, vars_to_be_mapped, x, y)?;
        }

        Ok(())
    }

    /// adds to the variable mapping the variables that we now know the mapping of.
    /// Returns false if the mapping is not possible (contradiction to the pattern found).
    /// Returns true if the mapping is possible.
    pub fn update_variable_mapping(
        &mut self,
        real_diff_i: &SortedVecOfLiterals,
        real_diff_j: &SortedVecOfLiterals,
        patter_diff_i: &SortedVecOfLiterals,
        patter_diff_j: &SortedVecOfLiterals,
    ) -> bool {
        debug_assert!(self.stack_pointer < self.max_block_count);
        if self.stack_pointer == 0 {
            for i in self.m.mappings.get_matrix_mut(self.stack_pointer) {
                *i = None;
            }
            for i in self.m.mapping_counts.get_row_mut(self.stack_pointer) {
                *i = 0;
            }
            for i in self.m.mapping_units.get_row_mut(self.stack_pointer) {
                *i = None;
            }
        } else {
            self.m
                .mappings
                .copy_matrix(self.stack_pointer - 1, self.stack_pointer);
            self.m
                .mapping_counts
                .copy_row(self.stack_pointer - 1, self.stack_pointer);
            self.m
                .mapping_units
                .copy_row(self.stack_pointer - 1, self.stack_pointer);
        }

        for (pattern, real) in [(patter_diff_i, real_diff_i), (patter_diff_j, real_diff_j)].iter() {
            for l in pattern.iter() {
                let v = l.variable();
                let set = real.peek().peek();
                let negated = l.is_negated();
                if let Err(()) = Self::indicate_var_in_set(
                    &mut self.m,
                    &self.vars_to_be_mapped,
                    self.stack_pointer,
                    v,
                    set,
                    negated,
                ) {
                    return false;
                }
                // let y = self.vars_to_be_mapped.iter().position(|x| *x == v).unwrap();
                // let row = self.mappings.get_row(self.stack_pointer, y);
                // println!("Y ({}): ", v);
                // for e in row.iter() {
                //     print!(
                //         "{}, ",
                //         match e {
                //             Some(l) => l.to_string(),
                //             None => "None".to_string(),
                //         }
                //     );
                // }
                // println!();
            }
        }

        for (yi, yj) in self.neq.iter() {
            if let (Some(a), Some(b)) = (
                self.m.mapping_units.get(self.stack_pointer, *yi),
                self.m.mapping_units.get(self.stack_pointer, *yj),
            ) {
                if a.variable() == b.variable() {
                    return false;
                }
            }
        }

        self.stack_pointer += 1;
        true
    }

    pub fn undo(&mut self) {
        debug_assert!(self.stack_pointer > 0);
        self.stack_pointer -= 1;
    }

    // fn _get_mapping_debug(&self) -> Vec<(Variable, Option<SortedVecOfLiterals>)> {
    //     let x = self.mappings.last().as_ref().unwrap().iter().cloned();
    //     self.vars_to_be_mapped.iter().copied().zip(x).collect()
    // }

    pub fn get_mapping(&mut self) -> Vec<(Variable, Literal)> {
        debug_assert!(self.stack_pointer > 0);
        let x = self.stack_pointer - 1;
        let mut r = Vec::with_capacity(self.vars_to_be_mapped.len());
        for (y, v) in self.vars_to_be_mapped.iter().enumerate() {
            let some_count = Self::get_mapping_counts(&self.m.mapping_counts, x, y);
            let first_literal = self
                .m
                .mappings
                .get_row(x, y)
                .iter()
                .find(|z| z.is_some())
                .unwrap()
                .unwrap();
            r.push((*v, first_literal));
            if some_count != 1 {
                // remove literal from all other future variables
                for y2 in y + 1..self.vars_to_be_mapped.len() {
                    let row = self.m.mappings.get_row_mut(x, y2);
                    for e in row.iter_mut() {
                        if let Some(z) = e {
                            if *z == first_literal {
                                *e = None;
                            }
                        }
                    }
                }
            }
        }
        r
    }
}

impl Display for VariableMapping {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for x in 0..self.stack_pointer {
            writeln!(f, "X {}", x)?;
            for (y, v) in self.vars_to_be_mapped.iter().enumerate() {
                let row = self.m.mappings.get_row(x, y);
                write!(f, "Y ({}): ", v)?;
                for e in row.iter() {
                    write!(
                        f,
                        "{}, ",
                        match e {
                            Some(l) => l.to_string(),
                            None => "None".to_string(),
                        }
                    )?;
                }
                writeln!(f)?;
            }
        }
        Ok(())
    }
}
