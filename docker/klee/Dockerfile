FROM rust_compiler_dev:latest

ARG DEBIAN_FRONTEND=noninteractive
RUN apt-get --yes update \
  && apt-get install --no-install-recommends --yes \
  bison \
  build-essential \
  clang \
  cmake \
  curl \
  doxygen \
  flex \
  git \
  libboost-all-dev \
  libcap-dev \
  libgoogle-perftools-dev \
  libncurses5-dev \
  libsqlite3-dev \
  libtcmalloc-minimal4 \
  perl \
  python \
  python-minimal \
  python3-pip \
  unzip \
  # Cleanup
  && apt-get clean \
  # Install Python packages
  && pip3 install --no-cache-dir setuptools \
  && pip3 install --no-cache-dir \
    colored \
    lit \
    tabulate \
    termcolor \
    toml \
    wllvm

RUN mkdir $HOME/klee
COPY build_minisat /root/klee/
RUN cd $HOME/klee && chmod +x build_minisat && ./build_minisat
COPY build_stp /root/klee/
RUN cd $HOME/klee && chmod +x build_stp && ./build_stp
COPY build_googletest /root/klee/
RUN cd $HOME/klee && chmod +x build_googletest &&./build_googletest
COPY build_klee /root/klee/
RUN cd $HOME/klee && export PATH=$PATH:`echo /rust/build/x86_*/llvm/bin` && chmod +x build_klee && ./build_klee 
