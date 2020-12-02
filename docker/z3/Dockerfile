FROM rvt_klee:latest

USER root
RUN mkdir /home/z3
WORKDIR /home/z3

# Install the binary version of Z3.
# (Changing this to build from source would be fine - but slow)
#
# The Ubuntu version is a little out of date but that doesn't seem to cause any problems
ARG Z3_VERSION
RUN curl --location https://github.com/Z3Prover/z3/releases/download/z3-${Z3_VERSION}/z3-${Z3_VERSION}-x64-ubuntu-16.04.zip > z3.zip \
  && unzip -q z3.zip \
  && rm z3.zip

ENV Z3_DIR=/home/z3/z3-${Z3_VERSION}-x64-ubuntu-16.04
