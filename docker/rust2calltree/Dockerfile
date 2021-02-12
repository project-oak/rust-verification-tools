FROM rvt:latest

# Placeholder args that are expected to be passed in at image build time.
# See https://code.visualstudio.com/docs/remote/containers-advanced#_creating-a-nonroot-user
ARG USERNAME=user-name-goes-here
ARG USER_UID=1000
ARG USER_GID=${USER_UID}

# Install more packages
USER root
RUN apt-get --yes update \
  && apt-get install --no-install-recommends --yes \
  dbus-x11 \
  kcachegrind \
  # Cleanup
  && apt-get clean

# Switch back to unprivileged user
USER ${USERNAME}
WORKDIR ${USER_HOME}
ENV USER=${USERNAME}

COPY --chown=${USER_UID}:${USER_GID} ./rust2calltree ${USER_HOME}/rust2calltree
RUN cargo install --root=${USER_HOME} --path=${USER_HOME}/rust2calltree
