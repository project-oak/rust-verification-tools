FROM rvt_yices:latest

ARG USERNAME

# Switch to USERNAME and install tools / set environment
USER ${USERNAME}
WORKDIR ${USER_HOME}
ENV USER=${USERNAME}
ENV PATH="${USER_HOME}/.cargo/bin:${PATH}"
ENV PATH="$PATH:${USER_HOME}/bin"

# Install rustup
RUN curl --location https://sh.rustup.rs > /tmp/rustup \
  && sh /tmp/rustup -y \
  && rustup --version \
  && rm /tmp/rustup
