FROM rvt_z3:latest

USER root

RUN echo ${Z3_DIR} && ls ${Z3_DIR}

# Install Debian and Python dependencies
ARG DEBIAN_FRONTEND=noninteractive
RUN apt-get --yes update \
  && apt-get install --no-install-recommends --yes \
  build-essential \
  clang-10 \
  clang-format-10 \
  clang-tools-10 \
  cmake \
  g++-7-multilib \
  gcc-multilib \
  git \
  lib32stdc++-7-dev \
  libboost-all-dev \
  libgmp-dev \
  libgmpxx4ldbl \
  libncurses5-dev \
  lld-10 \
  llvm-10 \
  ncurses-doc \
  ninja-build \
  subversion \
  # Cleanup
  && apt-get clean

WORKDIR /opt

ARG YICES_VERSION=2.6.2
RUN curl --location https://yices.csl.sri.com/releases/${YICES_VERSION}/yices-${YICES_VERSION}-x86_64-pc-linux-gnu-static-gmp.tar.gz > yices.tgz \
  && tar xf yices.tgz \
  && rm yices.tgz

RUN mkdir /root/seahorn
WORKDIR /root/seahorn

RUN git clone https://github.com/yvizel/verify-c-common.git
RUN git clone --branch=dev10 https://github.com/seahorn/seahorn.git

RUN mkdir seahorn/build \
  && cd seahorn/build \
  && cmake \
     -DCMAKE_INSTALL_PREFIX=run \
     # -DCMAKE_BUILD_TYPE="Debug" \
     -DCMAKE_BUILD_TYPE="Release" \
     -DCMAKE_CXX_COMPILER="clang++-10" \
     -DCMAKE_C_COMPILER="clang-10" \
     -DZ3_ROOT=${Z3_DIR} \
     -DYICES2_HOME=${YICES_DIR} \
     -DSEA_ENABLE_LLD="ON" \
     -GNinja \
     -DCMAKE_EXPORT_COMPILE_COMMANDS=1 \
     -DLLVM_DIR=${RUSTC_DIR}/build/x86_64-unknown-linux-gnu/llvm/lib/cmake/llvm \
     ..

RUN cd seahorn/build && cmake --build . --target extra  && cmake ..
RUN cd seahorn/build && cmake --build . --target crab  && cmake ..
RUN cd seahorn/build && cmake --build . --target install
# RUN cd seahorn/build && cmake --build . --target units_z3
# RUN cd seahorn/build && cmake --build . --target units_yices2
# RUN cd seahorn/build && cmake --build . --target package
