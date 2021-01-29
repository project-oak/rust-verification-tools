#!/usr/bin/env bash

# Most of the script is boilerplate for processing command-line arguments (see
# usage below).
# For the interesting part jump to the 'main' function.

# Debug
# set -x
set -e

SCRIPTDIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" >/dev/null 2>&1 && pwd )"

usage()
{
    cat <<ENDUSAGE
Usage: $CMDNAME [OPTION]... [--] [SEA_OPTION]...
Descritopn...

-h, --help                  Display this message.
-c, --crate CRATE           Use cargo to generate bitcode; do some preprocessing; run SH.
-t. --test TEST
-f, --features FS           Passed to cargo.
-n, --no-lto                Don't add '-Clto' to RUSTFLAGS.
-i, --input FILE            Run SH on FILE.
-m, --mode MODE
ENDUSAGE
}

inodeof()
{
    [ -z "$1" ] || stat -c '%i' "$1"
}

parse_cmd()
{
    CMDNAME="${0##*/}"
    if [ "$(inodeof "$(command -v "$CMDNAME")")" != "$(inodeof "$0")" ] ; then
        CMDNAME="$0"
    fi

    if ! args=$(getopt -o 'hc:t:f:ni:m:' -l 'help,crate:,test:,features:,no-lto,input:,mode:' --name "${CMDNAME}" -- "$@"); then
        usage >&2
        exit 2
    fi
    # Note the quotes around "$args": they are essential!
    eval set -- "$args"
    unset args


    while [ $# -gt 0 ] ; do
        case "$1" in
            '-h'|'--help')
                shift
                usage
                exit 0
                ;;
            '-c'|'--crate')
                readonly CRATE="$2"
                shift 2
                ;;
            '-t'|'--test')
                readonly TEST="$2"
                shift 2
                ;;
            '-f'|'--features')
                readonly FEATURES="$2"
                shift 2
                ;;
            '-n'|'--no-lto')
                shift
                LTO=false
                ;;
            '-i'|'--input')
                INPUT="$2"
                shift 2
                ;;
            '-m'|'--mode')
                MODE="$2"
                shift 2
                ;;
            '--')
                shift
                break
                ;;
            *)
                echo "${CMDNAME}: internal error!" >&2
                exit 1
                ;;
        esac
    done

    SEAFLAGS=("$@")

    # Default values:
    : "${LTO:=true}"
    : "${MODE:=ybpf}"

    : "${VCC:="${SCRIPTDIR}/../../verify-c-common"}"
    : "${TEMPDIR:=sea-temp}"
}


# Extract the file name from $1, without leading path and suffix (if any).
# E.g. 'bname path/to/foo.bar' will echo 'foo'.
bname()
{
    # remove prefix
    res="${1##*/}"
    # remove suffix
    res="${res%.*}"
    echo "$res"
}

# Expects rvt-patch-llvm to be installed (i.e. 'cargo install --path .')
RVTPATCHLLVM=rvt-patch-llvm
# As the above takes 5 minutes to build, use the existing debug build instead
# RVTPATCHLLVM="${SCRIPTDIR}/../rvt-patch-llvm/target/debug/rvt-patch-llvm"

pp_rvt-patch-llvm()
{
    if [[ ! "${INPUT}" =~ \.rvt\.(.*\.)*ll$ ]]; then
        output="${TEMPDIR}/$(bname "${INPUT}").rvt.ll"
        { ENTRY="$("${RVTPATCHLLVM}" -vv -s ${TEST:+-t "${TEST}"} -o "${output}" "${INPUT}" | tee /dev/fd/3 | sed -n 's/^ENTRY: //p')"; } 3>&1
        SEAFLAGS=(--entry="${ENTRY}" "${SEAFLAGS[@]}")
        INPUT="${output}"
    fi
}

CARGO=(cargo)
# CARGO=(cargo +stage2-for-seahorn -v)

main()
{
    # Use 'cargo' to generate bitcode
    if [[ -n "$CRATE" ]]; then
        # You might need to do `cargo clean`
        rm -f "target/debug/deps/${CRATE}-"*

        # Generate bitcode
        RUSTFLAGS="-Cembed-bitcode=yes --emit=llvm-bc ${RUSTFLAGS}"
        # Check overflow
        RUSTFLAGS="-Warithmetic-overflow -Coverflow-checks=yes ${RUSTFLAGS}"
        # Abort, instead of unwind
        RUSTFLAGS="-Zpanic_abort_tests -Cpanic=abort ${RUSTFLAGS}"
        if [[ "${LTO}" == "true" ]] ; then
            # Enable link time optimizations (for us it means generate a big .bc
            # file with all the code).
            RUSTFLAGS="-Clto ${RUSTFLAGS}"
        fi

        export RUSTFLAGS
        if [[ -n "${TEST}" ]]; then
            "${CARGO[@]}" test --no-run ${FEATURES:+--features "$FEATURES"}
        else
            "${CARGO[@]}" build ${FEATURES:+--features "$FEATURES"}
        fi

        # Find the bitcode that was just generated
        INPUT="$(ls -1t "target/debug/deps/${CRATE}-"*.bc | head -1)"
    fi

    mkdir -p "$TEMPDIR"

    pp_rvt-patch-llvm

    # Run SeaHorn
    case "${MODE}" in
        'ybpf')
            sea yama -y "$VCC/seahorn/sea_base.yaml" bpf "${INPUT}" --temp-dir "${TEMPDIR}" "${SEAFLAGS[@]}"
            ;;
        *)
            sea "${MODE}" "${INPUT}" --temp-dir "${TEMPDIR}" "${SEAFLAGS[@]}"
            ;;
    esac
}

parse_cmd "$@"
main
