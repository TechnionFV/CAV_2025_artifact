// ************************************************************************************************
// use
// ************************************************************************************************

use std::{
    fmt,
    time::{Duration, Instant},
};

use crate::models::PrettyTable;

// ************************************************************************************************
// struct
// ************************************************************************************************

#[derive(Debug, Clone, Copy)]
pub enum AnalysisTiming {
    Periodic(usize),
    ExponentialDecayWithPeriodicMaximum {
        start: usize,
        multiplier: usize,
        maximum: usize,
    },
    Once(usize),
    Skip,
}

#[derive(Debug, Clone, Copy)]
pub struct PropertyDirectedReachabilityParameters {
    /// Random seed for the random number generator, this is used to generate the seeds
    /// for the sat solvers, for shuffling the clauses, etc.
    pub seed: u64,
    /// The time when the algorithm started, this is initialized when this object is created.
    pub start_time: Option<Instant>,

    /// The maximum time that the algorithm is allowed to run for.
    pub timeout: Duration,
    /// The maximum depth that the algorithm will go to.
    pub max_depth: usize,

    /// If true, then the algorithm will print out information about what it is doing (similar to verbose settings in other engines).
    pub verbose: bool,
    /// If true, then the algorithm will print out the time statistics of the functions that are running during the run.
    pub should_print_time_stats_during_run: bool,
    /// If true, then the algorithm will print out the clauses when they are added into any frame.
    pub should_print_clauses_when_added: bool,
    /// If true, then the algorithm will print out the clauses when they are added into any frame, the format will be ternary.
    pub should_print_clauses_when_added_as_ternary: bool,
    /// If true, then the algorithm will print out the proof obligations.
    pub should_print_proof_obligations: bool,
    /// If true, then the algorithm will print out when a new extension variable is added.
    pub should_print_when_extension_variable_is_added: bool,
    /// If true, then the algorithm will print out information about the analysis using the largest inductive cycle.
    pub should_print_largest_inductive_cycle_progress: bool,
    /// If true, then the algorithm will print out debug information about the BVA run.
    pub should_print_bva_debug_information: bool,

    /// The minimum length of a clause that is needed to be able to generalize it.
    /// (This is used to prevent generalizing small clauses).
    /// If the clause is smaller than this, then it will not be generalized.
    pub minimum_clause_length_to_generalize: usize,
    /// Amount to decay the weights of each literal after generalize.
    pub decay: f64,
    /// If true, transition clauses and extension variable definitions are revered when added to the solver
    pub insert_transition_clauses_reversed: bool,
    /// If true, the extension variable definition clauses will be revered when added to the solver.
    pub insert_extension_variable_definitions_reversed: bool,
    /// If true, the clauses inside the frames are inserted in reversed order
    pub insert_frame_clauses_reversed: bool,
    /// If true, then the algorithm will only use one solver for all frames and use activation literals.
    pub use_only_one_solver: bool,

    /// If true, then the algorithm will use the infinite frame (Attempt to push to it when propagating).
    pub use_infinite_frame: bool,
    /// Limit for inf frame propagation
    pub infinite_frame_propagation_limit: usize,
    /// During generalization of a clause in f inf, the clause can be added as soon as it is known to be inductive in F inf,
    /// this helps generalization become stronger because F_inf changes as the clause is generalized.
    pub continuously_insert_in_f_inf_generalize: bool,
    /// Should assume constraints in second cycle of transition relation
    pub assume_constraints_in_second_cycle: bool,
    /// Should simplify the CNF at construction
    pub simplify_cnf_at_construction: bool,

    /// If true, then the algorithm will perform an analysis that checks if some clauses are inductive.
    pub perform_lic_analysis: bool,
    /// If true, then the algorithm will add the extension variables before performing the LIC analysis.
    pub use_extension_variables_in_lic_analysis: bool,
    /// The amount of LIC calls that are done in stage 1 of the LIC analysis.
    /// Stage 1 is the stage where LIC calls are called on each proof obligation that does not have a predecessor.
    pub lic_calls_in_stage_1: usize,
    /// After stage 1, the algorithm will continue to perform LIC analysis on the proof obligations.
    /// However, the algorithm will only continue to do so as long as the success rate of LIC analysis calls
    /// does not drop below this value.
    pub lic_min_success_rate_in_stage_2: f64,
    pub lic_max_success_rate_in_stage_2: f64,

