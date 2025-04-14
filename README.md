# CAV artifact for standard paper

This artifact is built on top of a formal verification framework developed by us in the rust programming language. 
The framework will be officially released on GitHub. 
The parts of it that are needed to re-produce the results in our paper are present in this artifact.

* Title of the submitted paper: `Property Directed Reachability with Extended Resolution`
* Submission number: `174`
* We request that this artifact be considered for all three badges.
    * Available Badge: The artifact is made available on Zenodo, the DOI is `?`
    * Functional Badge: The artifact documents the steps that need to be taken to reproduce the results in the paper. The documentation includes a dockerfile to retrace the exact steps needed.
    * Reusable Badge: It also provides a dockerfile to run the proof solver as a standalone tool. This allows for checking arbitrary AIGER files and producing proofs and counterexamples. As well as allowing for fine grained control over all parameters mentioned in the paper, as well as some parameters that were not.

## Artifact Requirements

Reproducing the results in the paper requires running around 950 models with three solvers each for a timeout of one hour.
Furthermore, running two solvers on the same machine at the same time may result in random noise in performance metrics due to the two processes conflicting in memory cache.

It is for this reason that each benchmark and prover pair (which I'll call **job**) were run on a dedicated machine. These runs were performed on 88 machines running `ubuntu 22.04` .

Each job ran with a timeout provided by `/bin/timeout` (not relying on solver timeouts).
Each machine had access to 32GB of RAM with a AMD EPYC 74F3 CPU (3.2 GHz).

Since these tests require a long time to run, we also allow the option to run the experiments partially by providing a list of tests or a common phrase in test names one would like to run. Furthermore, we allow either running the experiments one at a time with one single core or utilizing multiple cores on the same machine.

## Structure and Content

```
.
├── README.md               (this file)
├── LICENSE                 (GPL3 license)
├── Dockerfile              (Dockerfile to reproduce the results)
├── expected_results        (Expected results of the smoke test and the experiments)
├── benchmark-hwmc          (Scripts to run the experiments with different configurations)
├── pdrer_crate
│   ├── Dockerfile          (Dockerfile to run the PDR/PDRER as a standalone tool for proving AIG files)
│   ├── Cargo.toml          (Rust dependencies file, all open-source)
│   ├── src                 (Location of PDR and PDER)
│   └── examples            (Contains the implementation of the main function)
└── benchmarks_aig          (Contains benchmark files)
    ├── aig_inputs          (AIGER benchmarks)
    ├── Dockerfile          (Dockerfile to reproduce hwmcc benchmarks)
    └── convert.py          (Script for pre-processing AIG files)
```

For viewing the implementation of our project one can refer to `pdrer_crate` this is a standalone crate (rust's terminology for a library) that includes many of the data-structures required to implement the solver. This library can be compiled using `cargo build` and be used and expanded on outside the scope of this artifact. For viewing the documentation for the library run `cd pdrer_crate ; cargo doc --open` provided you have rust installed.

## Getting Started (Smoke Test)

Provided is a list of steps to check that this artifact works.

### Step 1

First make sure docker is running properly by running:
```
docker run hello-world
```
Estimated Time: less than one minute

### Step 2

Then check that you are able to run the PDR/PDRER solver by running:
```
docker build -t pdr_image pdrer_crate/.
docker run -v $(pwd)/benchmarks_aig/aig_inputs:/aig_inputs pdr_image -v on aig_inputs/hwmcc19_fold_fraigy_orchestrate/aig/goel/industry/cal3/cal3.aig
```
Estimated Time: 2 minutes

The output of the previous commands should be similar to `expected_results/smoke_test.out` yet most likely not identical due to time differences.


### Step 3

#### Part a

Running a small experiment, first build the container:
```
docker build -t exp .
```
Estimated Time: less than one minute

#### Part b

Now check that the script outputs its usage description:
```
docker run exp --help
```
Estimated Time: 1 second

#### Part c

As apparent from the previous command, the experimentation script can be run with multiple option:
* `--local` or `--slurm` for the purposes of this artifact `--local` must be used.
* `--suite` dictates the benchmark suite you will run
* `-c` number of cores to use
* `-t` select timeout for each run in seconds
* `-d` the prover you want to run, use 0 for ABC PDR, 7 for our PDR and 8 for PDRER
* `--tests` a list of strings that filters which files to run from the suite, for a test to run it must have one of these strings in its path. 

Now run the three solvers on a subset of `hwmcc19_fold_fraigy_orchestrate` suite with a 30 second timeout. Where the subset is defined as the set of benchmarks containing the word `vis`

```
docker run exp --local -c 1 -d 0 -t 30 --suit hwmcc19_fold_fraigy_orchestrate --tests vis
docker run exp --local -c 1 -d 7 -t 30 --suit hwmcc19_fold_fraigy_orchestrate --tests vis
docker run exp --local -c 1 -d 8 -t 30 --suit hwmcc19_fold_fraigy_orchestrate --tests vis
```
Estimated Time: 10 minutes

Each experiment produces amongst other things a results CSV file that is available in the path `/usr/src/benchmark-hwmc/results/deployment_0.csv` inside the container.
For convince the CSV is also printed at the end of an experiment run. 

The directory `expected_results` includes the results we got on all the experiments as a reference.

## Re-producing all benchmarks

To reproduce all benchmarks with 8 cores run:
```
docker run exp --local -c 8 -d 0 -t 3600 --suit hwmcc19_fold_fraigy_orchestrate 
docker run exp --local -c 8 -d 7 -t 3600 --suit hwmcc19_fold_fraigy_orchestrate
docker run exp --local -c 8 -d 8 -t 3600 --suit hwmcc19_fold_fraigy_orchestrate

docker run exp --local -c 8 -d 0 -t 3600 --suit hwmcc20_fold_fraigy_orchestrate 
docker run exp --local -c 8 -d 7 -t 3600 --suit hwmcc20_fold_fraigy_orchestrate
docker run exp --local -c 8 -d 8 -t 3600 --suit hwmcc20_fold_fraigy_orchestrate

docker run exp --local -c 8 -d 0 -t 3600 --suit hwmcc24_fold_fraigy_orchestrate 
docker run exp --local -c 8 -d 7 -t 3600 --suit hwmcc24_fold_fraigy_orchestrate
docker run exp --local -c 8 -d 8 -t 3600 --suit hwmcc24_fold_fraigy_orchestrate
```
Estimated Time: 8 days \
Maximum Time: 15 days

Since running this is too long you can choose to run specific tests and compare to the expected results in `expected_results` like so:

```
docker run exp --local -c 8 -d 7 -t 3600 --suit hwmcc<number>_fold_fraigy_orchestrate --tests <test_name_1> <test_name_2> ...
```

## Input Files

Out solver expects an AIG with a single safety property (the old AIGER format, prior to 1.9). The artifact includes all AIG files after being processed by ABC to produce AIGs in the format rfv expects (e.g. folding constraint into the property).

All the benchmarks are available in `benchmarks_aig/aig_inputs`. In `benchmarks_aig/README.md` the procedure to convert these benchmarks is described.

