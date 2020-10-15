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


#install rushup
RUN curl --proto '=https' --tlsv1.2 https://sh.rustup.rs -sSf > $HOME/rustup.sh
COPY install_rushup /root/install_rushup
RUN cd $HOME && chmod +x install_rushup && chmod +x rustup.sh && ./install_rushup
#install rustc
RUN cd $HOME && git clone https://github.com/rust-lang/rust.git
RUN cd $HOME/rust && git checkout 1.46.0 && cp config.toml.example config.toml && sed -i 's/AArch64;ARM;Hexagon;MSP430;Mips;NVPTX;PowerPC;RISCV;Sparc;SystemZ;WebAssembly;X86/X86/g' config.toml && RUSTFLAGS_STAGE_NOT_0="-Cembed-bitcode=yes" && ./x.py build
#install cargo
RUN apt install libssl-dev pkg-config -y
RUN cd $HOME/rust && RUSTFLAGS_STAGE_NOT_0="-Cembed-bitcode=yes" && ./x.py build src/tools/cargo
#set path variable
RUN echo "export PATH="\$PATH:\$HOME/rust/build/x86_64-unknown-linux-gnu/stage2-tools-bin"" >> $HOME/.bashrc
RUN echo "export PATH="\$PATH:\$HOME/.cargo/bin"" >> $HOME/.bashrc
RUN export PATH="$PATH:$HOME/rust/build/x86_64-unknown-linux-gnu/stage2-tools-bin:$HOME/.cargo/bin" && cd $HOME/rust && rustup toolchain link stage2 build/x86*/stage2 && rustup default stage2
