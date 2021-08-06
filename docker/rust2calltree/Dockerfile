ARG FROM_IMAGE_FOR_RVT_R2CT
FROM ${FROM_IMAGE_FOR_RVT_R2CT}

# Placeholder args that are expected to be passed in at image build time.
# See https://code.visualstudio.com/docs/remote/containers-advanced#_creating-a-nonroot-user
ARG USERNAME=user-name-goes-here
ARG USER_UID=1000
ARG USER_GID=${USER_UID}

# Install more packages
# We don't install this in base because they are HUGE and optional
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