    /// Should propagating clauses start from lowest changed frame or from the first frame.
    pub propagate_from_lowest_changed_frame: bool,

    /// If true, then new extension variables will be added when needed.
    pub er: bool,
    /// If true, extension variables will be used when performing generalization.
    pub er_generalization: bool,
    /// number of clause inserts to frames before condense is called
    pub er_delta: usize,
    /// If true, then when a propagation of a clause with an extension variable fails. The algorithm will
    /// recursively try to propagate a sub-clause derived from "splitting" the extension variable.
    /// This procedure is called fractional propagation.
    pub er_fp: bool,
    /// Use fractional propagation when propagating beyond the depth reached by PDR. This does nothing if er_fp is false.
    pub er_fp_for_f_inf: bool,
    /// Perform the implication check correctly by not only doing syntactic subsumption but also BDDs
    pub er_impl: bool,

    // /// Controls how the algorithm will generalize with extension variables.
    // pub generalize_with_extension_variables: bool,
    // /// If true, then the algorithm will re-write the frame delta when BVA finds a simplification for that delta.
    // /// Otherwise, bva is only used for defining new signals but does not use those signals immediately.
    //pub re_write_frame_when_bva_finds_simplification: bool,
    // /// The amount of clauses that the frame needs to grow in order to call condense on it again.
    // pub delta_in_frame_size_to_call_condense: usize,
    // /// The maximum amount of variables that can be added during BVA on a frame. (BVA is called during condense).
    // pub max_variables_to_add_during_bva: usize,
    // /// If true, then the algorithm will always condense the frame when a new clause is added to it.
    // pub condense_always: bool,
    /// Minimum amount of pairs that need to be in to add a definition.
    pub min_match_count_to_add_definition: usize,

    /// generalize using Counterexample To Generalization as presented in
    /// Z. Hassan, A. R. Bradley and F. Somenzi, "Better generalization in IC3"
    pub generalize_using_ctg: bool,
    pub generalize_using_ctg_max_depth: usize,
    pub generalize_using_ctg_max_ctgs: usize,
}

// ************************************************************************************************
// impl
// ************************************************************************************************

impl PropertyDirectedReachabilityParameters {
    pub const DEFAULT: Self = Self {
        seed: 43,
        start_time: None,
        timeout: Duration::MAX,
        max_depth: usize::MAX,

        verbose: false,
        should_print_time_stats_during_run: false,
        should_print_clauses_when_added: false,
        should_print_clauses_when_added_as_ternary: false,
        should_print_proof_obligations: false,
        should_print_when_extension_variable_is_added: false,
        should_print_largest_inductive_cycle_progress: false,
        should_print_bva_debug_information: false,

        minimum_clause_length_to_generalize: 2,
        decay: 0.9,
        insert_transition_clauses_reversed: false,
        insert_extension_variable_definitions_reversed: false,
        insert_frame_clauses_reversed: false,
        use_only_one_solver: true,

        use_infinite_frame: true,
        infinite_frame_propagation_limit: usize::MAX,
        continuously_insert_in_f_inf_generalize: false,
        assume_constraints_in_second_cycle: false,
        simplify_cnf_at_construction: true,

        perform_lic_analysis: false,
        use_extension_variables_in_lic_analysis: false,
        lic_calls_in_stage_1: 100,
        lic_min_success_rate_in_stage_2: 0.001,
        lic_max_success_rate_in_stage_2: 2.0,

        propagate_from_lowest_changed_frame: false,

        er: true,
        er_generalization: true,
        er_delta: 1408,
        er_fp: true,
        er_fp_for_f_inf: true,
        er_impl: true,
        // generalize_with_extension_variables: true,
        // re_write_frame_when_bva_finds_simplification: true,
        // delta_in_frame_size_to_call_condense: 8,
        // max_variables_to_add_during_bva: 1,
        // condense_always: false,
        min_match_count_to_add_definition: 1,
        // min_pair_count_to_add_and_definition: 1,
        generalize_using_ctg: false,
        generalize_using_ctg_max_depth: 1,
        generalize_using_ctg_max_ctgs: 3,
    };

