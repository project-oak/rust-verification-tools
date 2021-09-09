ARG FROM_IMAGE_FOR_SMACK
FROM ${FROM_IMAGE_FOR_SMACK}

USER root
RUN apt-get update && \
      apt-get -y install \
      software-properties-common \
      wget \
      sudo \
      g++

# get repo
USER ${USERNAME}
WORKDIR ${USER_HOME}

ENV SMACK_DIR=${USER_HOME}/smack

RUN git clone --no-checkout https://github.com/smackers/smack.git ${SMACK_DIR} \
  && cd ${SMACK_DIR} \
  && git checkout develop

# build and install smack

ENV INSTALL_Z3=0
ENV TEST_SMACK=0
ENV INSTALL_RUST=0

RUN cd ${SMACK_DIR} && bin/build.sh --prefix ${SMACK_DIR}/smack-install

RUN echo "source ${USER_HOME}/smack.environment" >> ${USER_HOME}/.bashrc
