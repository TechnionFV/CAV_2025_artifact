# CAV artifact for standard paper

This artifact is built on top of a formal verification framework developed by us in the rust programming language. The framework is yet to be officially released. However, the parts of it that are needed to re-produce the results in our paper are present in this artifact.



* Title of the submitted paper: `Property Directed Reachability with Extended Resolution`
* Submission number: `174`
* Claimed badges, all three badges:
* We request that this artifact be considered for all three badges.
* Available Badge: The artifact is made available on Zenodo, the DOI is `?`
* Functional Badge: The artifact documents the steps that need to be taken to reproduce the results in the paper. The documentation includes a dockerfile to retrace the exact steps needed.
* Reusable Badge: It also provides a dockerfile to run the proof solver as a standalone tool. This allows for checking arbitrary AIGER files and producing proofs and counterexamples. As well as allowing for fine grained control over all parameters mentioned in the paper, as well as some parameters that were not.



## Artifact Requirements

Reproducing the results in the paper requires running around 950 models with three solvers each for a timeout of one hour.
Furthermore, running two solvers on the same machine at the same time may result in random noise in performance metrics due to the two processes conflicting in memory cache.

Thus the results we've shown in the paper were ran on 11 machines running in a cluster. Each machine was split into 8 virtual machines where each virtual machine has a dedicated L3 cache. This allows us to run 88 jobs in parallel. It is worth noting that this split was performed in the cluster configuration, and the term virtual machine does not refer to a separate OS in this context.

Each job is a run of an solver on a model, with a timeout provided by `/bin/timeout` (not relying on solver timeouts).
Each virtual machine had access to 32GB of RAM. Each machine had three non-hyper-threaded cores of AMD EPYC 74F3 CPU.

