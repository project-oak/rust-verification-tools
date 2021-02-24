// Copyright 2020-2021 The Propverify authors
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

#![feature(command_access)]

use std::{
    collections::HashSet,
    error, fmt,
    path::{Path, PathBuf},
    process::{exit, Command},
    str::from_utf8,
};

use cargo_metadata::{CargoOpt, MetadataCommand};
use glob::glob;
use lazy_static::lazy_static;
use log::error;
use rayon::prelude::*;
use regex::Regex;
use rustc_demangle::demangle;
use structopt::{clap::arg_enum, StructOpt};
use utils::{add_pre_ext, Append};

// utils must come before the other modules as it defines macros that they might
// use.
#[macro_use]
mod utils;

mod backends_common;
mod klee;
mod proptest;
mod run_tools;
mod seahorn;

use run_tools::*;

// Command line arguments
#[derive(StructOpt)]
#[structopt(
    name = "cargo-verify",
    about = "Execute verification tools",
    // version number is taken automatically from Cargo.toml
)]
pub struct Opt {
    // TODO: make this more like 'cargo test --manifest-path <PATH>'
    // (i.e., path to Cargo.toml)
    /// Path to Cargo.toml
    #[structopt(
        long = "manifest-path",
        name = "PATH",
        parse(from_os_str),
        default_value = "Cargo.toml"
    )]
    cargo_toml: PathBuf,

    /// Arguments to pass to program under test
    #[structopt(name = "ARG", last = true)]
    args: Vec<String>,

    // backend_arg is used for holding the CL option. After parsing, if the user
    // specified a backend it will be copied to the `backend` field below, if
    // the user didn't specify a backend, we will auto-detect one, and put it
    // in the `backend` field.
    /// Select verification backend
    #[structopt(
        short = "b",
        long = "backend",
        name = "BACKEND",
        possible_values = &Backend::variants(),
        case_insensitive = true,
    )]
    backend_arg: Option<Backend>,

    // See the comment of `backend_arg` above.
    #[structopt(skip = Backend::Klee)] // the initial value has no meaning, it will be overwritten
    backend: Backend,

    /// Extra verification flags
    #[structopt(long, number_of_values = 1, use_delimiter = true)]
    backend_flags: Vec<String>,

    /// Space or comma separated list of features to activate
    #[structopt(long, name = "FEATURES", number_of_values = 1, use_delimiter = true)]
    features: Vec<String>,

    /// Run `cargo clean` first
    #[structopt(short, long)]
    clean: bool,

    /// Verify all tests instead of 'main'
    #[structopt(short, long)]
    tests: bool,

    // TODO: make this more like 'cargo test [TESTNAME]'
    /// Only verify tests containing this string in their names
    #[structopt(long, number_of_values = 1, name = "TESTNAME")]
    test: Vec<String>,

    // jobs_arg is used for holding the CL option. After parsing, if the user
    // specified a value it will be copied to the `jobs` field below, if the
    // user didn't specify a value, we will use num_cpus, and put it in the
    // `jobs` field.
    /// Number of parallel jobs, defaults to # of CPUs
    #[structopt(short = "j", long = "jobs", name = "N")]
    jobs_arg: Option<usize>,

    // See the comment of `jobs_arg` above.
    #[structopt(skip)]
    jobs: usize,

    /// Replay to display concrete input values
    #[structopt(short, long, parse(from_occurrences))]
    replay: usize,

    /// Use verbose output (-vvvvvv very verbose output)
    #[structopt(short, long, parse(from_occurrences))]
    verbose: usize,
}

arg_enum! {
    #[derive(Debug, PartialEq, Copy, Clone)]
    enum Backend {
        Proptest,
        Klee,
        Seahorn,
    }
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Status {
    Unknown, // E.g. the verifier failed to execute.
    Verified,
    Error, // E.g. the verifier found a violation.
    Timeout,
    Overflow,
    Reachable,
}

impl fmt::Display for Status {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Status::Unknown => write!(f, "UNKNOWN"),
            Status::Verified => {
                if f.alternate() {
                    // "{:#}"
                    write!(f, "OK")
                } else {
                    // "{}"
                    write!(f, "VERIFIED")
                }
            }
            Status::Error => write!(f, "ERROR"),
            Status::Timeout => write!(f, "TIMEOUT"),
            Status::Overflow => write!(f, "OVERFLOW"),
            Status::Reachable => write!(f, "REACHABLE"),
        }
    }
}

