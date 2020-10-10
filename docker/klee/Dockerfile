FROM rust_compiler_dev:v3.0

RUN apt-get install -y cmake bison flex libboost-all-dev python perl
RUN apt-get install -y build-essential curl libcap-dev git cmake libncurses5-dev
RUN apt-get install -y python-minimal python3-pip unzip libtcmalloc-minimal4 libgoogle-perftools-dev
RUN apt-get install -y libsqlite3-dev doxygen clang
RUN pip3 install tabulate wllvm lit toml colored
RUN mkdir $HOME/klee
COPY build_minisat /root/klee/
RUN cd $HOME/klee && chmod +x build_minisat && ./build_minisat
COPY build_stp /root/klee/
RUN cd $HOME/klee && chmod +x build_stp && ./build_stp
COPY build_googletest /root/klee/
RUN cd $HOME/klee && chmod +x build_googletest &&./build_googletest
COPY build_klee /root/klee/
RUN cd $HOME/klee && export PATH=$PATH:`echo /rust/build/x86_*/llvm/bin` && chmod +x build_klee && ./build_klee 
