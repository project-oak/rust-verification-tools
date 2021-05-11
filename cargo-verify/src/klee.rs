// Copyright 2020-2021 The Propverify authors
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::{collections::HashMap, ffi::OsString, fs, path::Path, process::Command};

use lazy_static::lazy_static;
use log::{info, warn};
use regex::Regex;

use crate::{utils::Append, *};

/// Check if Klee is avilable.
pub fn check_install() -> bool {
    let output = Command::new("which").arg("klee").output().ok();

    match output {
        Some(output) => output.status.success(),
        None => false,
    }
}

/// Run Klee and replay
pub fn verify(opt: &Opt, name: &str, entry: &str, bcfile: &Path) -> CVResult<Status> {
    // KLEE output files are put in kleeout directory with filename `name`
    let klee_dir = opt.cargo_toml.with_file_name("kleeout");
    fs::create_dir_all(&klee_dir)?;

    let out_dir = klee_dir.append(name);
    // Ignoring result. We don't care if it fails because the path doesn't
    // exist.
    fs::remove_dir_all(&out_dir).unwrap_or_default();
    if out_dir.exists() {
        Err(format!(
            "Directory or file '{}' already exists, and can't be removed",
            out_dir.to_string_lossy()
        ))?
    }

    info!("     Running KLEE to verify {}", name);
    info!("      file: {}", bcfile.to_string_lossy());
    info!("      entry: {}", entry);
    info!("      results: {}", out_dir.to_string_lossy());

    let (status, stats) = run(&opt, &name, &entry, &bcfile, &out_dir)?;
    if !stats.is_empty() {
        match stats.get("completed paths") {
            Some(n) => info!("     {}: {} paths", name, n),
            None => (),
        }
        info!("     {}: {:?}", name, stats);
    }

    // {out_dir}/test*.err
    let mut failures =
        glob(&glob::Pattern::escape(out_dir.to_str().ok_or("not UTF-8")?).append("/test*.err"))?
            .filter_map(Result::ok)
            .collect::<Vec<_>>();
    failures.sort_unstable();
    info!("      Failing test: {:?}", failures);

    if opt.replay > 0 {
        // use -r -r to see all tests, not just failing tests
        let mut ktests = if opt.replay > 1 {
            // {out_dir}/test*.ktest
            glob(
                &glob::Pattern::escape(out_dir.to_str().ok_or("not UTF-8")?).append("/test*.ktest"),
            )?
            .filter_map(Result::ok)
            .collect::<Vec<_>>()
        } else {
            // Remove the '.err' extension and replace the '.*' ('.abort' or
            // '.ptr') with '.ktest'.
            failures
                .iter()
                .map(|p| {
                    p.with_extension("") // Remove '.err'
                        .with_extension("ktest") // Replace '.*' with '.ktest'
                })
                .collect::<Vec<_>>()
        };
        ktests.sort_unstable();

        for ktest in ktests {
            println!("    Test input {}", ktest.to_str().unwrap_or("???"));
            match replay_klee(&opt, &name, &ktest) {
                Ok(()) => (),
                Err(err) => warn!("Failed to replay: {}", err),
            }
        }
    }

    Ok(status)
}

/// Return an int indicating importance of a line from KLEE's output
/// Low numbers are most important, high numbers least important
///
/// -1: script error (always shown)
/// 1: brief description of error
/// 2: long details about an error
/// 3: warnings
/// 4: non-KLEE output
/// 5: any other KLEE output
fn importance(line: &str, expect: &Option<&str>, name: &str) -> i8 {
    if line.starts_with("VERIFIER_EXPECT:") {
        4
    } else if backends_common::is_expected_panic(&line, &expect, &name) {
        // low priority because we report it directly
        5
    } else if line.contains("assertion failed") {
        1
    } else if line.contains("verification failed") {
        1
    } else if line.contains("with overflow") {
        1
    } else if line.starts_with("KLEE: ERROR: Could not link") {
        -1
    } else if line.starts_with("KLEE: ERROR: Unable to load symbol") {
        -1
    } else if line.starts_with("KLEE: ERROR:") {
        2
    } else if line.starts_with("warning: Linking two modules of different data layouts") {
        4
    } else if line.contains("KLEE: WARNING:") {
        4
    } else if line.contains("KLEE: WARNING ONCE:") {
        4
    } else if line.starts_with("KLEE: output directory") {
        5
    } else if line.starts_with("KLEE: Using") {
        5
    } else if line.starts_with("KLEE: NOTE: Using POSIX model") {
        5
    } else if line.starts_with("KLEE: done:") {
        5
    } else if line.starts_with("KLEE: HaltTimer invoked") {
        5
    } else if line.starts_with("KLEE: halting execution, dumping remaining states") {
        5
    } else if line.starts_with("KLEE: NOTE: now ignoring this error at this location") {
        5
    } else if line.starts_with("KLEE:") {
        // Really high priority to force me to categorize it
        0
    } else {
        // Remaining output is probably output from the application, stack dumps, etc.
        3
    }
}

