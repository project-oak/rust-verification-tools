FROM rust_klee:latest

# Placeholder args that are expected to be passed in at image build time.
# See https://code.visualstudio.com/docs/remote/containers-advanced#_creating-a-nonroot-user
ARG USERNAME=user-name-goes-here
ARG USER_UID=1000
ARG USER_GID=${USER_UID}
ARG HOME=/home/${USERNAME}

ARG RUSTC_DIR=${HOME}/rust

# Copy the rust-verification-tools repo into this image.
# (An alternative would be to mount the directory when running the image.)
USER root
ENV RVT_DIR=/home/rust-verification-tools
RUN mkdir ${RVT_DIR} \
  && chown ${USER_UID}:${USER_GID} ${RVT_DIR}
COPY --chown=${USER_UID}:${USER_GID} . ${RVT_DIR}

# Switch to USERNAME and install tools / set environment
USER ${USERNAME}
WORKDIR ${HOME}

ENV PATH="${RUSTC_DIR}/build/x86_64-unknown-linux-gnu/stage1-tools-bin:${PATH}"
ENV PATH="${RUSTC_DIR}/build/x86_64-unknown-linux-gnu/llvm/bin:${PATH}"
ENV PATH="$PATH:${HOME}/.cargo/bin"
ENV PATH="$PATH:${RVT_DIR}/scripts"

# Install the toolchain that we built
RUN rustup toolchain link stage1 ${RUSTC_DIR}/build/x86*/stage1 \
  && rustup default stage1


RUN echo "export PATH=\"${PATH}\"" >> ${HOME}/.bashrc
