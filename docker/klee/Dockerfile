FROM rvt_stp:latest

USER root
RUN mkdir /root/klee
WORKDIR /root/klee

COPY build_googletest .
RUN sh build_googletest

COPY build_klee .
RUN sh build_klee