/// Run Klee and analyse its output.
fn run(
    opt: &Opt,
    name: &str,
    entry: &str,
    bcfile: &Path,
    out_dir: &Path,
) -> CVResult<(Status, HashMap<String, isize>)> {
    let mut cmd = Command::new("klee");

    let user_flags: Vec<_> = opt
        .backend_flags
        .iter()
        .map(|flag| backends_common::format_flag(&flag, &entry, &bcfile, &out_dir))
        .collect::<Result<_, _>>()?;

    if !opt.replace_backend_flags {
        cmd.args(&[
            "--exit-on-error",
            "--entry-point",
            entry,
            // "--posix-runtime",
            // "--libcxx",
            "--libc=klee",
            "--silent-klee-assume",
            "--disable-verify", // workaround https://github.com/klee/klee/issues/937
        ])
        .arg("--output-dir")
        .arg(out_dir)
        .args(user_flags)
        .arg(bcfile)
        .args(&opt.args);
    } else {
        cmd.args(user_flags);
    }

    let (_, stderr, _) = cmd.latin1_output_info_ignore_exit(&opt, Verbosity::Major)?;

    // We scan stderr for:
    // 1. Indications of the expected output (eg from #[should_panic])
    // 2. Indications of success/failure
    // 3. Information relevant at the current level of verbosity
    // 4. Statistics

    // Scan for expectation message
    let mut expect = None;
    for l in stderr.lines() {
        if l == "VERIFIER_EXPECT: should_panic" {
            expect = Some("");
        } else if let Some(e) = l
            .strip_prefix("VERIFIER_EXPECT: should_panic(expected = \"")
            .and_then(|l| l.strip_suffix("\")"))
        {
            info!("Expecting '{}'", e);
            expect = Some(e);
        }
    }

    // Scan for first message that indicates result
    let status = stderr
        .lines()
        .find_map(|l| {
            if l.starts_with("KLEE: HaltTimer invoked") {
                Some(Status::Timeout)
            } else if l.starts_with("KLEE: halting execution, dumping remaining states") {
                Some(Status::Timeout)
            } else if l.starts_with("KLEE: ERROR: Could not link") {
                Some(Status::Unknown)
            } else if l.starts_with("KLEE: ERROR: Unable to load symbol") {
                Some(Status::Unknown)
            } else if l.starts_with("KLEE: ERROR:") && l.contains("unreachable") {
                Some(Status::Reachable)
            } else if l.starts_with("KLEE: ERROR:") && l.contains("overflow") {
                Some(Status::Overflow)
            } else if l.starts_with("KLEE: ERROR:") {
                Some(Status::Error)
            } else if l.starts_with("VERIFIER_EXPECT:") {
                // don't confuse this line with an error!
                None
            } else if backends_common::is_expected_panic(&l, &expect, &name) {
                Some(Status::Verified)
            } else if l.contains("assertion failed") {
                Some(Status::AssertFailed)
            } else if l.contains("verification failed") {
                Some(Status::Error)
            } else if l.contains("index out of bounds") {
                Some(Status::OutOfBounds)
            } else if l.contains("with overflow") {
                Some(Status::Overflow)
            } else if l.contains("panicked at") {
                Some(Status::Panic)
            } else if l.contains("note: run with `RUST_BACKTRACE=1`") {
                Some(Status::Error)
            } else if l.contains("KLEE: done:") {
                match expect {
                    None => Some(Status::Verified),
                    _ => Some(Status::Error),
                }
            } else {
                None
            }
        })
        .unwrap_or_else(|| {
            warn!("Unable to determine status of {}", name);
            Status::Unknown
        });

    info!("Status: '{}' expected: '{:?}'", status, expect);

    // Scan for statistics
    lazy_static! {
        static ref KLEE_DONE: Regex = Regex::new(r"^KLEE: done:\s+(.*)= (\d+)").unwrap();
    }

    let stats: HashMap<String, isize> = stderr
        .lines()
        // .filter(|l| l.starts_with("KLEE: done:"))
        .filter_map(|l| {
            KLEE_DONE.captures(l).and_then(|caps| {
                // If the value doesn't parse we throw the line.
                caps.get(2)
                    .unwrap()
                    .as_str()
                    .parse::<isize>()
                    .ok()
                    .map(|v| (caps.get(1).unwrap().as_str().trim().to_string(), v))
            })
        })
        .collect();

    for l in stderr.lines() {
        if importance(&l, &expect, &name) < opt.verbose as i8 {
            println!("{}", l);
        }
    }

    Ok((status, stats))
}

/// Replay a KLEE "ktest" file
fn replay_klee(opt: &Opt, name: &str, ktest: &Path) -> CVResult<()> {
    let mut cmd = Command::new("cargo");

    if opt.tests || !opt.test.is_empty() {
        cmd.arg("test").arg("--manifest-path").arg(&opt.cargo_toml);

        if !opt.features.is_empty() {
            cmd.arg("--features").arg(opt.features.join(","));
        }

        cmd.arg(&name).args(&["--", "--nocapture"]);
    } else {
        cmd.arg("run").arg("--manifest-path").arg(&opt.cargo_toml);

        if !opt.features.is_empty() {
            cmd.arg("--features").arg(opt.features.join(","));
        }

        if !opt.args.is_empty() {
            cmd.arg("--").args(opt.args.iter());
        }
    }

    let rustflags = match std::env::var_os("RUSTFLAGS") {
        Some(env_rustflags) => env_rustflags.append(" --cfg=verify"),
        None => OsString::from("--cfg=verify"),
    };
    cmd.env("RUSTFLAGS", rustflags).env("KTEST_FILE", ktest);

    // Note that we do not treat this as an error, because
    // the interesting case for replay is when KLEE had found an error.
    let (stdout, stderr, _success) = cmd.output_info_ignore_exit(&opt, Verbosity::Major)?;

    for line in stdout.lines().chain(stderr.lines()) {
        println!("{}", line);
    }

    Ok(())
}