Since these tests require a long time to run, we also allow the option to run the experiments partially by providing a list of tests or a common phrase in test names one would like to run. Furthermore, we allow either running the experiments one at a time with one single core, utilizing multiple cores on the same machine, or sending the jobs to a [slurm](https://slurm.schedmd.com/overview.html) cluster. 


## Structure and Content


```bash
.
├── README.md               (this file)
├── LICENSE                 (GPL3 license)
├── benchmarks_aig          (Benchmark files)
│   ├── aig_inputs          (AIGER benchmarks)
│   ├── Dockerfile          (Dockerfile to reproduce hwmcc benchmarks)
│   └── convert.py          (Script for pre-processing AIG files)
├── pdrer_crate
│   ├── Dockerfile          (Dockerfile to run the PDR/PDRER as a standalone tool for proving AIG files)
│   ├── Cargo.toml          (Rust dependencies file, all open-source)
│   ├── src                 (Location of PDR and PDER)
│   └── examples            (Contains the implementation of the main function)
└── evaluate.sh
```

For viewing the implementation of our project one can refer to `pdrer_crate` this is a standalone crate (rust's terminology for a library) that includes many of the data-structures required to implement the solver. This library can be compiled using `cargo build` and be used and expanded on outside the scope of this artifact. For viewing the documentation for the library run `cd pdrer_crate ; cargo doc --open` provided you have rust installed.

## Getting Started (Smoke Test)

<!-- Describe how to execute and briefly test your artifact in order to complete the smoke-test phase of the evaluation. Below is an example for Docker images. -->

First make sure docker is running properly by running:
```
docker run hello-world
```

Then check that you are able to run the PDR/PDRER solver by running:
```
docker build -t pdr_image pdrer_crate/.
docker run \
    -v $(pwd)/benchmarks_aig/aig_inputs:/aig_inputs \
    pdr_image \
    -v on \
    aig_inputs/hwmcc19_fold_fraigy_orchestrate/aig/goel/industry/cal3/cal3.aig
```

The output of the previous commands should be similar to `expected_results/smoke_test.out`


## Creating HWMCC benchmarks

In our paper we use **And Inverter Graph (AIG)** benchmarks in the `AIGER` format. These benchmarks are taken from sets introduced by past runs of the **Hardware Model Checking Competition (HWMCC)**. The AIGs are first pre-processed by ABC before running any benchmarks. The preprocessing entails three steps:

1. Fold the AIGER file: a latch is introduced and the constraints are removed by "folding" them into the property.
2. Functionally Reduced And Inverter Graphs (FRAIG): which decreases the size of the AIG
3. Orchestrate: reorganizes the AIG

These preprocessing steps were given a combined timeout of 2 minutes, where checkpoints were saved between each two consecutive steps.

We implemented a Dockerfile that produces these simplified versions of the benchmarks, to produce the benchmarks yourself please run:
```bash
cd benchmarks_aig
docker build --tag hwmcc_producer_image . 
docker create --name hwmcc_producer hwmcc_producer_image
docker cp hwmcc_producer:/usr/src/hwmcc19_fold_fraigy_orchestrate ./aig_inputs/hwmcc19_fold_fraigy_orchestrate
docker cp hwmcc_producer:/usr/src/hwmcc20_fold_fraigy_orchestrate ./aig_inputs/hwmcc20_fold_fraigy_orchestrate
docker cp hwmcc_producer:/usr/src/hwmcc24_fold_fraigy_orchestrate ./aig_inputs/hwmcc24_fold_fraigy_orchestrate
docker rm -f hwmcc_producer
```

Building the docker container requires running the pre-processing step on 959 benchmarks which can take up to `32 hours / # cores`. Additionally, due to timeout constraints, the resulting benchmark may differ slightly in the benchmark that we used for testing, thus we provide the benchmark we used in aig_inputs.

Furthermore, as part of our paper, we produced our own benchmarks to demonstrate some features of our proposed algorithm, these are also available in `benchmarks_aig/aig_inputs/ER_hwmc_benchmarks`


<!-- ### Getting Started (example)

First, load the docker image `docker-tool-image` from the .tar archive (docker may require `sudo` root privileges):

```bash
docker load < docker-tool-image.tar
```

Upon loading the image, you can run the container with:

```bash
docker run -v `pwd`/output:/tool/output --rm -it docker-tool
```

The command above starts the docker container and places you in a bash environment, where you can inspect the source code or run the experiments. `-v` option will mount `output` folder in your current directory to the corresponding folder within the container where the evaluation results will be stored. This will allow you to view the generated output even after the container has stopped running. `--rm` is an optional flag that creates a disposable container that will be deleted upon exit.

To run all the experiments (should take up to 8 hours), use:

```bash
./evaluate.sh 
```

The evaluation script has the following additional options:
* `--smoke-test` option allows you to detect any technical difficulties for the smoke-test phase (should take up to 5 minutes)
* `--brief` option allows you to run the subset of experiments, namely Tables 1 & 4 of the paper (should take up to an hour)

If finished successfully, the evaluation script should print:

```
All experiments were successful.
```

You can exit the container by typing `exit`. Output files generated by the evaluation script (logs, tables, plots, etc.) remain available in `$PWD/output`. Upon finishing your review, you can remove the image from the Docker environment using:
```
docker rmi docker-tool
``` -->


## Functional badge

If you claim a functional badge for the artifact:

* Document which claims or results of the paper can be replicated with the artifact and how, including how to run the experiments and how to read and interpret the output. To simplify the reviewing process, we recommend providing evaluation scripts (where applicable).
* Document which claims or results of the paper cannot be replicated and why.
* Explain how the correctness of the artifact (i.e. the presented tool/method) was tested.
* If possible, include log files reporting the results that were presented in the paper, and point to their location in the artifact.
* If possible, include source code within your artifact, and point the reviewer to the parts of the source code that are most relevant to the submitted paper.


## Reusable badge

If you claim a reusable badge for the artifact:

* Make sure your artifact has a license which allows repurposing and reuse, and is easy to use.
* Make sure that all dependencies and used libraries are well-documented and up to date.
* Explain in sufficient detail how the artifact can be used beyond the paper.
* If the artifact is not open source, provide documented interfaces for extensions.
* Explain how the artifact can be used in a different environment, e.g. built on another system, used outside of the Docker or VM image, etc.