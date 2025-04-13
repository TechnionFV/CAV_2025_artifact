use super::SignalTracker;

impl SignalTracker {
    pub fn new() -> Self {
        Self {
            transformations: Vec::new(),
        }
    }
}

impl Default for SignalTracker {
    fn default() -> Self {
        Self::new()
    }
}
