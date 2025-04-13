//! Should be compiled as a static binary using the following command:
//!
//! ```
//! RUSTFLAGS='-C target-feature=+crt-static' cargo build --release --example pdr_engine_for_hwmcc --target x86_64-unknown-linux-gnu
//! ```
//!
//! To check if the executable is static, you can use the following command:
//! ldd ./target/x86_64-unknown-linux-gnu/release/examples/pdr_engine_for_hwmcc
//! and check if the output is "not a dynamic executable":
//!
//! Output when not static:
//! ```
//!     linux-vdso.so.1 (0x00007fffbebf8000)
//!     libstdc++.so.6 => /lib/x86_64-linux-gnu/libstdc++.so.6 (0x000070555a200000)
//!     libgcc_s.so.1 => /lib/x86_64-linux-gnu/libgcc_s.so.1 (0x000070555a898000)
//!     libm.so.6 => /lib/x86_64-linux-gnu/libm.so.6 (0x000070555a517000)
//!     libc.so.6 => /lib/x86_64-linux-gnu/libc.so.6 (0x0000705559e00000)
//!     /lib64/ld-linux-x86-64.so.2 (0x000070555a8e0000)
//! ```
//!
//! Output when static:
//! ```
//!     statically linked
//! ```
//!
//! Example usage:
//!
//! ```
//! RUSTFLAGS='-C target-feature=+crt-static' cargo build --release --example pdr_engine_for_hwmcc --target x86_64-unknown-linux-gnu &&
//! ./target/x86_64-unknown-linux-gnu/release/examples/pdr_engine_for_hwmcc -v on --extension-variables off ./tests/examples/hwmcc24/benchmarks_aiger/aiger/2024/hku/seq/kalman_wrapper_1_3_2/kalman_bit_width_small.aig
//! ```

// ********************************************************************************************
// imports
// ********************************************************************************************
use clap::Parser;
use rust_formal_verification::{
    engines::{
        pdr::PropertyDirectedReachabilityProofError, PropertyDirectedReachability,
        PropertyDirectedReachabilityParameters,
    },
    formulas::Variable,
    models::{
        finite_state_transition_system::{FiniteStateTransitionSystemError, ProofResult},
        AndInverterGraph, Circuit, Counterexample, FiniteStateTransitionSystem, Proof, Signal,
        SignalTracker,
    },
    solvers::{dd::CuddBdd, sat::incremental::CaDiCalSolver},
};
use std::{
    cell::RefCell,
    fmt::{self, Display, Formatter},
    process::ExitCode,
    rc::Rc,
};
use std::{fs, time::Duration};

// ********************************************************************************************
// Types
// ********************************************************************************************

const D: PropertyDirectedReachabilityParameters = PropertyDirectedReachabilityParameters::DEFAULT;

#[derive(Debug, Clone, Copy, clap::ValueEnum)]
enum Toggle {
    On,
    Off,
}

impl From<bool> for Toggle {
    fn from(b: bool) -> Self {
        if b {
            Toggle::On
        } else {
            Toggle::Off
        }
    }
}

impl From<Toggle> for bool {
    fn from(t: Toggle) -> Self {
        match t {
            Toggle::On => true,
            Toggle::Off => false,
        }
    }
}

impl Display for Toggle {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Toggle::On => write!(f, "on"),
            Toggle::Off => write!(f, "off"),
        }
    }
}

// ********************************************************************************************
// Args struct
// ********************************************************************************************

/// AIGER file proof engine, takes in a path to a ".aig" file and either proves or finds a counter example for the given file.
/// The engine produces witness files for counter examples that can be checked using the AIGER tool `aigsim`.
/// The engine also produces a witness file for the proof of the property that can be checked using the certifaiger tool `check`.
#[derive(Parser, Debug)]
#[command(version = env!("GIT_HASH"), about, long_about = None)]
struct Args {
    /// Path to the AIGER file
    #[arg()]
    input_aig_path: String,

    /// Exit code to return when the property is safe
    #[arg(long, default_value_t = 0)]
    safe_exit_code: u8,

    /// Exit code to return when the property is unsafe
    #[arg(long, default_value_t = 1)]
    unsafe_exit_code: u8,

