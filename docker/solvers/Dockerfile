ARG FROM_IMAGE_FOR_SOLVERS
FROM ${FROM_IMAGE_FOR_SOLVERS}

ARG USERNAME
USER ${USERNAME}

# Install minisat solver

RUN mkdir ${USER_HOME}/minisat
WORKDIR ${USER_HOME}/minisat

ARG MINISAT_VERSION
RUN git clone --no-checkout https://github.com/stp/minisat.git \
  && cd minisat \
  && git checkout ${MINISAT_VERSION} \
  && git submodule init \
  && git submodule update \
  && mkdir build \
  && cd build \
  && cmake .. \
  && make -j4 \
  && sudo make install \
  && make clean

# Install stp solver

RUN mkdir ${USER_HOME}/stp
WORKDIR ${USER_HOME}/stp

ARG STP_VERSION
RUN git clone --no-checkout https://github.com/stp/stp.git \
  && cd stp \
  && git checkout tags/${STP_VERSION} \
  && mkdir build \
  && cd build \
  && cmake .. \
  && make -j4 \
  && sudo make install \
  && make clean

# Install yices solver

RUN mkdir ${USER_HOME}/yices
WORKDIR ${USER_HOME}/yices

ARG YICES_VERSION
RUN curl --location https://yices.csl.sri.com/releases/${YICES_VERSION}/yices-${YICES_VERSION}-x86_64-pc-linux-gnu-static-gmp.tar.gz > yices.tgz \
  && tar xf yices.tgz \
  && rm yices.tgz \
  && cd "yices-${YICES_VERSION}" \
  && sudo ./install-yices \
  && cd .. \
  && rm -r "yices-${YICES_VERSION}"

ENV YICES_DIR=${USER_HOME}/yices/yices-${YICES_VERSION}

# Install the binary version of Z3.
# (Changing this to build from source would be fine - but slow)
#
# The Ubuntu version is a little out of date but that doesn't seem to cause any problems

RUN mkdir ${USER_HOME}/z3
WORKDIR ${USER_HOME}/z3
ARG Z3_VERSION
RUN curl --location https://github.com/Z3Prover/z3/releases/download/z3-${Z3_VERSION}/z3-${Z3_VERSION}-x64-ubuntu-16.04.zip > z3.zip \
  && unzip -q z3.zip \
  && rm z3.zip

ENV Z3_DIR=${USER_HOME}/z3/z3-${Z3_VERSION}-x64-ubuntu-16.04
