# Docker images

These scripts provide a convenient, portable way to build the Rust verification tools and their dependencies.

At present, only the KLEE backend is supported.

Todo: at the moment, the scripts use 'sudo' when invoking docker.
This is necessary on some platforms but not necessary on other platforms.
It is not clear whether there is any value in avoiding the use of sudo.


## Building docker images

The script `docker/build` builds the following images

- `rust_compiler_dev:latest` contains the Rust compiler
- `rust_klee:latest` adds KLEE and its dependencies
- `rvt:latest` adds a snapshot of the Rust verification tools

In practice, the only image that is useful is `rvt`.
The other images exist only to make it faster to rebuild rvt
It takes several hours to build the `rust_compiler_dev` image.

The `docker/build` script should be invoked in the top directory of `rust-verification-tools`.
The script will invoke `sudo` so you may be asked for your password.

Building the images creates an unprivileged user with the same username, uid and gid as the user that
ran `docker/build`.

No attempt has been made to reduce the size of the images – they are each around 10GB.


## Using docker images

The `rvt` image can be invoked using the script `docker/run` script.
This script:

- Mounts the current directory as read/write so that `rvt` can access
  any files in the current directory or its subdirectories.
  Parent directories are not accessible.

- The image will run with the permissions of the user that created the image.
  It is expected that this will be the current user.

- The PATH contains the Rust compiler, LLVM, Cargo and the Rust-verification-tools script `cargo-verify`.

- The `rvt` image contains a copy of `rust-verification-tools` in `/home/rust-verification-tools`.

  The dependencies in `Cargo.toml` files typically contain lines like

  ```
  verification-annotations = { path="/home/rust-verification-tools/verification-annotations" }
  propverify               = { path="/home/rust-verification-tools/propverify" }
  ```

- If any arguments are provided, they will be treated as commands to run in `rvt`.
  Otherwise, the `bash` shell will be run and `exit` can be used to exit docker.