    /// Exit code to return when the property is unknown
    #[arg(long, default_value_t = 2)]
    unknown_exit_code: u8,

    /// Exit code to return when an error occurs
    #[arg(long, default_value_t = 3)]
    error_exit_code: u8,

    /// Path to the counterexample witness file, if empty then no counterexample witness file will be produced.
    #[arg(long, default_value_t = format!("counterexample.wit"))]
    counterexample: String,

    /// Path to the proof witness file, if empty then no proof witness file will be produced.
    #[arg(long, default_value_t = format!("certificate.aig"))]
    certificate: String,

    /// Seed to seed the random number generator with
    #[arg(short, long, default_value_t = D.seed)]
    seed: u64,

    /// Time out in seconds
    #[arg(short, long, default_value_t = D.timeout.as_secs())]
    time_out_seconds: u64,

    /// Max depth that the PDR engine will reach
    #[arg(long, default_value_t = D.max_depth)]
    max_depth: usize,

    /// Ground the Xs in the counter example, if true then the counter example will not contain x.
    #[arg(short, long, default_value_t = false.into())]
    ground_ternary: Toggle,

    /// Check the result of the proof or the counterexample using an internal check.
    #[arg(short, long, default_value_t = false.into())]
    check_result: Toggle,

    /// Toggle using extended resolution or not.
    #[arg(long, default_value_t = D.er.into())]
    er: Toggle,

    /// Toggle using fractional propagation for extended resolution.
    #[arg(long, default_value_t = D.er_fp.into())]
    er_fp: Toggle,

    /// Toggle using generalization with auxiliary variables or not.
    #[arg(long, default_value_t = D.er_generalization.into())]
    er_gen: Toggle,

    /// Toggle using BDD implication checks when checking for clause redundancies.
    #[arg(long, default_value_t = D.er_impl.into())]
    er_impl: Toggle,

    /// Number of clauses learned between adding extension literals
    /// (relevant only when --extension-variables is on)
    #[arg(long, default_value_t = D.er_delta.into())]
    ev_delta: usize,

    /// Toggle using generalization using CTGs or not.
    #[arg(long, default_value_t = D.generalize_using_ctg.into())]
    ctg: Toggle,

    /// The max depth that is allowed to be reached when using generalization using CTGs.
    #[arg(long, default_value_t = D.generalize_using_ctg_max_depth.into())]
    ctg_depth: usize,

    /// The max number of CTGs that is allowed to be developed when using generalization using CTGs.
    #[arg(long, default_value_t = D.generalize_using_ctg_max_ctgs.into())]
    ctg_count: usize,

    /// Toggle using largest inductive sub-clause analysis
    #[arg(long, default_value_t = D.perform_lic_analysis.into())]
    lic: Toggle,

    /// Amount to decay the weights of each literal after generalize.
    #[arg(long, default_value_t = D.decay.into())]
    decay: f64,

    /// Toggle printing time stats of functions upon each depth increase
    #[arg(long, default_value_t = D.should_print_time_stats_during_run.into())]
    time_stats: Toggle,

    /// Toggle using verbose mode
    #[arg(short, long, default_value_t = D.verbose.into())]
    verbose: Toggle,

    /// Toggle using infinite frame
    #[arg(long, default_value_t = D.use_infinite_frame.into())]
    use_infinite_frame: Toggle,

    /// The limit of frames to add when propagating to the infinite frame
    /// (relevant only when --use-infinite-frame is on)
    #[arg(long, default_value_t = D.infinite_frame_propagation_limit.into())]
    propagation_limit: usize,

    /// Toggle reversing frame clauses when inserting to solver
    #[arg(long, default_value_t = D.insert_frame_clauses_reversed.into())]
    rev_frame_clauses: Toggle,

    /// Toggle reversing transition clauses when inserting to solver
    #[arg(long, default_value_t = D.insert_transition_clauses_reversed.into())]
    rev_tr_clauses: Toggle,

    /// Toggle reversing EV clauses when inserting to solver
    #[arg(long, default_value_t = D.insert_extension_variable_definitions_reversed.into())]
    rev_ev_clauses: Toggle,
}

// ********************************************************************************************
// helper functions
// ********************************************************************************************

