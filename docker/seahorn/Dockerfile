FROM rvt_yices:latest

USER root

RUN pip3 install pyyaml

RUN mkdir /home/seahorn
WORKDIR /home/seahorn

ARG VERIFY_COMMON_VERSION
ARG SEAHORN_VERSION
ARG YICES_VERSION

# cargo-verify relies on this variable to find the yaml files
ENV VERIFY_COMMON_DIR=/home/seahorn/verify-c-common

RUN git clone --no-checkout https://github.com/seahorn/verify-c-common.git ${VERIFY_COMMON_DIR} \
  && cd ${VERIFY_COMMON_DIR} \
  && git checkout ${VERIFY_COMMON_VERSION}

ENV SEAHORN_DIR=/home/seahorn/seahorn

RUN git clone --no-checkout https://github.com/seahorn/seahorn.git ${SEAHORN_DIR} \
  && cd ${SEAHORN_DIR} \
  && git checkout ${SEAHORN_VERSION}

RUN mkdir ${SEAHORN_DIR}/build \
  && cd ${SEAHORN_DIR}/build \
  && cmake \
     -DCMAKE_INSTALL_PREFIX=run \
     # -DCMAKE_BUILD_TYPE="Debug" \
     -DCMAKE_BUILD_TYPE="Release" \
     -DCMAKE_CXX_COMPILER="clang++-10" \
     -DCMAKE_C_COMPILER="clang-10" \
     -DZ3_ROOT=${Z3_DIR} \
     -DYICES2_HOME="/home/yices/yices-${YICES_VERSION}" \
     -DSEA_ENABLE_LLD="ON" \
     -GNinja \
     -DCMAKE_EXPORT_COMPILE_COMMANDS=1 \
     -DLLVM_DIR=${RUSTC_DIR}/build/x86_64-unknown-linux-gnu/llvm/lib/cmake/llvm \
     ..

RUN cd seahorn/build && cmake --build . --target extra  && cmake ..
RUN cd seahorn/build && cmake --build . --target crab  && cmake ..
RUN cd seahorn/build && cmake --build . --target install

ENV PATH="${SEAHORN_DIR}/build/run/bin:$PATH"
