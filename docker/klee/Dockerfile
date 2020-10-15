FROM rust_compiler_dev:latest

# Placeholder args that are expected to be passed in at image build time.
# See https://code.visualstudio.com/docs/remote/containers-advanced#_creating-a-nonroot-user
ARG USERNAME=user-name-goes-here
ARG USER_UID=1000
ARG USER_GID=${USER_UID}
ARG HOME=/home/${USERNAME}

USER root
RUN mkdir /root/klee
WORKDIR /root/klee

COPY docker/klee/build_minisat .
RUN sh build_minisat

COPY docker/klee/build_stp .
RUN sh build_stp

COPY docker/klee/build_googletest .
RUN sh build_googletest

ARG RUSTC_DIR=${HOME}/rust
COPY docker/klee/build_klee .
RUN sh build_klee

USER ${USERNAME}