type CVResult<T> = Result<T, Box<dyn error::Error>>;

/// Parse the command line and make sure it makes sense.
fn process_command_line() -> CVResult<Opt> {
    // cargo-verify can be called directly, or by placing it on the `PATH` and
    // calling it through `cargo` (i.e. `cargo verify ...`.
    let mut args: Vec<_> = std::env::args().collect();
    if args.get(1).map(AsRef::as_ref) == Some("verify") {
        // Looks like the script was invoked by `cargo verify` - we have to
        // remove the second argument.
        args.remove(1);
    }
    let mut opt = Opt::from_iter(args.into_iter());
    // let mut opt = Opt::from_args();

    opt.backend = match opt.backend_arg {
        // Check if the backend that was specified on the CL is installed.
        Some(Backend::Proptest) => {
            assert!(proptest::check_install());
            Backend::Proptest
        }
        Some(Backend::Klee) => {
            if !klee::check_install() {
                Err("Klee is not installed")?;
            }
            Backend::Klee
        }
        Some(Backend::Seahorn) => {
            if !seahorn::check_install() {
                Err("Seahorn is not installed")?;
            }
            Backend::Seahorn
        }
        None => {
            // If the user did not specify a backend, use the first one that we find.
            let backend = if klee::check_install() {
                Backend::Klee
            } else if seahorn::check_install() {
                Backend::Seahorn
            } else {
                assert!(proptest::check_install());
                Backend::Proptest
            };
            println!("Using {} as backend", backend);
            backend
        }
    };

    // To be compatible with `cargo test`, features might be space separated.
    opt.features = opt
        .features
        .iter()
        .flat_map(|s| s.split(' '))
        .map(String::from)
        .collect::<Vec<_>>();

    // Backend specific options.
    match opt.backend {
        Backend::Proptest => {
            if opt.replay > 0 && !opt.args.is_empty() {
                Err("The Proptest backend does not support '--replay' and passing arguments together.")?;
            }
        }
        Backend::Seahorn => {
            if !opt.args.is_empty() {
                Err("The Seahorn backend does not support passing arguments yet.")?;
            }
            if opt.replay != 0 {
                Err("The Seahorn backend does not support '--replay' yet.")?;
            }

            opt.features.push(String::from("verifier-seahorn"));
        }
        Backend::Klee => {
            opt.features.push(String::from("verifier-klee"));
        }
    }

    // Use the user specified number of jobs, or the number of CPUs.
    opt.jobs = opt.jobs_arg.unwrap_or(num_cpus::get());

    Ok(opt)
}

/// Invoke a checker (verifier or fuzzer) on a crate.
fn main() -> CVResult<()> {
    let opt = process_command_line()?;
    stderrlog::new().verbosity(opt.verbose).init()?;

    if opt.clean {
        clean(&opt);
    }

    let package = get_meta_package_name(&opt)?;
    info_at!(&opt, 1, "Checking {}", &package);

    let status = match opt.backend {
        Backend::Proptest => {
            info_at!(&opt, 1, "  Invoking cargo run with proptest backend");
            proptest::run(&opt)
        }
        _ => {
            let target = get_default_host(&opt.cargo_toml.parent().unwrap_or(&PathBuf::from(".")))?;
            info_at!(&opt, 4, "target: {}", target);
            verify(&opt, &package, &target)
        }
    }
    .unwrap_or_else(|err| {
        error!("{}", err);
        exit(1)
    });

    println!("VERIFICATION_RESULT: {}", status);
    if status != Status::Verified {
        exit(1);
    }
    Ok(())
}

