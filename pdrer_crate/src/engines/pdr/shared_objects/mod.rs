use std::{cell::RefCell, rc::Rc};

use rand::rngs::StdRng;

use crate::models::{FiniteStateTransitionSystem, TimeStats};

use super::{pdr_stats::PDRStats, PropertyDirectedReachabilityParameters, Weights};

#[derive(Debug, Clone)]
pub struct SharedObjects {
    /// FiniteStateTransitionSystem we operate on
    pub fin_state: Rc<RefCell<FiniteStateTransitionSystem>>,
    /// importance of variables, used in generalization
    pub weights: Rc<RefCell<Weights>>,
    /// random number generator that allows engine to be pseudo-random
    pub rng: Rc<RefCell<StdRng>>,
    /// parameters that the algorithm is operating with
    pub parameters: Rc<PropertyDirectedReachabilityParameters>,
    /// time statistics
    pub time_stats: Rc<RefCell<TimeStats>>,
    /// PDR specific statistics
    pub pdr_stats: Rc<RefCell<PDRStats>>,
}
