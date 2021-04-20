FROM rvt_klee:latest

USER ${USERNAME}
WORKDIR ${USER_HOME}

ARG SEAHORN_VERIFY_C_COMMON_VERSION
ARG SEAHORN_VERSION
ARG LLVM_VERSION
ARG YICES_VERSION

# cargo-verify relies on this variable to find the yaml files
ENV SEAHORN_VERIFY_C_COMMON_DIR=${USER_HOME}/verify-c-common

RUN git clone --no-checkout https://github.com/seahorn/verify-c-common.git ${SEAHORN_VERIFY_C_COMMON_DIR} \
  && cd ${SEAHORN_VERIFY_C_COMMON_DIR} \
  && git checkout ${SEAHORN_VERIFY_C_COMMON_VERSION}

ENV SEAHORN_DIR=${USER_HOME}/seahorn

RUN git clone --no-checkout https://github.com/seahorn/seahorn.git ${SEAHORN_DIR} \
  && cd ${SEAHORN_DIR} \
  && git checkout ${SEAHORN_VERSION}

RUN mkdir ${SEAHORN_DIR}/build \
  && cd ${SEAHORN_DIR}/build \
  && cmake \
     -DCMAKE_INSTALL_PREFIX=run \
     # -DCMAKE_BUILD_TYPE="Debug" \
     -DCMAKE_BUILD_TYPE="Release" \
     -DCMAKE_CXX_COMPILER="clang++-${LLVM_VERSION}" \
     -DCMAKE_C_COMPILER="clang-${LLVM_VERSION}" \
     -DZ3_ROOT=${Z3_DIR} \
     -DYICES2_HOME=${YICES_DIR} \
     -DSEA_ENABLE_LLD="ON" \
     -GNinja \
     -DCMAKE_EXPORT_COMPILE_COMMANDS=1 \
     -DLLVM_DIR=/usr/lib/llvm-${LLVM_VERSION}/lib/cmake/llvm \
     ..

RUN cd seahorn/build && cmake --build . --target extra  && cmake ..
RUN cd seahorn/build && cmake --build . --target crab  && cmake ..
RUN cd seahorn/build && cmake --build . --target install

# Clean up large files
# We try to leave enough files that rebuilding is easy (in case you
# want to tweak the configuration and rebuild) while not creating
# enormous Docker images.
RUN cd seahorn/build && sudo cmake --build . --target clean && rm -r run

ENV PATH="${SEAHORN_DIR}/build/run/bin:$PATH"
