FROM rvt_stp:latest

USER root

# Install python2 - because uclibc needs it
ARG DEBIAN_FRONTEND=noninteractive
RUN apt-get --yes update \
  && apt-get install --no-install-recommends --yes \
  python \
  # Cleanup
  && apt-get clean

USER root
RUN mkdir /home/klee
WORKDIR /home/klee

ARG GTEST_VERSION
COPY build_googletest .
RUN sh build_googletest

ARG KLEE_VERSION
COPY build_klee .
RUN sh build_klee
