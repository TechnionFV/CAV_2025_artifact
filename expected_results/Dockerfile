FROM rust:1.86

RUN apt-get update
RUN apt-get install -y --no-install-recommends python3 python3-pip libssl-dev ca-certificates
RUN apt-get update -y
RUN apt-get install -y libx11-dev
RUN apt-get install -y python3-tk
RUN apt-get install -y python3-matplotlib
RUN apt-get install -y time

WORKDIR /usr/src/graph_maker

COPY . .

ENTRYPOINT [ "python3", "./make_graphs.py" ]
CMD [ "./make_graphs.py" ]