FROM rvt_base:latest

ARG USERNAME

USER ${USERNAME}
WORKDIR ${USER_HOME}

# Fetch rustc
ENV RUSTC_DIR=${USER_HOME}/rust

# We turn off SSE2 to avoid having to deal with a large number of x86-specific vector instructions.
# In particular, a lot of hand-vectorized code in major libraries tests whether SSE2 (or AVX2)
# is available and disabling SSE2 is enough to make the code fall back to a more generic version
# of the code.
# But, turning SSE2 off with an x86 compiler makes it fall back on the x87 FPU (which is
# possibly worse than SSE2) so, to compensate, we turn on soft-float.
ARG RUSTFLAGS_NOT_BOOTSTRAP="-Cembed-bitcode=yes -Csoft-float=yes -Ctarget-feature=-mmx,-sse,-sse2,-sse3,-ssse3,-sse4.1,-sse4.2,-3dnow,-3dnowa,-avx,-avx2"

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
