FROM rust_klee:v3.0

RUN cd $HOME && git clone https://github.com/project-oak/rust-verification-tools.git
RUN cd $HOME/rust-verification-tools && export PATH="$PATH:$HOME/rust/build/x86_64-unknown-linux-gnu/stage2-tools-bin" && cargo new try_klee
COPY main.rs /root/rust-verification-tools/try_klee/src/
COPY cargo_build /root/rust-verification-tools/try_klee/
COPY cargo_tail /root/rust-verification-tools/try_klee/
COPY run_klee /root/rust-verification-tools/try_klee/
COPY klee_test /root/rust-verification-tools/try_klee/
COPY replay_input_values /root/rust-verification-tools/try_klee/
RUN cd $HOME/rust-verification-tools/try_klee && chmod +x cargo_build && chmod +x run_klee && chmod +x klee_test && chmod +x replay_input_values && cat cargo_tail >> Cargo.toml
RUN rm -f $HOME/install_rushup $HOME/rustup.sh $HOME/klee/build_googletest $HOME/klee/build_klee $HOME/klee/build_minisat $HOME/klee/build_stp $HOME/klee/release-1.7.0.zip $HOME/rust-verification-tools/try_klee/cargo_tail
CMD /bin/bash

