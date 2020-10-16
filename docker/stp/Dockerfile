FROM rvt_minisat:latest

USER root
RUN mkdir /home/stp
WORKDIR /home/stp

COPY build_stp ./
RUN sh build_stp