/// Compile a Rust crate to generate bitcode and run one of the LLVM verifier
/// backends on the result.
fn verify(opt: &Opt, package: &str, target: &str) -> CVResult<Status> {
    // Compile and link the patched file using LTO to generate the entire
    // application in a single LLVM file
    info_at!(&opt, 1, "  Building {} for verification", package);
    let bcfile = build(&opt, &package, &target)?;

    // Get the functions we need to verify, and their mangled names.
    let tests = if opt.tests || !opt.test.is_empty() {
        // If using the --tests or --test flags, generate a list of tests and
        // their mangled names.
        info_at!(&opt, 3, "  Getting list of tests in {}", &package);
        let mut tests = list_tests(&opt, &target)?;
        if !opt.test.is_empty() {
            tests = tests
                .into_iter()
                .filter(|t| opt.test.iter().any(|f| t.contains(f)))
                .collect();
        }
        if tests.is_empty() {
            Err("  No tests found")?
        }
        let tests: Vec<String> = tests
            .iter()
            .map(|t| format!("{}::{}", package, t))
            .collect();

        // then look up their mangled names in the bcfile
        mangle_functions(&opt, &bcfile, &tests)?
    } else if opt.backend == Backend::Seahorn {
        // Find the entry function (mangled main)
        let mains = mangle_functions(&opt, &bcfile, &[String::from(package) + "::main"])?;
        match mains.as_slice() {
            [(_, _)] => mains,
            [] => Err("  FAILED: can't find the 'main' function")?,
            _ => Err("  FAILED: found more than one 'main' function")?,
        }
    } else {
        vec![("main".to_string(), "main".to_string())]
    };

    // Remove the package name from the function names (important for Klee?) in tests.
    let tests: Vec<_> = tests
        .into_iter()
        .map(|(name, mangled)| {
            if let Some(name) = name.strip_prefix(&format!("{}::", package)) {
                (name.to_string(), mangled)
            } else {
                (name, mangled)
            }
        })
        .collect();

    #[rustfmt::skip]
    info_at!(&opt, 1, "  Checking {}",
             tests.iter().cloned().unzip::<_, _, Vec<_>, Vec<_>>().0.join(", ")
    );
    info_at!(opt, 4, "Mangled: {:?}", tests);

    // For each test function, we run the backend and sift through its
    // output to generate an appropriate status string.
    println!("Running {} test(s)", tests.len());

    let results: Vec<Status> = if opt.jobs > 1 {
        // Run the verification in parallel.

        // `build_global` must not be called more than once!
        // This call configures the thread-pool for `par_iter` below.
        rayon::ThreadPoolBuilder::new()
            .num_threads(opt.jobs)
            .build_global()?;

        tests
            .par_iter() // <- parallelised iterator
            .map(|(name, entry)| verifier_run(&opt, &bcfile, &name, &entry))
            .collect()
    } else {
        // Same as above but without the overhead of rayon
        tests
            .iter() // <- this is the only difference
            .map(|(name, entry)| verifier_run(&opt, &bcfile, &name, &entry))
            .collect()
    };

    // Count pass/fail
    let passes = results.iter().filter(|r| **r == Status::Verified).count();
    let fails = results.len() - passes;
    // randomly pick one failing status (if any)
    let status = results
        .into_iter()
        .find(|r| *r != Status::Verified)
        .unwrap_or(Status::Verified);

    println!(
        "test result: {:#}. {} passed; {} failed",
        status, passes, fails
    );
    Ok(status)
}

/// Invoke one of the supported verification backends on entry point 'entry'
/// (with pretty name 'name') in bitcodefile 'bcfile'.
fn verifier_run(opt: &Opt, bcfile: &Path, name: &str, entry: &str) -> Status {
    let status = match opt.backend {
        Backend::Klee => klee::verify(&opt, &name, &entry, &bcfile),
        Backend::Seahorn => seahorn::verify(&opt, &name, &entry, &bcfile),
        Backend::Proptest => unreachable!(),
    }
    .unwrap_or_else(|err| {
        error!("{}", err);
        error!("Failed to run test '{}'.", name);
        Status::Unknown
    });

    println!("test {} ... {:#}", name, status);
    status
}

