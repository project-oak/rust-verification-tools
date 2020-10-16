FROM rvt_rustc:latest

USER root
RUN mkdir /home/minisat
WORKDIR /home/minisat

COPY build_minisat ./
RUN sh build_minisat
