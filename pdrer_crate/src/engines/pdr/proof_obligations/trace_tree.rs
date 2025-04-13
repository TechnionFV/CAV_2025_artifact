// ************************************************************************************************
// use
// ************************************************************************************************

use crate::formulas::Cube;
use crate::models::{Counterexample, UniqueSortedVec};
use fxhash::FxHashMap;

// ************************************************************************************************
// struct
// ************************************************************************************************

#[derive(Debug, Clone)]
struct TraceTreeNode {
    pub state: Cube,
    pub input: Cube,
    pub successor_index: Option<usize>,
}

pub struct TraceTree {
    trace_tree: Vec<TraceTreeNode>,
    index: FxHashMap<Cube, usize>,
}

// ************************************************************************************************
// impl
// ************************************************************************************************

impl Default for TraceTree {
    fn default() -> Self {
        Self::new()
    }
}

impl TraceTree {
    // ********************************************************************************************
    // helper functions
    // ********************************************************************************************

    // ********************************************************************************************
    // construction API
    // ********************************************************************************************

    pub fn new() -> Self {
        Self {
            trace_tree: Vec::new(),
            index: FxHashMap::default(),
        }
    }

    pub fn push(
        &mut self,
        is_initial: bool,
        state: Cube,
        input: Cube,
        successor: Option<Cube>,
    ) -> Result<usize, Counterexample> {
        if let Some(i) = self.get_index(&state) {
            // cube already exist, do nothing
            return Ok(i);
        }

        // cube not already exist
        let successor_index = successor.map(|x| self.index.get(&x).unwrap().to_owned());
        let this_index = self.trace_tree.len();
        self.trace_tree.push(TraceTreeNode {
            state: state.to_owned(),
            input,
            successor_index,
        });
        self.index.insert(state.to_owned(), this_index);

        if is_initial {
            return Err(self.extract_counter_example(state));
        }

        Ok(this_index)
    }

    pub fn get_index(&self, cube: &Cube) -> Option<usize> {
        self.index.get(cube).copied()
    }

    pub fn get_state(&self, index: usize) -> Option<&Cube> {
        self.trace_tree.get(index).map(|x| &x.state)
    }

    pub fn len(&self) -> usize {
        debug_assert_eq!(self.trace_tree.len(), self.index.len());
        self.trace_tree.len()
    }

    pub fn is_empty(&self) -> bool {
        debug_assert_eq!(self.trace_tree.is_empty(), self.index.is_empty());
        self.trace_tree.is_empty()
    }

    pub fn gc(&mut self) {
        self.trace_tree.clear();
        self.index.clear();
    }

    /// Takes the stack head, and takes n - 1 successors from the stack head.
    /// Returns the negations of those cubes.
    pub fn get_last_n_cubes_from_stack_head(&self, n: usize) -> Vec<Cube> {
        let mut cubes = vec![];
        let mut i = self.trace_tree.len() - 1;
        for _ in 0..n {
            let cube = self.trace_tree[i].state.to_owned();
            cubes.push(cube);
            match self.trace_tree[i].successor_index {
                Some(x) => i = x,
                None => break,
            }
        }

        cubes
    }

    pub fn extract_counter_example(&self, initial_cube: Cube) -> Counterexample {
        let mut current_cube = *self.index.get(&initial_cube).unwrap();
        let mut inputs = Vec::new();
        loop {
            let input = self.trace_tree[current_cube].input.to_owned();
            inputs.push(input.to_owned());
            let next_cube = self.trace_tree[current_cube].successor_index;
            match next_cube {
                Some(next_cube) => {
                    current_cube = next_cube;
                }
                None => break,
            }
        }

        Counterexample {
            initial_cube,
            inputs,
        }
    }

    pub fn get_all_leafs(&self) -> Vec<Cube> {
        let mut leafs = Vec::new();
        let used_indexes = self
            .trace_tree
            .iter()
            .filter_map(|x| x.successor_index)
            .collect::<Vec<_>>();
        let used_indexes = UniqueSortedVec::from_sequence(used_indexes);

        for (i, node) in self.trace_tree.iter().enumerate() {
            if used_indexes.contains(&i) {
                continue;
            }
            // never used thus leaf
            leafs.push(node.state.to_owned());
        }

        leafs
    }
}
