FROM rvt_yices:latest

USER root
RUN mkdir /home/seahorn
WORKDIR /home/seahorn

ARG VERIFY_COMMON_VERSION
ARG SEAHORN_VERSION
ARG YICES_VERSION

RUN git clone https://github.com/yvizel/verify-c-common.git \
  && cd verify-c-common \
  && git checkout ${VERIFY_COMMON_VERSION}

RUN git clone https://github.com/seahorn/seahorn.git \
  && cd seahorn \
  && git checkout ${SEAHORN_VERSION}

RUN mkdir seahorn/build \
  && cd seahorn/build \
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