/// Compile, link and do transformations on LLVM bitcode.
fn build(opt: &Opt, package: &str, target: &str) -> CVResult<PathBuf> {
    let (mut bc_file, c_files) = compile(&opt, &package, target)?;

    // Link bc file (from all the Rust code) against the (backend-specific)
    // runtime library and any c_files generated by build scripts from any C/C++
    // build scripts
    let new_bc_file = add_pre_ext(&bc_file, "link");
    //let rvt_dir = std::env::var("RVT_DIR")?;
    //let backend_lc = format!("{}", opt.backend).to_lowercase();
    //let runtime = format!("{}/runtime/rvt-{}.bc", rvt_dir, backend_lc);
    //let runtime = PathBuf::from(&runtime);
    info_at!(
        &opt,
        3,
        "  Linking {} and [{}] to produce {}",
        bc_file.to_string_lossy(),
        //runtime.to_string_lossy(),
        c_files
            .iter()
            .map(|p| p.to_string_lossy())
            .collect::<Vec<_>>()
            .join(", "),
        new_bc_file.to_string_lossy()
    );
    // Link multiple bitcode files together.
    Command::new("llvm-link")
        .arg("-o")
        .arg(&new_bc_file)
        //.arg(runtime)
        .arg(&bc_file)
        .args(&c_files)
        .output_info()?;
    bc_file = new_bc_file;

    if opt.backend == Backend::Seahorn {
        info_at!(&opt, 1, "  Patching LLVM file for Seahorn");
        let new_bc_file = add_pre_ext(&bc_file, "patch-sea");
        patch_llvm(&opt, &["--seahorn"], &bc_file, &new_bc_file)?;
        bc_file = new_bc_file;
    }

    // todo: This is probably useful with all verifiers - but
    // making it KLEE-only until we have a chance to test it.
    if opt.backend == Backend::Klee {
        info_at!(&opt, 1, "  Patching LLVM file for initializers and feature tests");
        let new_bc_file = add_pre_ext(&bc_file, "patch-init-feat");
        patch_llvm(
            &opt,
            &["--initializers", "--features"],
            &bc_file,
            &new_bc_file,
        )?;
        bc_file = new_bc_file;
    }

    Ok(bc_file)
}

/// Return the environment variables needed for building.  Each item in the
/// vector is a pair `(a, b)` where `a` is the variable name and `b` is its
/// value.
fn get_build_envs(opt: &Opt) -> CVResult<Vec<(String, String)>> {
    let mut rustflags = vec![
        "-Clto", // Generate linked bitcode for entire crate
        "-Cembed-bitcode=yes",
        "--emit=llvm-bc",
        // Any value except 0 seems to work
        "--cfg=verify", // Select verification versions of libraries
        // "-Ccodegen-units=1",     // Optimize a bit more?
        "-Zpanic_abort_tests", // Panic abort is simpler
        "-Cpanic=abort",
        "-Warithmetic-overflow", // Detecting errors is good!
        "-Coverflow-checks=yes",
        "-Cno-vectorize-loops", // KLEE does not support vector intrinisics
        "-Cno-vectorize-slp",
        "-Ctarget-feature=-mmx,-sse,-sse2,-sse3,-ssse3,-sse4.1,-sse4.2,-3dnow,-3dnowa,-avx,-avx2",
        // use clang to link with LTO - to handle calls to C libraries
        "-Clinker-plugin-lto",
        "-Clinker=clang-10",
        "-Clink-arg=-fuse-ld=lld",
    ]
    .join(" ");

    if opt.backend != Backend::Seahorn {
        // Avoid generating SSE instructions
        rustflags.push_str(" -Copt-level=1");
    }

    match std::env::var_os("RUSTFLAGS") {
        Some(env) => {
            rustflags.push_str(" ");
            rustflags.push_str(env.to_str().ok_or("not UTF-8")?);
        }
        None => (),
    };

    Ok(vec![
        (String::from("RUSTFLAGS"), rustflags),
        (String::from("CRATE_CC_NO_DEFAULTS"), String::from("true")),
        (String::from("CFLAGS"), String::from("-flto=thin")),
        (String::from("CC"), String::from("clang-10")),
    ])
}

