// ************************************************************************************************
// use
// ************************************************************************************************

use std::collections::BinaryHeap;

// ************************************************************************************************
// struct
// ************************************************************************************************

#[derive(Debug, Clone)]
struct QueueItem {
    pub frame: usize,
    pub priority: u32,
    pub cube_index: usize,
    pub hash_when_added: usize,
}

pub struct ProofObligationsQueueEntry {
    pub frame: usize,
    pub cube_index: usize,
    pub hash_when_added: usize,
}

pub struct ProofObligationsQueue {
    queue: BinaryHeap<QueueItem>,
    /// priority, smaller is better
    current_priority: u32,
}

// ************************************************************************************************
// impl
// ************************************************************************************************

impl Default for ProofObligationsQueue {
    fn default() -> Self {
        Self::new()
    }
}

impl ProofObligationsQueue {
    pub fn new() -> Self {
        Self {
            current_priority: 0,
            queue: BinaryHeap::new(),
        }
    }

    pub fn push(&mut self, entry: ProofObligationsQueueEntry) {
        let ipo = QueueItem {
            cube_index: entry.cube_index,
            frame: entry.frame,
            priority: self.current_priority,
            hash_when_added: entry.hash_when_added,
        };
        self.current_priority += 1;
        self.queue.push(ipo);
    }

    pub fn len(&self) -> usize {
        self.queue.len()
    }

    pub fn pop(&mut self) -> Option<ProofObligationsQueueEntry> {
        let r = self.queue.pop();
        r.map(|x| ProofObligationsQueueEntry {
            cube_index: x.cube_index,
            frame: x.frame,
            hash_when_added: x.hash_when_added,
        })
    }

    // pub fn peek(&mut self) -> Option<(usize, usize)> {
    //     let r = self.queue.peek();
    //     r.map(|x| (x.cube_index, x.frame))
    // }

    pub fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }
}

// ************************************************************************************************
// impl traits for proof obligation
// ************************************************************************************************

impl Ord for QueueItem {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // smallest frame, smallest priority is the "greatest" proof obligation
        other
            .frame
            .cmp(&self.frame)
            .then(other.priority.cmp(&self.priority))
            .then(other.cube_index.cmp(&self.cube_index))
    }
}

impl PartialOrd for QueueItem {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for QueueItem {
    fn eq(&self, other: &Self) -> bool {
        self.cube_index == other.cube_index
            && self.frame == other.frame
            && self.priority == other.priority
    }
}

impl Eq for QueueItem {}
