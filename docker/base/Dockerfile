FROM ubuntu:18.04

# Install Debian and Python dependencies
ARG DEBIAN_FRONTEND=noninteractive
RUN apt-get --yes update \
  && apt-get install --no-install-recommends --yes \
  bison \
  build-essential \
  clang-10 \
  clang-tools-10 \
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
  lld-10 \
  llvm-10 \
  ninja-build \
  perl \
  pkg-config \
  python3 \
  python3-minimal \
  python3-pip \
  sudo \
  unzip \
  wget \
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
ENV USER_HOME=/home/${USERNAME}

# Create the specified user and group and add them to sudoers list
#
# Ignore errors if the user or group already exist (it should only happen if the image is being
# built as root, which happens on GCB).
RUN (groupadd --gid=${USER_GID} ${USERNAME} || true) \
  && (useradd --shell=/bin/bash --uid=${USER_UID} --gid=${USER_GID} --create-home ${USERNAME} || true) \
  && echo "${USERNAME}  ALL=(ALL) NOPASSWD: ALL" >> /etc/sudoers

