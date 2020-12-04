FROM rvt_stp:latest

# Temporary patch until we update base
# Install python2 - because uclibc needs it
USER root
ARG DEBIAN_FRONTEND=noninteractive
RUN apt-get --yes update \
  && apt-get install --no-install-recommends --yes \
  python \
  # Cleanup
  && apt-get clean

ARG USERNAME

WORKDIR ${USER_HOME}
COPY build_googletest .
COPY build_klee .
RUN chown ${USERNAME} -R build_googletest build_klee

USER ${USERNAME}
WORKDIR ${USER_HOME}

ARG GTEST_VERSION
ENV GTEST_DIR=${USER_HOME}/googletest-release-${GTEST_VERSION}
RUN sh build_googletest

ARG KLEE_VERSION
RUN sh build_klee
