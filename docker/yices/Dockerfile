FROM rvt_z3:latest

USER root
RUN mkdir /home/yices
WORKDIR /home/yices

ARG YICES_VERSION
RUN curl --location https://yices.csl.sri.com/releases/${YICES_VERSION}/yices-${YICES_VERSION}-x86_64-pc-linux-gnu-static-gmp.tar.gz > yices.tgz \
  && tar xf yices.tgz \
  && rm yices.tgz \
  && cd "yices-${YICES_VERSION}" \
  && ./install-yices
