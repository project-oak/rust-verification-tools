---
layout: post
title: Using KLEE on Coreutils
---

![KLEE logo](https://klee.github.io/images/klee.svg){: style="float: left; width: 10%; padding: 1%"}
A lot of our work over the last year was on [identifying and fixing obstacles to using KLEE with Rust][KLEE status]
and the main technique we used for finding new obstacles was to try to use
[KLEE] with different Rust programs and libraries.
One of the largest suites of programs we tackled was the [Rust CoreUtils]
library: a Rust rewrite and drop in replacement for the GNU CoreUtils suite
that includes programs like ls, cp, df, cat, and about 90 other standard Unix shell
commands.

This is a brief summary of how to use [RVT][RVT git repo] and [KLEE] on the
[Rust CoreUtils].
My goal is that, by the end, you should be able to run [KLEE] on [Rust
CoreUtils]
and, hopefully, any other applications of similar complexity.
At least in this post, I am not going to try to show the best way to use KLEE or all of the things
that you can do with KLEE although I make a few suggestions at the end
if you want to explore yourself.


## Setup

The first step is to fetch RVT and build a docker image (this will take about
15-20 minutes).

``` shell
# install docker
sudo apt-get install -y docker
sudo groupadd docker
sudo usermod -aG docker $USER
# you may have to log out and back in to make the last command take effect

# install RVT and build a docker image
git clone https://github.com/project-oak/rust-verification-tools.git rvt
cd rvt
docker/build

# fetch uutils coreutils
git clone https://github.com/uutils/coreutils
```

And, from now on, let's run everything in docker.
This ensures that you are using the right version of the Rust compiler, etc.

```
docker/run
```

(Actually, I find it convenient to have two terminals open: one not using docker
that I run an editor in and one inside docker where I execute commands.)


## Getting `cargo-verify` and KLEE to run on coreutils

Let's start with the first coreutils application: `arch`

```
cd coreutils/src/uu/arch
```

Note that this is a good choice to start with because (as far as I know)
the arch command cannot harm your system.
If you go on to run 'rm' under KLEE, you should be ready for the fact that
KLEE could 'discover' and execute commands like 'rm -rf ..' – deleting data you
were wanting to keep. While running KLEE within Docker will help reduce the
damage that can be done, you will want to build a more secure sandbox to run in
before you try running all the coreutils applications in KLEE.


To use [KLEE] on a Rust crate, you need to

1. Add some extra dependencies to the Cargo.toml file
2. Use RVT's `cargo-verify` program to build the crate
3. Run KLEE on the resulting LLVM bitcode file

The extra dependencies look like this and should be added to the end of
`coreutils/src/uu/arch/Cargo.toml`.

```
[target.'cfg(not(verify))'.dependencies]
proptest = { version = "0.10" }

[target.'cfg(verify)'.dependencies]
propverify = { path="/home/rust-verification-tools/propverify" }

[features]
verifier-klee = ["propverify/verifier-klee"]
verifier-crux = ["propverify/verifier-crux"]
verifier-seahorn = ["propverify/verifier-seahorn"]
```

Note that these mention the [PropTest]
property-based testing library, the [Crux-MIR] verifier and the [SeaHorn]
verifier as well – I like to add the same text no matter what tool I am
using but you could choose to omit propverify, verifier-crux and
verifier-seahorn if you want a more minimal change.
Note too that this assumes that you are running in Docker where
we mount the Rust Verification Tools directory at 
"/home/rust-verification-tools/propverify".

Now we use RVT's `cargo-verify` tool to build the crate and generate
a bitcode file `app.bc` that is suitable for KLEE to use.

```
cargo verify -v --bin arch -o app.bc
```

The `-v` flag increases verbosity which helps reduce anxiety a little since the command takes a
minute or so to run.
The `--bin arch` flag is required because the Cargo.toml file defines both a
`[lib]` and a `[[bin]]` so we need to say that we want the bin.

Finally, we run [KLEE] on the bitcode file

```
klee --libc=klee --posix-runtime --disable-verify app.bc --sym-args 0 3 10 --sym-files 2 8
```

This produces a lot of output: warnings, output from the program being tested,
etc.
You may also have to use ctrl-C to quit KLEE: after a minute or so it doesn't
seem to produce any new output.

We can slightly reduce the noise by explicitly disabling backtraces like this

```
RUST_BACKTRACE=0 klee --libc=klee --posix-runtime --disable-verify app.bc --sym-args 0 3 10 --sym-files 2 8
```

This will eliminate all the messages about running with `RUST_BACKTRACE=1` set.

Let's look at the remaining output a little at a time.

The first three lines just tell us something about the KLEE configuration and
can be ignored.

```
KLEE: NOTE: Using POSIX model: /usr/lib/x86_64-linux-gnu/klee/runtime/libkleeRuntimePOSIX64_Debug+Asserts.bca
KLEE: output directory is "/home/rust-verification-tools/downloads/coreutils/src/uu/arch/klee-out-0"
KLEE: Using STP solver backend
```

The next 20 lines or so are warnings that the bitcode file does not define some
functions from the Rust runtime or standard libraries such as
`_Unwind_Backtrace` and 
`_ZN4core3fmt9Formatter15debug_upper_hex17h371aeb098993d32aE` (which is the
mangled version of `core::fmt::Formatter::debug_upper_hex`).
These often don't matter and, in any case, they will not prevent us from running
KLEE so ignore these as well.

After that, there are about 18 warnings that the bitcode file does not define
some system calls or C standard library functions such as `__fgetc_unlocked` or `writev`.
Some of these functions are called by the application being tested so they might
cause a problem that we will need to revisit later but, again, they will not
prevent us from running KLEE so, for now, we will ignore them.

```
KLEE: WARNING: undefined reference to function: __fgetc_unlocked
KLEE: WARNING: undefined reference to function: __fputc_unlocked
KLEE: WARNING: undefined reference to function: __rdos_backtrace_create_state
KLEE: WARNING: undefined reference to function: __rdos_backtrace_pcinfo
KLEE: WARNING: undefined reference to function: __rdos_backtrace_syminfo
KLEE: WARNING: undefined reference to function: __xpg_strerror_r
KLEE: WARNING: undefined reference to function: dladdr
KLEE: WARNING: undefined reference to function: endutent
KLEE: WARNING: undefined reference to function: getenv
KLEE: WARNING: undefined reference to function: getutent
KLEE: WARNING: undefined reference to function: mprotect
KLEE: WARNING: undefined reference to function: realpath
KLEE: WARNING: undefined reference to function: setutent
KLEE: WARNING: undefined reference to function: sigaltstack
KLEE: WARNING: undefined reference to function: signal
KLEE: WARNING: undefined reference to function: sysconf
KLEE: WARNING: undefined reference to function: utmpname
KLEE: WARNING: undefined reference to function: writev
```

At this stage, KLEE has finished loading the bitcode file and it is ready to
start symbolic execution of the program being tested.
Some of the first things that the program is going to do is
to call some of those missing functions and each time it does so, it produces a
warning message.
As you might expect by now, we are going to ignore those.

```
KLEE: WARNING ONCE: Alignment of memory from call "malloc" is not modelled.
Using alignment of 8.
KLEE: WARNING ONCE: calling external: syscall(4, 94432679386248, 94432641680800)
at runtime/POSIX/fd.c:544 12
KLEE: WARNING ONCE: calling __klee_posix_wrapped_main with extra arguments.
KLEE: WARNING ONCE: calling external: signal(13, 1) at [no debug info]
KLEE: WARNING ONCE: calling external: sysconf(30) at [no debug info]
KLEE: WARNING ONCE: sigaction: silently ignoring
KLEE: WARNING ONCE: calling external: sigaltstack(0, 94432681267360) at [no debug info]
KLEE: WARNING ONCE: Alignment of memory from call "realloc" is not modelled.
Using alignment of 8.
KLEE: WARNING ONCE: calling external: uname(94432689661152) at .cargo/registry/src/github.com-1ecc6299db9ec823/platform-info-0.1.0/src/unix.rs:36 16
KLEE: WARNING ONCE: calling external: getenv(94432682275392) at [no debug info]
```

There is then an error report about strlen.

```
KLEE: ERROR: runtime/klee-libc/strlen.c:14: memory error: out of bound pointer
KLEE: NOTE: now ignoring this error at this location
```

It appears that the Rust standard library
runs the C function `strlen` on the output of `getenv` and, for reasons that I
do not fully understand, this triggers an error.
The error does not appear if we use `--libc=uclibc` instead of `--libc=klee`
which suggests that this is a minor problem with KLEE's libc support so
we can either ignore it or switch to `--libc=uclibc`.

Most of the rest of the output is either output from the application or consists of
messages from KLEE.
(The first time that I ran this, there were also a lot of panic messages
due to finding bugs in the UTF-8 handling. Those have since been fixed.)

```
arch 0.0.7
x86_64
arch 0.0.7arch 0.0.7KLEE: WARNING ONCE: skipping fork (memory cap exceeded)

KLEE: WARNING: killing 2452 states (over memory cap: 2101MB)
arch 0.0.7arch 0.0.7arch 0.0.7
arch 0.0.7arch 0.0.7arch 0.0.7arch 0.0.7arch 0.0.7
```


## More aggressive use of KLEE

At this point, we are able to run KLEE on a coreutils application.
It runs ok but it is not reporting anything actionable so 
let's look at [KLEE documentation on testing the GNU coreutils
applications](https://klee.github.io/docs/coreutils-experiments/).

That page recommends the following command line

```
$ klee --simplify-sym-indices --write-cvcs --write-cov --output-module \
    --max-memory=1000 --disable-inlining --optimize --use-forked-solver \
    --use-cex-cache --libc=uclibc --posix-runtime \
    --external-calls=all --only-output-states-covering-new \
    --env-file=test.env --run-in-dir=/tmp/sandbox \
    --max-sym-array-size=4096 --max-solver-time=30s --max-time=60min \
    --watchdog --max-memory-inhibit=false --max-static-fork-pct=1 \
    --max-static-solve-pct=1 --max-static-cpfork-pct=1 --switch-type=internal \
    --search=random-path --search=nurs:covnew \
    --use-batching-search --batch-instructions=10000 \
    ./paste.bc \
    --sym-args 0 1 10 --sym-args 0 2 2 --sym-files 1 8 --sym-stdin 8 --sym-stdout
```

This needs a couple of minor changes to make it work with Rust.

1. Our application is called `app.bc`, not `paste.bc`

2. We need to add the `--disable-verify` flag from above and drop the
   `--optimize` flag to avoid some issues in KLEE.

3. We can omit `--use-cex-cache`, `--switch-type`, `--search` and
   `--batch-instructions` because those are the default values.
   But it's worth knowing about them if you want to try non-default settings.

4. We can omit `--write-cov` and `--write-cvcs` unless we want to look
   at the files that they generate.

5. The above code expects you to create a "sandbox" to prevent your
   application from damaging your system.
   Since we don't think `arch` can do any harm, let's skip that extra
   complication. YOLO!

This results in the following command

```
klee --disable-verify --simplify-sym-indices \
  --output-module --max-memory=1000 --disable-inlining \
  --use-forked-solver --libc=uclibc --posix-runtime \
  --external-calls=all --only-output-states-covering-new \
  --max-sym-array-size=4096 --max-solver-time=30s --max-time=60min --watchdog \
  --max-memory-inhibit=false --max-static-fork-pct=1 --max-static-solve-pct=1 \
  --max-static-cpfork-pct=1 --use-batching-search --batch-instructions=10000 \
  ./app.bc \
  --sym-args 0 1 10 --sym-args 0 2 2 --sym-files 1 8 --sym-stdin 8 --sym-stdout
```

This results in warning messages like before followed by a lot of variations
on the following output

```
error: Found argument '' which wasn't expected, or isn't valid in this context

USAGE:
    app.bc

    For more information try --help
```

Amongst this output, we will occasionally see error messages like the following

```
unexpected invalid UTF-8 code point',
/home/adreid/.cargo/registry/src/github.com-1ecc6299db9ec823/clap-2.33.3/src/app/parser.rs:1685:46
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

Looking at [line 1685 of parser.rs](https://github.com/clap-rs/clap/blob/v2.33.3/src/app/parser.rs#L1685)
we see that `expect` is being used to check that an argument was correctly
converted to a string.

```
self.did_you_mean_error(arg.to_str().expect(INVALID_UTF8), matcher, &args_rest2[..])
```

As the author clearly recognized, this will panic if the argument is not legal UTF-8.
It is a matter of taste whether this is the best way to report an error in user
input and therefore whether this is a bug or working as intended.
(A similar error is reported for line 33 of the same file.)

After running for a while (I got bored before the 60 minute timeout was
reached), it looks as though KLEE is not going to find any further problems with
the `arch` command so let's try some different commands.

(If you want to spend more time with this command, you might investigate the
contents of the `klee-last` directory. This will now contain lots of files
with names like `test000037.ktest`.
You can use `ktest-tool test000037.ktest` to look at the input for this test.
And, if you want to [see what parts of the code have (not) been hit]({{site.baseurl}}{% post_url 2021-03-12-profiling-rust %}),
you can also examine `run.istats` using `kcachegrind`
using [our Rust name demangling tool][rust2calltree].)


## Testing other commands

To test some other command we can repeat the steps above by executing the
following commands in any directory. (You will need to change the `--bin`
argument to cargo-verify to match the directory name.)

```
cat >> Cargo.toml << "EOF"

[target.'cfg(not(verify))'.dependencies]
proptest = { version = "0.10" }

[target.'cfg(verify)'.dependencies]
propverify = { path="/home/rust-verification-tools/propverify" }

[features]
verifier-klee = ["propverify/verifier-klee"]
verifier-crux = ["propverify/verifier-crux"]
verifier-seahorn = ["propverify/verifier-seahorn"]
EOF

cargo verify -v --clean --bin base64 -o app.bc

klee --disable-verify --simplify-sym-indices \
  --output-module --max-memory=1000 --disable-inlining \
  --use-forked-solver --libc=uclibc --posix-runtime \
  --external-calls=all --only-output-states-covering-new \
  --max-sym-array-size=4096 --max-solver-time=30s --max-time=60min --watchdog \
  --max-memory-inhibit=false --max-static-fork-pct=1 --max-static-solve-pct=1 \
  --max-static-cpfork-pct=1 --use-batching-search --batch-instructions=10000 \
  ./app.bc \
  --sym-args 0 1 10 --sym-args 0 2 2 --sym-files 1 8 --sym-stdin 8 --sym-stdout
```

When run in `coreutils/src/uu/base64`, this produces a lot of error messages
such as the following (each report was repeated many times).

```
base64: : No such file or directory
base64: extra operand ''
�: No such file or directory
Input with broken encoding occurred! (s = '�')
```

When run in the `coreutils/src/uu/ls` directory, this produces a lot of error messages
such as the following (each report was repeated many times).

```
Input with broken encoding occurred! (s = '�')
ls: cannot access '': No such file or directory
error: The argument '--ignore <PATTERN>...' requires a value but none was supplied
error: The argument '--wrap <wrap>' requires a value but none was supplied
error: The argument '--width <COLS>' requires a value but none was supplied
```

An obvious question to ask about this output is

> We are seeing a lot of error messages – does this mean that we are finding bugs?

KLEE is exploring lots of different paths through the code and
checking if any of those paths can fail.
In the process, it is finding and checking lots of error handling code and so we
are seeing the programs *correctly* detect incorrect input, report errors in
that input and quit.
We would expect to see something similar if we were fuzz-testing the same
programs with random input data.


## Wrapping up

At this point, we can now compile the [Rust CoreUtils] for
use with KLEE and we can run KLEE on these binaries.
We also found some minor issues with UTF-8 encoding that it might be worth
fixing.

Some possible next steps are:

- Setup a sandbox and repeat the experience on all of the coreutils programs.
  See [CoreUtils
  experiments](https://klee.github.io/docs/coreutils-experiments/)

- Find a way to filter out all the benign error reports generated when you run KLEE.
  A simple starting point might just be to pipe the output through `sort -u`
  or to pipe it through `grep -v 'Input with broken encoding'` or similar.

- Explore the coverage information in `klee-last/run.istats` files to understand
  what is being covered and what is not being tested.
  See [this blog
  post](http://ccadar.blogspot.com/2020/07/measuring-coverage-achieved-by-symbolic.html)
  and [this post]({{site.baseurl}}{% post_url 2021-03-12-profiling-rust %}).

- Use the ktest files in `klee-last/test*.ktest` to generate testcases that
  rerun any errors that you find or that rerun all of the inputs that KLEE
  generated to help you detect any regressions.
  See
  [this post](https://verificationglasses.wordpress.com/2020/10/02/symbolic-execution-klee/)

- Explore KLEE's search heuristics (i.e., `--search=...`) to get KLEE to explore
  different parts of the behaviour of the programs it checks.
  See [this page](https://klee.github.io/docs/options/#search-heuristics)

- At the moment, we are relying on KLEE's built in ability to generate random
  command line arguments, input files, etc. and to test entire applications.
  If we are willing to write a bit of code, we could write test harnesses
  that probe individual parts of the applications.
  I believe that this would be much more effective and it is what our tools
  and libraries are primarily intended to do – but I will leave that to a future
  post.

Enjoy!

--------

[Crux-MIR]:                       https://github.com/GaloisInc/mir-verifier/
[KLEE]:                           https://klee.github.io/
[KLEE status]:                    {{site.baseurl}}{% post_url 2021-03-29-klee-status %}
[PropTest]:                       https://github.com/AltSysrq/proptest/
[Rust CoreUtils]:                 https://github.com/uutils/coreutils
[RVT git repo]:                   {{site.gitrepo}}/
[SeaHorn]:                        https://seahorn.github.io/
[rust2calltree]:                  {{site.gitrepo}}tree/main/rust2calltree
