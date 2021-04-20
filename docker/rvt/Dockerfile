FROM rvt_seahorn:latest

# Placeholder args that are expected to be passed in at image build time.
# See https://code.visualstudio.com/docs/remote/containers-advanced#_creating-a-nonroot-user
ARG USERNAME=user-name-goes-here
ARG USER_UID=1000
ARG USER_GID=${USER_UID}

# Switch to USERNAME and install tools / set environment
USER ${USERNAME}
WORKDIR ${USER_HOME}
ENV USER=${USERNAME}

ENV PATH="${PATH}:${USER_HOME}/bin"
ENV PATH="${PATH}:${USER_HOME}/.cargo/bin"

# Install rustup
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y

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

# Prebuild all the tools and libraries
# Note that we can't mount RVT_DIR while we do this - so we have to make
# a copy and build from there. (Which is pretty hacky)
ENV RVT_DIR=${USER_HOME}/rvt_dir
RUN mkdir ${RVT_DIR}
COPY --chown=${USER_UID}:${USER_GID} . ${RVT_DIR}
WORKDIR ${RVT_DIR}
RUN ${RVT_DIR}/docker/init
RUN rm -r ${RVT_DIR}

# Directory we mount RVT repo in
# Note that this overrides value we just built
ENV RVT_DIR=/home/rust-verification-tools

ENV PATH="${PATH}:${RVT_DIR}/scripts"
ENV PATH="${PATH}:${RVT_DIR}/scripts/bin"

# Create a bashrc file
RUN echo "export PATH=\"${PATH}\"" >> ${USER_HOME}/.bashrc \
  && echo "ulimit -c0" >> ${USER_HOME}/.bashrc
