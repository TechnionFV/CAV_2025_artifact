FROM ubuntu:25.04

# get and build abc
WORKDIR /usr/src/abc
# install git
RUN apt-get update
RUN apt-get install -y git
RUN apt-get install -y make
RUN apt-get install -y g++
# get the code for abc from their github
RUN git clone --depth 1 https://github.com/berkeley-abc/abc.git .
# compile the code
RUN make ABC_USE_NO_READLINE=1 -j
# Copy abc to bin path
RUN cp abc /usr/local/bin/abc


RUN apt-get install -y curl
RUN apt-get install -y xz-utils

# get benchmarks
WORKDIR /usr/src/hwmcc/19
RUN curl "https://fmv.jku.at/hwmcc19/hwmcc19-single-benchmarks.tar.xz" -o hwmcc19-single-benchmarks.tar.xz -#
RUN tar -xf hwmcc19-single-benchmarks.tar.xz
RUN rm -rf hwmcc19-single-benchmarks.tar.xz btor2

WORKDIR /usr/src/hwmcc/20
RUN curl "https://fmv.jku.at/hwmcc20/hwmcc20benchmarks.tar.xz" -o hwmcc20benchmarks.tar.xz -#
RUN tar -xf hwmcc20benchmarks.tar.xz
RUN rm -rf hwmcc20benchmarks.tar.xz hwmcc20/btor2 hwmcc20/README

WORKDIR /usr/src/hwmcc/24
RUN curl "https://zenodo.org/records/14156844/files/benchmarks_aiger.tar.gz?download=1" -o benchmarks_aiger.tar.gz -#
RUN tar -xf benchmarks_aiger.tar.gz
RUN rm -rf benchmarks_aiger.tar.gz
# Removing malformed AIG in the benchmark
RUN ls aiger/2024/sosylab/eca-rers2018/Problem17.aig
RUN rm -rf aiger/2024/sosylab/eca-rers2018/Problem17.aig

# fold, fraig and orchastrate benchmarks
WORKDIR /usr/src/convert
# copy artifact scripts
COPY convert.py ./
# install python
RUN apt-get update && apt-get install -y python3
# run python convertion script
RUN python3 convert.py /usr/src/hwmcc/19 /usr/src/hwmcc19_fold_fraigy_orchestrate
RUN python3 convert.py /usr/src/hwmcc/20 /usr/src/hwmcc20_fold_fraigy_orchestrate
RUN python3 convert.py /usr/src/hwmcc/24 /usr/src/hwmcc24_fold_fraigy_orchestrate