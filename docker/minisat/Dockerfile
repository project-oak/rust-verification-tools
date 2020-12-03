FROM rvt_rustc:latest

USER root
RUN mkdir /home/minisat
WORKDIR /home/minisat

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
  && make install
