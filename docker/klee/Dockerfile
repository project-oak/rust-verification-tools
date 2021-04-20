FROM rvt_solvers:latest

ARG USERNAME

USER root
WORKDIR ${USER_HOME}
COPY build_googletest .
COPY build_klee .
RUN chown ${USERNAME} -R build_googletest build_klee

USER ${USERNAME}
WORKDIR ${USER_HOME}

ARG GTEST_VERSION
ENV GTEST_DIR=${USER_HOME}/googletest-release-${GTEST_VERSION}
RUN sh build_googletest

ARG UCLIBC_VERSION

ARG LLVM_VERSION
ENV LLVM_VERSION=${LLVM_VERSION}

ARG KLEE_VERSION
RUN sh build_klee
