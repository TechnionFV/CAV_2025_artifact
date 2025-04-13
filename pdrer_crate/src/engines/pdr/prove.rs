// ************************************************************************************************
// use
// ************************************************************************************************

use fxhash::FxHashMap;

use super::{
    PropertyDirectedReachability, PropertyDirectedReachabilityProofError,
    PropertyDirectedReachabilitySolver,
};
use crate::{
    formulas::{Clause, Cube, Literal, Variable, CNF},
    function,
    models::{
        finite_state_transition_system::ProofResult, time_stats::function_timer::FunctionTimer,
        Definition, Proof, SortedVecOfLiterals,
    },
    solvers::{dd::DecisionDiagramManager, sat::incremental::CaDiCalSolver},
};

// ************************************************************************************************
// impl
// ************************************************************************************************

impl<T: PropertyDirectedReachabilitySolver, D: DecisionDiagramManager>
    PropertyDirectedReachability<T, D>
{
    /// Performs the proof on the provided model
    pub fn prove(&mut self) -> Result<ProofResult, PropertyDirectedReachabilityProofError> {
        let _timer = FunctionTimer::start(function!(), self.s.time_stats.clone());
        self.print_start_message_if_verbose();
        if let Some(t) = self.s.fin_state.borrow().is_trivial::<CaDiCalSolver>() {
            self.print_final_message_if_verbose(false);
            return Ok(t);
        }

        loop {
            let optional_c = self.frames.get_bad_cube(self.frames.depth());
            let result = self.perform_proof_iteration(optional_c);
            match result {
                Ok(r) => {
                    if let Some(r) = r {
                        self.print_final_message_if_verbose(false);
                        debug_assert!(self.frames.regression_check());
                        return Ok(r);
                    }
                }
                Err(e) => {
                    self.print_final_message_if_verbose(true);
                    debug_assert!(self.frames.regression_check());
                    return Err(e);
                }
            }
        }
    }

    pub fn is_blocked(&mut self, c: &Cube, frame: usize) -> bool {
        let _timer = FunctionTimer::start(function!(), self.s.time_stats.clone());
        self.frames.is_cube_blocked_in_frame(c, frame)
    }

    pub fn is_blocked_in_f_infinity(&mut self, c: &Cube) -> bool {
        let _timer = FunctionTimer::start(function!(), self.s.time_stats.clone());
        self.frames
            .is_cube_blocked_in_frame(c, self.frames.len() - 1)
    }

    pub fn depth(&self) -> usize {
        let _timer = FunctionTimer::start(function!(), self.s.time_stats.clone());
        self.frames.depth()
    }

    fn get_proof(&self, i: usize) -> Proof {
        let mut base = self.s.fin_state.borrow().get_max_variable();
        self.s.fin_state.borrow().add_tags_to_variable(&mut base, 1);
        base = Variable::new(base.number() + 1);

        let ev_mapping: FxHashMap<Variable, Variable> = self
            .frames
            .get_definitions_lib()
            .iter()
            .map(|d| {
                let r = (d.variable, base);
                base = Variable::new(base.number() + 1);
                r
            })
            .collect();

        let map_var = |v: Variable| -> Variable {
            if let Some(v) = ev_mapping.get(&v) {
                return v.to_owned();
            }
            v
        };

        let map_literal = |l: Literal| -> Literal {
            Literal::new(map_var(l.variable())).negate_if_true(l.is_negated())
        };

        Proof {
            all_initial_states_violate_constraints: false,
            invariant: CNF::from_sequence({
                let mut r = self.frames.get_cnf_of_frame(i);
                for c in r.iter_mut() {
                    *c = Clause::from_ordered_set(c.iter().map(|l| map_literal(*l)).collect());
                }
                r
            }),
            definitions: self
                .frames
                .get_definitions()
                .iter()
                .map(|d| Definition {
                    variable: map_var(d.variable),
                    function: d.function,
                    inputs: SortedVecOfLiterals::from_ordered_set(
                        d.inputs
                            .iter()
                            .map(|l| map_literal(*l))
                            .collect::<Vec<Literal>>(),
                    ),
                })
                .collect(),
        }
    }

    pub fn call_propagate(&mut self) -> Option<Proof> {
        let _timer = FunctionTimer::start(function!(), self.s.time_stats.clone());
        let propagation_result = self.frames.propagate();
        if let Some(i) = propagation_result {
            // invariant found may store it here.
            return Some(self.get_proof(i));
        }
        if self.s.parameters.use_infinite_frame {
            let propagation_result = self.frames.propagate_to_infinite_frame();
            if let Some(i) = propagation_result {
                // invariant found may store it here.
                return Some(self.get_proof(i));
            }
        }
        debug_assert!(self.frames.regression_check());

        None
    }

    pub fn increase_pdr_depth(
        &mut self,
    ) -> Result<Option<ProofResult>, PropertyDirectedReachabilityProofError> {
        let _timer = FunctionTimer::start(function!(), self.s.time_stats.clone());

        if self.frames.depth() >= self.s.parameters.max_depth {
            return Err(PropertyDirectedReachabilityProofError::MaxDepthReached);
        }

        self.frames.increase_depth();
        if let Some(proof) = self.call_propagate() {
            return Ok(Some(Ok(proof)));
        }

        // if self.depth() % 20 == 0 {
        //     assert!(self.frames.regression_check());
        // }

        Ok(None)
    }

    pub fn perform_proof_iteration(
        &mut self,
        optional_c: Option<(Cube, Cube)>,
    ) -> Result<Option<ProofResult>, PropertyDirectedReachabilityProofError> {
        let _timer = FunctionTimer::start(function!(), self.s.time_stats.clone());
        // let optional_c = self.z.get_bad_cube(self.depth(), &self.weights);

        if self.s.parameters.start_time.unwrap().elapsed() > self.s.parameters.timeout {
            return Err(PropertyDirectedReachabilityProofError::TimeOutReached);
        }

        match optional_c {
            Some((bad, input)) => {
                debug_assert!(!self
                    .frames
                    .is_cube_blocked_in_frame(&bad, self.frames.depth()));
                self.s
                    .pdr_stats
                    .borrow_mut()
                    .increment_generic_count("Violating State Developed");
                let r = self.recursively_block_cube(bad, input, self.frames.depth());
                match r {
                    Ok(Ok(())) => {}
                    Ok(Err(e)) => return Err(e),
                    Err(er) => {
                        return Ok(Some(Err(er)));
                    }
                }
            }
            None => {
                self.print_progress_if_verbose("Before Propagating");
                let r = self.increase_pdr_depth()?;
                // println!(
                //     "{}",
                //     CNF::from_ordered_set(
                //         self.frames
                //             .at(self.frames.depth() - 1)
                //             .get_unique_clauses()
                //             .to_owned()
                //     )
                // );
                // self.print_progress_if_verbose("After Propagating");
                match r {
                    Some(Ok(p)) => return Ok(Some(Ok(p))),
                    Some(Err(e)) => return Ok(Some(Err(e))),
                    None => {}
                }
            }
        }

        Ok(None)
    }
}
