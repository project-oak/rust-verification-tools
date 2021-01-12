FROM rvt_seahorn:latest

# Placeholder args that are expected to be passed in at image build time.
# See https://code.visualstudio.com/docs/remote/containers-advanced#_creating-a-nonroot-user
ARG USERNAME=user-name-goes-here
ARG USER_UID=1000
ARG USER_GID=${USER_UID}

# Directory we mount RVT repo in
ENV RVT_DIR=/home/rust-verification-tools

# The following replicates code in base_Dockerfile and
# can be deleted next time I do a full rebuild of the images
USER root
RUN apt-get install -y sudo libffi-dev
RUN echo "${USERNAME}  ALL=(ALL) NOPASSWD: ALL" >> /etc/sudoers

# Switch to USERNAME and install tools / set environment
USER ${USERNAME}
WORKDIR ${USER_HOME}
ENV USER=${USERNAME}

# Install rustup
ENV PATH="${USER_HOME}/.cargo/bin:${PATH}"
RUN curl --location https://sh.rustup.rs > /tmp/rustup \
  && sh /tmp/rustup -y --default-toolchain=none \
  && rustup --version \
  && rm /tmp/rustup

ENV PATH="${RUSTC_DIR}/build/x86_64-unknown-linux-gnu/stage1-tools-bin:${PATH}"
ENV PATH="${RUSTC_DIR}/build/x86_64-unknown-linux-gnu/llvm/bin:${PATH}"
ENV PATH="$PATH:${USER_HOME}/.cargo/bin"
ENV PATH="$PATH:${RVT_DIR}/scripts"
ENV PATH="$PATH:${RVT_DIR}/scripts/bin"
ENV PATH="$PATH:${USER_HOME}/bin"

# Install the toolchain that we built
RUN rustup toolchain link stage1 ${RUSTC_DIR}/build/x86*/stage1 \
  && rustup default stage1

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
RUN cargo install --root=${USER_HOME} --path=${USER_HOME}/rvt-patch-llvm

# Create a bashrc file
RUN echo "export PATH=\"${PATH}\"" >> ${USER_HOME}/.bashrc
