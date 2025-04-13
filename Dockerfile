FROM python:3.13-slim

WORKDIR /usr/src/benchmark_runner

COPY ./benchmarks_aig/aig_inputs ./

