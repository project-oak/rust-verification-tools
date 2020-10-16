FROM rvt_base:latest

USER ${USERNAME}
WORKDIR ${USER_HOME}

# Fetch rustc
ENV RUSTC_DIR=${USER_HOME}/rust
ARG RUSTFLAGS_STAGE_NOT_0="-Cembed-bitcode=yes"
RUN git clone https://github.com/rust-lang/rust.git
RUN cd ${RUSTC_DIR} \
  && git checkout 1.46.0

# optionally, fetch the submodules first so that they get into the Docker cache
# RUN git submodule update --init --recursive

# Build rustc, llvm tools, lib{core,std,...} and cargo
RUN cd ${RUSTC_DIR} \
  && cp config.toml.example config.toml \
  && sed -i 's/^#targets.*/targets = "X86"/' config.toml \
  && grep targets config.toml \
  && python3 ./x.py build --stage 1 \
  && python3 ./x.py build --stage 1 src/tools/cargo

ENV PATH="${RUSTC_DIR}/build/x86_64-unknown-linux-gnu/stage1-tools-bin:${PATH}"
ENV PATH="${RUSTC_DIR}/build/x86_64-unknown-linux-gnu/llvm/bin:${PATH}"
