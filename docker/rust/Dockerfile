FROM ubuntu:18.04

# Install Debian and Python dependencies
ARG DEBIAN_FRONTEND=noninteractive
RUN apt-get --yes update \
  && apt-get install --no-install-recommends --yes \
  bison \
  build-essential \
  clang \
  cmake \
  curl \
  doxygen \
  expect \
  flex \
  git \
  libboost-all-dev \
  libcap-dev \
  libgoogle-perftools-dev \
  libncurses5-dev \
  libsqlite3-dev \
  libssl-dev \
  libtcmalloc-minimal4 \
  perl \
  pkg-config \
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

# Create the specified user and group.
#
# Ignore errors if the user or group already exist (it should only happen if the image is being
# built as root, which happens on GCB).
RUN (groupadd --gid=${USER_GID} ${USERNAME} || true) \
  && (useradd --shell=/bin/bash --uid=${USER_UID} --gid=${USER_GID} --create-home ${USERNAME} || true)

# Set the default user as the newly created one, so that any operations performed from within the
# Docker container will appear as if performed by the outside user, instead of root.
USER ${USER_UID}
ENV HOME=/home/${USERNAME}
WORKDIR ${HOME}

# Install rustup and set default toolchain to the toolchain we built
ENV PATH="${HOME}/.cargo/bin:${PATH}"
RUN curl --location https://sh.rustup.rs > /tmp/rustup \
  && sh /tmp/rustup -y --default-toolchain=none \
  && rustup --version \
  && rm /tmp/rustup

# Install rustc and cargo
ENV RUSTC_DIR=${HOME}/rust
ARG RUSTFLAGS_STAGE_NOT_0="-Cembed-bitcode=yes"
RUN git clone https://github.com/rust-lang/rust.git
RUN cd ${RUSTC_DIR} \
  && git checkout 1.46.0

# optionally, fetch the submodules first so that they get into the Docker cache
# RUN git submodule update --init --recursive

RUN cd ${RUSTC_DIR} \
  && cp config.toml.example config.toml \
  && sed -i 's/^#targets.*/targets = "X86"/' config.toml \
  && grep targets config.toml \
  && python3 ./x.py build --stage 1 \
  && python3 ./x.py build --stage 1 src/tools/cargo

# Set our rustc as default compiler and put tools on path
RUN rustup toolchain link stage1 ${RUSTC_DIR}/build/x86*/stage1 \
  && rustup default stage1
ENV PATH="${RUSTC_DIR}/build/x86_64-unknown-linux-gnu/stage1-tools-bin:${PATH}"
ENV PATH="${RUSTC_DIR}/build/x86_64-unknown-linux-gnu/llvm/bin:${PATH}"