fn print_details_about_model(
    fin_state: &FiniteStateTransitionSystem,
    start_time: &std::time::Instant,
) {
    println!(
        "Number of state variables = {}",
        fin_state.get_state_variables().len()
    );
    println!(
        "Number of initialized latches = {}",
        fin_state.get_initial_relation().len()
    );
    println!(
        "Number of input variables = {}",
        fin_state.get_input_variables().len()
    );
    println!(
        "Number of constraints = {}",
        fin_state.get_invariant_constraints_on_internals().len()
    );
    let cnf = fin_state.construct_cnf();
    println!("Number of clauses in CNF = {}", cnf.len());
    println!("Number of variables in CNF = {}", cnf.get_variables().len());
    println!(
        "Number of literals in CNF = {}",
        cnf.iter().map(|c| c.len()).sum::<usize>()
    );
    println!(
        "Time preparing circuit = {}",
        start_time.elapsed().as_secs_f32()
    );
}

// ********************************************************************************************
// main function
// ********************************************************************************************

macro_rules! print_if_verbose {
    ($verbose:expr, $($arg:tt)*) => {
        if $verbose.into() {
            println!($($arg)*);
        }
    };
}

fn declare_sat<F: Fn(Signal) -> Variable>(
    args: &Args,
    aig: &AndInverterGraph,
    t: &SignalTracker,
    e: &Counterexample,
    fin_state: F,
) -> ExitCode {
    print_if_verbose!(
        args.verbose,
        "Unsafe, Counter example found of depth {}.",
        e.inputs.len()
    );
    let w = e.get_aigsim(t, aig, args.ground_ternary.into(), fin_state);
    if !args.counterexample.is_empty() {
        fs::write(&args.counterexample, w).expect("Unable to write counterexample file.");
    } else {
        print_if_verbose!(
            args.verbose,
            "Counterexample path is empty, skipping writing counterexample."
        );
    }
    ExitCode::from(args.unsafe_exit_code)
}

fn declare_un_sat<F: Fn(Signal) -> Variable>(
    args: &Args,
    aig: &AndInverterGraph,
    t: &SignalTracker,
    p: &Proof,
    s2v: F,
) -> ExitCode {
    // println!("I = {}", p.invariant);
    print_if_verbose!(
        args.verbose,
        "Safe, Proof found with {} clauses and {} extension variables.",
        p.invariant.len(),
        p.definitions.len()
    );
    let inv_vars = p.invariant.get_variables();
    for d in &p.definitions {
        if inv_vars.contains(&d.variable) {
            print_if_verbose!(
                args.verbose,
                "Extension Variable used in the invariant = {}, function = {}, inputs = {:?}.",
                d.variable,
                d.function,
                d.inputs
            );
        }
    }
    let witness_aig = p.get_certifaiger_witness(t, aig, s2v);
    let w = witness_aig.get_aig();
    if !args.certificate.is_empty() {
        fs::write(&args.certificate, w).expect("Unable to write certificate file.");
    } else {
        print_if_verbose!(
            args.verbose,
            "Certificate path is empty, skipping writing certificate."
        );
    }
    ExitCode::from(args.safe_exit_code)
}

fn final_print(args: &Args, start_time: &std::time::Instant) {
    print_if_verbose!(
        args.verbose,
        "Elapsed time = {}",
        start_time.elapsed().as_secs_f32()
    );
}

