FROM python:3.13-slim

WORKDIR /usr/src/benchmark-hwmc

COPY ./benchmarks_aig/aig_inputs ./aig_inputs
COPY ./benchmark-hwmc ./src

ENTRYPOINT [ "python", "./src/main.py" ]
CMD [ "./src/main.py" ]