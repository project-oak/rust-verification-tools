FROM rvt_klee:latest

USER ${USERNAME}
WORKDIR ${USER_HOME}

ARG CBMC_VERSION
ARG CBMC_VIEWER_VERSION
ARG RMC_VERSION
ARG UBUNTU_VERSION

ENV RMC_DIR=${USER_HOME}/rmc

RUN curl --location https://github.com/diffblue/cbmc/releases/download/cbmc-${CBMC_VERSION}/ubuntu-${UBUNTU_VERSION}-cbmc-${CBMC_VERSION}-Linux.deb > cbmc.deb \
    && sudo dpkg -i cbmc.deb \
    && curl --location https://github.com/awslabs/aws-viewer-for-cbmc/releases/download/viewer-${CBMC_VIEWER_VERSION}/cbmc_viewer-${CBMC_VIEWER_VERSION}-py3-none-any.whl > cbmc_viewer-${CBMC_VIEWER_VERSION}-py3-none-any.whl \
    && sudo python3 -m pip install --upgrade cbmc_viewer-${CBMC_VIEWER_VERSION}-py3-none-any.whl

RUN git clone --branch ${RMC_VERSION} https://github.com/model-checking/rmc.git

ENV PATH="/usr/local/bin:$PATH"
ENV PATH="${RMC_DIR}/scripts:$PATH"

# Hack: we need python to point to python3 but suggestions on SO failed so put
# a symlink in a directory that is on the path
RUN ln -s /usr/bin/python3 ${RMC_DIR}/scripts/python

RUN cd ${RMC_DIR} \
    && ./configure \
          --debuginfo-level-rustc=2 \
          --enable-debug \
          --set=llvm.download-ci-llvm=true \
          --set=rust.debug-assertions-std=false \
          --set=rust.deny-warnings=false \
          --set=rust.incremental=true

RUN python3 ${RMC_DIR}/x.py --help || echo It may be ok to ignore the error message that produces

RUN cd ${RMC_DIR} && python3 ./x.py build -i --stage 1 library/std

# Do not run regressions: some tests currently failing
# RUN cd ${RMC_DIR} && ./scripts/rmc-regression.sh