fn main() -> ExitCode {
    std::env::set_var("RUST_BACKTRACE", "1");
    let start_time = std::time::Instant::now();
    let args = Args::parse();

    print_if_verbose!(args.verbose, "aig_file_path = {}", args.input_aig_path);
    let aig = match AndInverterGraph::from_aig_path(&args.input_aig_path) {
        Ok(a) => a,
        Err(e) => {
            print_if_verbose!(
                args.verbose,
                "Error while reading aig file '{}' : {}",
                args.input_aig_path,
                e
            );
            return ExitCode::from(args.error_exit_code);
        }
    };

    let mut circuit = Circuit::from_aig(&aig);
    let t = circuit.simplify_circuit_before_using_proof_engine(args.verbose.into());

    let assume_output_is_bad = circuit.get_bad_wires().is_empty();
    let fin_state = match FiniteStateTransitionSystem::new(&circuit, assume_output_is_bad) {
        Ok(f) => f,
        Err(e) => {
            match e {
                FiniteStateTransitionSystemError::EmptyCircuit
                | FiniteStateTransitionSystemError::ConstraintWireIsConstantZero
                | FiniteStateTransitionSystemError::ConstraintWiresIncludeWireAndItsNegation => {
                    let p = Proof {
                        all_initial_states_violate_constraints: !matches!(
                            e,
                            FiniteStateTransitionSystemError::EmptyCircuit
                        ),
                        invariant: Default::default(),
                        definitions: Default::default(),
                    };
                    let r = declare_un_sat(&args, &aig, &t, &p, |s| Variable::new(s.number()));
                    final_print(&args, &start_time);
                    return r;
                }
                FiniteStateTransitionSystemError::BadWireIsConstantOne => todo!(),
                FiniteStateTransitionSystemError::BadWiresIncludeWireAndItsNegation => todo!(),
                FiniteStateTransitionSystemError::MaxWireTooHigh => {}
            }
            print_if_verbose!(
                args.verbose,
                "Error while building the CNF of the circuit: {}",
                e
            );
            return ExitCode::from(args.error_exit_code);
        }
    };

    if args.verbose.into() {
        print_details_about_model(&fin_state, &start_time);
    }

    let mut parameters = PropertyDirectedReachabilityParameters::new();

    parameters.er = args.er.into();
    parameters.er_fp = args.er_fp.into();
    parameters.er_generalization = args.er_gen.into();
    parameters.er_impl = args.er_impl.into();
    parameters.er_delta = args.ev_delta;

    parameters.verbose = args.verbose.into();
    parameters.timeout = Duration::from_secs(args.time_out_seconds);
    parameters.max_depth = args.max_depth;
    parameters.should_print_time_stats_during_run = args.time_stats.into();
    parameters.seed = args.seed;
    // parameters.should_print_bva_debug_information = true;
    parameters.perform_lic_analysis = args.lic.into();
    parameters.use_infinite_frame = args.use_infinite_frame.into();
    parameters.infinite_frame_propagation_limit = args.propagation_limit;

    parameters.generalize_using_ctg = args.ctg.into();
    parameters.generalize_using_ctg_max_ctgs = args.ctg_count;
    parameters.generalize_using_ctg_max_depth = args.ctg_depth;
    parameters.decay = args.decay;

    parameters.insert_frame_clauses_reversed = args.rev_frame_clauses.into();
    parameters.insert_transition_clauses_reversed = args.rev_tr_clauses.into();
    parameters.insert_extension_variable_definitions_reversed = args.rev_ev_clauses.into();

    let fin_state = Rc::new(RefCell::new(fin_state));

    let mut solver = PropertyDirectedReachability::<CaDiCalSolver, CuddBdd>::new(
        fin_state.to_owned(),
        parameters,
    )
    .unwrap();
    let pr = solver.prove();
    let pr = match &pr {
        Ok(o) => o,
        Err(e) => {
            match e {
                PropertyDirectedReachabilityProofError::MaxDepthReached => {
                    print_if_verbose!(
                        args.verbose,
                        "Max depth reached in PropertyDirectedReachability."
                    )
                }
                PropertyDirectedReachabilityProofError::TimeOutReached => {
                    print_if_verbose!(
                        args.verbose,
                        "Time out reached in PropertyDirectedReachability."
                    )
                }
            }

            return ExitCode::from(args.unknown_exit_code);
        }
    };

    if args.check_result.into() {
        let check_result = fin_state
            .borrow_mut()
            .check_proof_result::<CaDiCalSolver>(pr.clone());
        if let Err(e) = check_result {
            print_if_verbose!(args.verbose, "Error while checking the proof result: {}", e);
            return ExitCode::from(args.error_exit_code);
        }
        print_if_verbose!(args.verbose, "Result checked successfully!");
    }

    // print result
    let r = match &pr {
        ProofResult::Ok(p) => declare_un_sat(&args, &aig, &t, p, |s| {
            fin_state.borrow().convert_signal_to_variable(s)
        }),
        ProofResult::Err(e) => declare_sat(&args, &aig, &t, e, |s| {
            fin_state.borrow().convert_signal_to_variable(s)
        }),
    };

    // print time
    final_print(&args, &start_time);

    r
}
