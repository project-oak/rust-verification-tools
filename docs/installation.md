# Installation

Rust verification is relatively new and we are trying to use multiple
verification tools so, at least for now, these libraries have many complex dependencies.

If you want to use Crux-MIR, see [installing crux and its dependencies](install-crux.md).

All other verification tools are installed using [Docker](https://www.docker.com/) as follows

``` shell
git clone https://github.com/project-oak/rust-verification-tools.git
cd rust-verification-tools
docker/build
```

This will take several hours to build the Docker images.
The resulting docker image can be run by executing `docker/run`
which executes a bash shell using the current user in the current directory.

For more details see [the README](../docker/README.md).

If you are unable to use Docker, the best approach is to manually execute
the commands in the Dockerfile.

