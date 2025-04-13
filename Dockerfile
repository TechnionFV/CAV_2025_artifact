FROM rust:1.86

RUN apt-get update
RUN apt-get install -y --no-install-recommends python3 python3-pip libssl-dev ca-certificates
RUN apt-get update -y
RUN apt-get install -y libx11-dev
RUN apt-get install -y python3-tk
RUN apt-get install -y python3-matplotlib

WORKDIR /usr/src/benchmark-hwmc

COPY ./benchmarks_aig/aig_inputs ./aig_inputs
COPY ./benchmark-hwmc ./src
COPY ./pdrer_crate /usr/src/pdrer_crate

ENTRYPOINT [ "python3", "./src/main.py" ]
CMD [ "./src/main.py" ]