# Crea# Creating HWMCC benchmarks

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

Furthermore, as part of our paper, we produced our own benchmarks to demonstrate some features of our proposed algorithm, these are also available in `benchmarks_aig/aig_inputs/ER_hwmc_benchmarks`ting HWMCC benchmarks

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