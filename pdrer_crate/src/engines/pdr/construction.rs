// ************************************************************************************************
// use
// ************************************************************************************************

use std::cell::RefCell;
use std::rc::Rc;

use super::frames::Frames;
use super::pdr_stats::PDRStats;
use super::proof_obligations::ProofObligations;
use super::shared_objects::SharedObjects;
use super::{
    PropertyDirectedReachability, PropertyDirectedReachabilityError,
    PropertyDirectedReachabilityParameters, PropertyDirectedReachabilitySolver, Weights,
};
use crate::models::{FiniteStateTransitionSystem, TimeStats};
use crate::solvers::dd::DecisionDiagramManager;
use rand::{rngs::StdRng, SeedableRng};

// ************************************************************************************************
// impl
// ************************************************************************************************

impl<T: PropertyDirectedReachabilitySolver, D: DecisionDiagramManager>
    PropertyDirectedReachability<T, D>
{
    // ********************************************************************************************
    // helper functions
    // ********************************************************************************************

    // pub(super) fn static_print(
    //     params: &PropertyDirectedReachabilityParameters,
    //     operation: &str,
    //     message: &str,
    // ) {
    //     if params.verbose {
    //         println!(
    //             "LTLS - {:.3} - {operation} - {message}",
    //             params.start_time.elapsed().as_secs_f32()
    //         );
    //     }
    // }

    // ********************************************************************************************
    // API
    // ********************************************************************************************

    pub fn new(
        fin_state: Rc<RefCell<FiniteStateTransitionSystem>>,
        parameters: PropertyDirectedReachabilityParameters,
    ) -> Result<Self, PropertyDirectedReachabilityError> {
        let weights = Rc::new(RefCell::new(Weights::new(
            &fin_state.borrow(),
            parameters.decay,
        )));
        let s = SharedObjects {
            weights,
            fin_state,
            rng: Rc::new(RefCell::new(StdRng::seed_from_u64(parameters.seed))),
            parameters: Rc::new(parameters),
            time_stats: Rc::new(RefCell::new(TimeStats::new())),
            pdr_stats: Rc::new(RefCell::new(PDRStats::new())),
        };

        let frames = Frames::new(s.clone());

        Ok(Self {
            frames,
            proof_obligations: ProofObligations::new(),
            s,
        })
    }
}
