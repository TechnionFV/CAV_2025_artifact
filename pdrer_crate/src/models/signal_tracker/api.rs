// ************************************************************************************************
// use
// ************************************************************************************************

use crate::models::{Signal, Wire};

use super::{SignalTracker, SignalTransformation};

// ************************************************************************************************
// impl
// ************************************************************************************************

impl SignalTracker {
    // ********************************************************************************************
    // helper functions
    // ********************************************************************************************

    // fn insert(&mut self, from: Signal, to: Option<Wire>) -> Option<Wire> {
    //     match to {
    //         Some(x) => self.forward.insert(from, x),
    //         None => self.forward.remove(&from),
    //     }
    // }

    // ********************************************************************************************
    // API
    // ********************************************************************************************

    // pub fn mark_signal_removal(&mut self, signal: Signal) -> Option<Wire> {
    //     self.insert(signal, None)
    // }

    // pub fn mark_signal_change(&mut self, from: Signal, to: Wire) -> Option<Wire> {
    //     self.insert(from, Some(to))
    // }

    // // pub fn mark_signal_as_equivalent_to_wire(&mut self, signal: Signal, wire: Wire) {
    // //     self.equivalent.insert(signal, wire);
    // //     self.mark_signal_removal(signal);
    // // }

    // pub fn get(&self, from: &Signal) -> Option<Wire> {
    //     self.forward.get(from).copied()
    // }

    // pub fn is_identity(&self) -> bool {
    //     self.forward.iter_pairs().all(|(s, w)| s == w.signal())
    // }

    // pub fn contains(&self, from: &Signal) -> bool {
    //     self.forward.contains_key(from)
    // }

    // pub fn iter_pairs(&self) -> impl DoubleEndedIterator<Item = (Signal, &Wire)> + '_ {
    //     self.forward.iter_pairs()
    // }

    // pub fn get_equivalences(&self) -> Vec<Vec<Wire>> {
    //     let mut map = FxHashMap::new();
    //     for (from, to) in self.iter_pairs() {
    //         if let Some(to) = to {}
    //     }
    // }

    pub fn push(&mut self, transformation: SignalTransformation) {
        self.transformations.push(transformation);
    }

    pub fn len(&self) -> usize {
        self.transformations.len()
    }

    pub fn is_empty(&self) -> bool {
        self.transformations.is_empty()
    }

    pub fn get(&self, mut original: Signal) -> Option<Signal> {
        for t in self.transformations.iter() {
            match t {
                SignalTransformation::SignalReorder(r) => {
                    let x = r.get(&original).unwrap();
                    original = *x;
                }
                SignalTransformation::SignalsRemovedBecauseTheyAreNotUsed(r) => {
                    if r.contains(&original) {
                        return None;
                    }
                }
                SignalTransformation::SignalsRemovedBecauseOfEquivalentWires(r) => {
                    if r.contains_key(&original) {
                        return None;
                    }
                }
            }
        }
        Some(original)
    }

    pub fn backward(&self, mut final_signal: Signal, i: usize) -> Option<Signal> {
        for t in self.transformations.iter().take(i).rev() {
            if let SignalTransformation::SignalReorder(r) = t {
                for (k, v) in r.iter_pairs() {
                    if *v == final_signal {
                        final_signal = k;
                        break;
                    }
                }
                // unreachable!();
            }
        }
        Some(final_signal)
    }

    pub fn find_equivalent_if_removed(&self, mut original: Signal) -> Option<Wire> {
        for (i, t) in self.transformations.iter().enumerate() {
            match t {
                SignalTransformation::SignalReorder(r) => {
                    let x = r.get(&original).unwrap();
                    original = *x;
                }
                SignalTransformation::SignalsRemovedBecauseTheyAreNotUsed(r) => {
                    if r.contains(&original) {
                        return None;
                    }
                }
                SignalTransformation::SignalsRemovedBecauseOfEquivalentWires(r) => {
                    if let Some(w) = r.get(&original) {
                        let b = self.backward(w.signal(), i).unwrap();
                        return Some(b.wire(w.is_negated()));
                    }
                }
            }
        }
        None
    }
}
