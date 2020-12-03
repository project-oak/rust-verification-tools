FROM rvt_base:latest

USER ${USERNAME}
WORKDIR ${USER_HOME}

# Fetch rustc
ENV RUSTC_DIR=${USER_HOME}/rust
ARG RUSTFLAGS_STAGE_NOT_0="-Cembed-bitcode=yes"
ARG RUSTC_VERSION
RUN git clone --no-checkout https://github.com/rust-lang/rust.git ${RUSTC_DIR}
RUN cd ${RUSTC_DIR} \
  && git checkout ${RUSTC_VERSION} \
  && git submodule update --jobs 16 --init --recursive

# Configure
RUN cd ${RUSTC_DIR} \
  && cp config.toml.example config.toml \
  && sed -i \
     -e 's/^#targets = .*/targets = "X86"/' \
#     -e 's/^#release-debuginfo = .*/release-debuginfo = true/' \
     -e 's/^#ninja = .*/ninja = true/' \
     config.toml

# Build llvm tools, rustc, lib{core,std,...} and cargo
RUN cd ${RUSTC_DIR} && python3 ./x.py build --incremental --stage 1 llvm \
  && cd ${RUSTC_DIR} && python3 ./x.py build --incremental --stage 1 \
  && cd ${RUSTC_DIR} && python3 ./x.py build --incremental --stage 1 src/tools/cargo

ENV PATH="${RUSTC_DIR}/build/x86_64-unknown-linux-gnu/stage1-tools-bin:${PATH}"
ENV PATH="${RUSTC_DIR}/build/x86_64-unknown-linux-gnu/llvm/bin:${PATH}"
