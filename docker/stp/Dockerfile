FROM rvt_minisat:latest

USER root
RUN mkdir /home/stp
WORKDIR /home/stp

ARG STP_VERSION
RUN git clone --no-checkout https://github.com/stp/stp.git \
  && cd stp \
  && git checkout tags/${STP_VERSION} \
  && mkdir build \
  && cd build \
  && cmake .. \
  && make -j4 \
  && make install
