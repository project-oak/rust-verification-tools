# import this library with:
#     source $(dirname "$(realpath -s "$0")")/sudo_if_needed.bash

function sudo_if_needed() {
    if [[ -w /var/run/docker.sock ]]; then
        "$@"
    else
        echo "Running docker with sudo because you don't have write access to the docker socket."
        echo "Add yourself to the docker group to avoid the need for this in future."
        sudo "$@"
    fi
}