    pub fn new() -> Self {
        let mut d = Self::DEFAULT;
        d.start_time = Some(Instant::now());
        d
    }
}

// ************************************************************************************************
// Default
// ************************************************************************************************

impl Default for PropertyDirectedReachabilityParameters {
    fn default() -> Self {
        Self::new()
    }
}

// ************************************************************************************************
// Display
// ************************************************************************************************

macro_rules! write_filed {
    ($self:ident, $table:ident, $field:expr) => {
        $table
            .add_row(vec![
                stringify!($field).to_string().replace("self.", ""),
                $field.to_string(),
            ])
            .unwrap();
    };
}

impl fmt::Display for PropertyDirectedReachabilityParameters {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut table = PrettyTable::new(vec!["Parameter".to_string(), "Value".to_string()]);

        write_filed!(self, table, self.seed);
        write_filed!(
            self,
            table,
            self.start_time.unwrap().elapsed().as_secs_f32()
        );

        write_filed!(self, table, self.timeout.as_secs_f32());
        write_filed!(self, table, self.max_depth);

        write_filed!(self, table, self.verbose);
        write_filed!(self, table, self.should_print_time_stats_during_run);
        write_filed!(self, table, self.should_print_clauses_when_added);
        write_filed!(self, table, self.should_print_clauses_when_added_as_ternary);
        write_filed!(self, table, self.should_print_proof_obligations);
        write_filed!(
            self,
            table,
            self.should_print_when_extension_variable_is_added
        );
        write_filed!(
            self,
            table,
            self.should_print_largest_inductive_cycle_progress
        );
        write_filed!(self, table, self.should_print_bva_debug_information);

        write_filed!(self, table, self.minimum_clause_length_to_generalize);
        write_filed!(self, table, self.decay);
        write_filed!(self, table, self.insert_transition_clauses_reversed);
        write_filed!(
            self,
            table,
            self.insert_extension_variable_definitions_reversed
        );
        write_filed!(self, table, self.insert_frame_clauses_reversed);
        write_filed!(self, table, self.use_only_one_solver);

        write_filed!(self, table, self.use_infinite_frame);
        write_filed!(self, table, self.infinite_frame_propagation_limit);
        write_filed!(self, table, self.continuously_insert_in_f_inf_generalize);
        write_filed!(self, table, self.assume_constraints_in_second_cycle);
        write_filed!(self, table, self.simplify_cnf_at_construction);

        write_filed!(self, table, self.perform_lic_analysis);
        write_filed!(self, table, self.use_extension_variables_in_lic_analysis);
        write_filed!(self, table, self.lic_calls_in_stage_1);
        write_filed!(self, table, self.lic_min_success_rate_in_stage_2);
        write_filed!(self, table, self.lic_max_success_rate_in_stage_2);

        write_filed!(self, table, self.propagate_from_lowest_changed_frame);

        write_filed!(self, table, self.er);
        write_filed!(self, table, self.er_generalization);
        write_filed!(self, table, self.er_delta);
        write_filed!(self, table, self.er_fp);
        write_filed!(self, table, self.er_fp_for_f_inf);
        write_filed!(self, table, self.er_impl);
        // write_filed!(self, table, self.generalize_with_extension_variables);
        // write_filed!(
        //     self,
        //     table,
        //     self.re_write_frame_when_bva_finds_simplification
        // );
        // write_filed!(self, table, self.delta_in_frame_size_to_call_condense);
        // write_filed!(self, table, self.max_variables_to_add_during_bva);
        // write_filed!(self, table, self.condense_always);
        write_filed!(self, table, self.min_match_count_to_add_definition);
        // write_filed!(self, table, self.min_pair_count_to_add_and_definition);
        write_filed!(self, table, self.generalize_using_ctg);
        write_filed!(self, table, self.generalize_using_ctg_max_depth);
        write_filed!(self, table, self.generalize_using_ctg_max_ctgs);

        writeln!(f, "{}", table)
    }
}
