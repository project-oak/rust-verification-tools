FROM rvt_seahorn:latest

# Placeholder args that are expected to be passed in at image build time.
# See https://code.visualstudio.com/docs/remote/containers-advanced#_creating-a-nonroot-user
ARG USERNAME=user-name-goes-here
ARG USER_UID=1000
ARG USER_GID=${USER_UID}

# Directory we mount RVT repo in
ENV RVT_DIR=/home/rust-verification-tools

# Switch to USERNAME and install tools / set environment
USER ${USERNAME}
WORKDIR ${USER_HOME}
ENV USER=${USERNAME}

ENV PATH="$PATH:${RVT_DIR}/scripts"
ENV PATH="$PATH:${RVT_DIR}/scripts/bin"

# Install the nightly toolchain - we use it to build some of our tools.
# We do not set it as the default though because we want to use the
# version of rustc and lib{core,std} that we built before
# and, in particular, we have to use a version of rustc that uses LLVM-10.
RUN rustup toolchain install nightly

# Version of rustc that our tools support
# This is the default
ARG RUSTC_VERSION
ENV RUSTC_VERSION=${RUSTC_VERSION}
RUN echo Installing ${RUSTC_VERSION}
RUN rustup toolchain install ${RUSTC_VERSION} --profile=minimal
RUN rustup default ${RUSTC_VERSION}

# Build LLVM file patcher by first copying the source into this image
# and then building it.
#
# Note that before invoking docker-build, you need to arrange that the
# source is a symlink to the actual source directory.
#
# Note too that this is installed in ~/bin but that if you explicitly rebuild
# it in a docker session, then it will go into ${RVT_DIR}/scripts/bin
#
# There seems to be no way around this complex collection of hacks.
COPY --chown=${USER_UID}:${USER_GID} ./rvt-patch-llvm ${USER_HOME}/rvt-patch-llvm
RUN cargo install --features=llvm${LLVM_VERSION} --root=${USER_HOME} --path=${USER_HOME}/rvt-patch-llvm

# Build cargo-verify too
COPY --chown=${USER_UID}:${USER_GID} ./cargo-verify ${USER_HOME}/cargo-verify
RUN cargo +nightly install --root=${USER_HOME} --path=${USER_HOME}/cargo-verify

# Create a bashrc file
RUN echo "export PATH=\"${PATH}\"" >> ${USER_HOME}/.bashrc \
  && echo "ulimit -c0" >> ${USER_HOME}/.bashrc

