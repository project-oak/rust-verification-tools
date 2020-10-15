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
  python3 \
  python3-minimal \
  python3-pip \
  unzip \
  # Cleanup
  && apt-get clean \
  # Install Python packages
  && pip3 install --no-cache-dir setuptools \
  && pip3 install --no-cache-dir \
    argparse \
    colored \
    lit \
    tabulate \
    termcolor \
    toml \
    wllvm


# Placeholder args that are expected to be passed in at image build time.
# See https://code.visualstudio.com/docs/remote/containers-advanced#_creating-a-nonroot-user
ARG USERNAME=user-name-goes-here
ARG USER_UID=1000
ARG USER_GID=${USER_UID}
ARG HOME=/home/${USERNAME}


RUN mkdir $HOME/klee
COPY build_minisat /root/klee/
RUN cd $HOME/klee && chmod +x build_minisat && ./build_minisat
COPY build_stp /root/klee/
RUN cd $HOME/klee && chmod +x build_stp && ./build_stp
COPY build_googletest /root/klee/
RUN cd $HOME/klee && chmod +x build_googletest &&./build_googletest
COPY build_klee /root/klee/
RUN cd $HOME/klee && export PATH=$PATH:`echo /rust/build/x86_*/llvm/bin` && chmod +x build_klee && ./build_klee 