/// Compile a crate for verification.
/// Return a bcfile for the entire (linked) crate, and c object files that need
/// to be linked with the bcfile.
fn compile(opt: &Opt, package: &str, target: &str) -> CVResult<(PathBuf, Vec<PathBuf>)> {
    let mut cmd = Command::new("cargo");
    cmd.arg("build").arg("--manifest-path").arg(&opt.cargo_toml);

    if !opt.features.is_empty() {
        cmd.arg("--features").arg(opt.features.join(","));
    }

    if opt.tests || !opt.test.is_empty() {
        cmd.arg("--tests");
    }

    // The following line is not present because we care about the target It is
    // there to allow us to use -Clto to build crates whose dependencies invoke
    // proc_macros.
    // FIXME: "=="?
    cmd.arg(format!("--target={}", target))
        .args(vec!["-v"; opt.verbose])
        .envs(get_build_envs(&opt)?)
        .output_info()?;
    // .env("PATH", ...)

    // Find the target directory
    // (This may not be inside the crate if using workspaces)
    info_at!(&opt, 4, "  Getting target directory");
    let target_dir = get_meta_target_directory(&opt)?;

    // {target_dir}/{target}/debug/deps/{package}*.bc
    // where the file name has exactly one 1 '.' (because later we add similar
    // files, with multiple dots, and we don't want them here)
    // and the file has a main function.
    let bc_files = glob(
        &glob::Pattern::escape(
            target_dir
                .clone()
                .append(target)
                .append("debug")
                .append("deps")
                .append(package)
                .to_str()
                .ok_or("not UTF-8")?,
        )
        .append("*.bc"),
    )?
    .filter_map(Result::ok)
    // Filter only files that have exactly one '.'
    .filter(|p| p.file_name().map(|f| f.to_string_lossy().matches('.').count() == 1).unwrap_or(false))
    .filter(|p| count_symbols(&opt, p, &["main", "_main"]).map_or(false, |c| c > 0))
    .collect::<Vec<_>>();

    // Make sure there is only one such file.
    let bc_file: PathBuf = match bc_files.as_slice() {
        [_] => {
            // Move element 0 out of the Vec (and into `bcfile`).
            (bc_files as Vec<_>).remove(0)
        }
        [] => {
            if opt.tests || !opt.test.is_empty() {
                Err("  FAILED: Use --tests with library crates")?
            } else {
                Err(format!("  FAILED: Test {} compilation error", &package))?
            }
        }
        _ => {
            error!(
                "    Ambiguous bitcode files {}",
                bc_files
                    .iter()
                    .map(|p| p.to_string_lossy())
                    .collect::<Vec<_>>()
                    .join(", ")
            );
            Err(format!("  FAILED: Test {} compilation error", &package))?
        }
    };

    // {targetdir}/{target}/debug/build/ * /out/ *.o"
    let c_files = glob(
        &glob::Pattern::escape(
            target_dir
                .clone()
                .append(target)
                .append("debug")
                .append("build")
                .to_str()
                .ok_or("not UTF-8")?,
        )
        .append("/*/out/*.o"),
    )?
    .filter_map(Result::ok)
    .collect::<Vec<_>>();

    // build_plan = read_build_plan(crate, flags)
    // print(json.dumps(build_plan, indent=4, sort_keys=True))
    Ok((bc_file, c_files))
}

/// Patch LLVM file to enable verification
///
/// Some of the patching performed includes:
/// - arranging for initializers to be executed (this makes std::env::args()
///   work)
/// - redirecting panic! to invoke backend-specific intrinsic functions for
///   reporting errors
fn patch_llvm(opt: &Opt, options: &[&str], bcfile: &Path, new_bcfile: &Path) -> CVResult<()> {
    Command::new("rvt-patch-llvm")
        .arg(bcfile)
        .arg("-o")
        .arg(new_bcfile)
        .args(options)
        .args(vec!["-v"; opt.verbose])
        .output_info()?;
    Ok(())
}

/// Find a function defined in LLVM bitcode file.
/// Demangle all the function names, and compare tham to `names`.
fn mangle_functions(
    opt: &Opt,
    bcfile: &Path,
    names: &[impl AsRef<str>],
) -> CVResult<Vec<(String, String)>> {
    let names: HashSet<&str> = names.iter().map(AsRef::as_ref).collect();

    info_at!(
        &opt,
        4,
        "    Looking up {:?} in {}",
        names,
        bcfile.to_string_lossy()
    );

    let (stdout, _) = Command::new("llvm-nm")
        .arg("--defined-only")
        .arg(bcfile)
        .output_info()?;

    let rs: Vec<(String, String)> = stdout
        .lines()
        .map(|l| l.split(" ").collect::<Vec<&str>>())
        .filter_map(|l| {
            if l.len() == 3
                && l[1].to_lowercase() == "t"
                && (l[2].starts_with("__ZN") || l[2].starts_with("_ZN"))
            {
                let mangled = if l[2].starts_with("__ZN") {
                    // on OSX, llvm-nm shows a double underscore prefix
                    &l[2][1..]
                } else {
                    &l[2]
                };
                // The alternative format ({:#}) is without the hash at the end.
                let dname = format!("{:#}", demangle(mangled));
                if names.contains(dname.as_str()) {
                    Some((dname, mangled.into()))
                } else {
                    None
                }
            } else {
                None
            }
        })
        .collect();

    info_at!(&opt, 4, "      Found {:?}", rs);

    // TODO: this doesn't look right:
    // missing = set(paths) - paths.keys()
    let missing = names.len() - rs.len();
    if missing > 0 {
        Err(format!("Unable to find {} tests in bytecode file", missing))?
    }
    Ok(rs)
}
