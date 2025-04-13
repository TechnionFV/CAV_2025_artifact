// ************************************************************************************************
// use
// ************************************************************************************************

use std::time::Instant;

use crate::models::{
    circuit::CircuitSimplifier, signal_tracker::SignalTransformation, Circuit, Signal,
    SignalTracker, UniqueSortedVec,
};

use super::{
    detect_generic_patterns::CircuitGenericPatternDetector, CircuitAndGateMerger, CircuitCondenser,
    CircuitStructuralHashing, CircuitTechnologyMapper, CircuitUnusedSignalRemover,
};

// ************************************************************************************************
// simplifier
// ************************************************************************************************

// type CircuitSimplifier = fn;

// ************************************************************************************************
// impl
// ************************************************************************************************

impl Circuit {
    // ********************************************************************************************
    // helper functions
    // ********************************************************************************************

    fn perform_simplification<F: CircuitSimplifier>(
        &mut self,
        verbose: bool,
        tracker: &mut SignalTracker,
        mut f: F,
    ) {
        let timer = Instant::now();
        let message = f.title();
        let t = f.simplify(self);
        tracker.push(t);
        if verbose {
            println!(
                "Elapsed time = {:.3} sec, size = {}, {}.",
                timer.elapsed().as_secs_f32(),
                self.get_number_of_nodes(),
                message
            )
        };
    }

    // ********************************************************************************************
    // API
    // ********************************************************************************************

    // pub fn can_ground_be_removed(&self) -> bool {
    //     // remove ground if it is not used
    //     self.greatest_signal != Signal::GROUND
    //         && self.nodes.contains_key(&Signal::GROUND)
    //         && !self.important_signals.contains(&Signal::GROUND)
    //         && self.nodes.get(&Signal::GROUND).unwrap().users.is_empty()
    // }

    pub fn recalculate_important_signals(&self) -> UniqueSortedVec<Signal> {
        let mut important = self.get_bad_wires().peek().to_vec();
        important.append(&mut self.get_invariant_constraint_wires().peek().to_vec());
        important.append(&mut self.get_output_wires().peek().to_vec());
        important.append(&mut self.get_wires_that_feed_into_latches());
        UniqueSortedVec::from_sequence(important.iter().map(|w| w.signal()).collect())
    }

    pub fn simplify_circuit_before_using_proof_engine(&mut self, verbose: bool) -> SignalTracker {
        if verbose {
            println!(
                "Size = {}, Simplifying circuit before using proof engine:",
                self.get_number_of_nodes()
            )
        }
        let mut map = SignalTracker::new();
        self.perform_simplification(verbose, &mut map, CircuitUnusedSignalRemover::new());
        self.perform_simplification(verbose, &mut map, CircuitStructuralHashing::new());
        self.perform_simplification(verbose, &mut map, CircuitUnusedSignalRemover::new());
        self.perform_simplification(verbose, &mut map, CircuitGenericPatternDetector::new(false));
        self.perform_simplification(verbose, &mut map, CircuitUnusedSignalRemover::new());
        self.perform_simplification(verbose, &mut map, CircuitAndGateMerger::new(false));
        // self.perform_simplification(verbose, &mut map, CircuitTechnologyMapper::new(4, 10));
        self.perform_simplification(verbose, &mut map, CircuitCondenser::new());

        map
    }

    pub fn remove_unused_signals(&mut self) -> SignalTransformation {
        let mut s = CircuitUnusedSignalRemover::new();
        s.simplify(self)
    }

    pub fn merge_and_gates(&mut self) -> SignalTransformation {
        let mut s = CircuitAndGateMerger::new(false);
        s.simplify(self)
    }

    pub fn condense(&mut self) -> SignalTransformation {
        let mut s = CircuitCondenser::new();
        s.simplify(self)
    }

    pub fn detect_generic_patterns(&mut self) -> SignalTransformation {
        let mut s = CircuitGenericPatternDetector::new(false);
        s.simplify(self)
    }

    pub fn structural_hash(&mut self) -> SignalTransformation {
        let mut s = CircuitStructuralHashing::new();
        s.simplify(self)
    }

    pub fn default_technology_mapping(&mut self) -> SignalTransformation {
        let mut s = CircuitTechnologyMapper::new(4, 10);
        s.simplify(self)
    }
}
