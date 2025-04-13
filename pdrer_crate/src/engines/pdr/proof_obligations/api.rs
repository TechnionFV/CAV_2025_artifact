// ************************************************************************************************
// use
// ************************************************************************************************

use super::queue::{ProofObligationsQueue, ProofObligationsQueueEntry};
use super::trace_tree::TraceTree;
use super::{ProofObligation, ProofObligations};
use crate::formulas::{Clause, Cube};
use crate::models::Counterexample;

// ************************************************************************************************
// impl
// ************************************************************************************************

impl ProofObligations {
    // ********************************************************************************************
    // helper functions
    // ********************************************************************************************

    // ********************************************************************************************
    // construction API
    // ********************************************************************************************

    pub fn new() -> Self {
        Self {
            queue: ProofObligationsQueue::new(),
            trace_tree: TraceTree::new(),
        }
    }

    pub fn push(
        &mut self,
        is_initial: bool,
        cube: Cube,
        // obligation: ProofObligation,
        frame: usize,
        total_frame_size: usize,
        input: Cube,
        successor: Option<Cube>,
    ) -> Result<(), Counterexample> {
        let index = self.trace_tree.push(is_initial, cube, input, successor)?;
        self.queue.push(ProofObligationsQueueEntry {
            cube_index: index,
            frame,
            hash_when_added: total_frame_size,
        });
        Ok(())
    }

    pub fn re_push(&mut self, obligation: ProofObligation) {
        let index = self.trace_tree.get_index(&obligation.cube).unwrap();
        self.queue.push(ProofObligationsQueueEntry {
            cube_index: index,
            frame: obligation.frame,
            hash_when_added: obligation.hash_when_added,
        });
    }

    pub fn len(&self) -> usize {
        debug_assert!(self.queue.len() <= self.trace_tree.len());
        self.queue.len()
    }

    pub fn pop(&mut self) -> Option<ProofObligation> {
        self.queue.pop().map(|ipo| ProofObligation {
            cube: self
                .trace_tree
                .get_state(ipo.cube_index)
                .unwrap()
                .to_owned(),
            frame: ipo.frame,
            hash_when_added: ipo.hash_when_added,
        })
    }

    // pub fn peek(&mut self) -> Option<ProofObligation> {
    //     self.queue
    //         .peek()
    //         .map(|(cube_index, frame)| ProofObligation {
    //             cube: self.trace_tree.get_state(cube_index).unwrap().to_owned(),
    //             frame,
    //         })
    // }

    pub fn is_empty(&self) -> bool {
        debug_assert!(self.queue.len() <= self.trace_tree.len());
        self.queue.is_empty()
    }

    pub fn gc(&mut self) {
        if self.queue.is_empty() {
            self.trace_tree.gc();
        }
    }

    /// Takes the stack head, and takes n - 1 successors from the stack head.
    /// Returns the negations of those cubes.
    pub fn get_last_n_cubes_from_stack_head(&self, n: usize) -> Vec<Clause> {
        let x = self.trace_tree.get_last_n_cubes_from_stack_head(n);
        x.into_iter().map(|x| !x).collect()
    }

    pub fn get_trace_tree(&self) -> &TraceTree {
        &self.trace_tree
    }
}

// ************************************************************************************************
// impl
// ************************************************************************************************

impl Default for ProofObligations {
    fn default() -> Self {
        Self::new()
    }
}
