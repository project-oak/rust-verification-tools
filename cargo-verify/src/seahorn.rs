// Copyright 2020-2021 The Propverify authors
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::{ffi::OsString, fs, path::Path, process::Command};

use log::{info, warn};

use crate::{utils::Append, *};

/// Check if Seahorn is avilable.
pub fn check_install() -> bool {
    // TODO: maybe it's better to check `seahorn --version`?
    let output = Command::new("which").arg("sea").output().ok();

    match output {
        Some(output) => output.status.success(),
        None => false,
    }
}

/// Run Seahorn
pub fn verify(opt: &Opt, name: &str, entry: &str, bcfile: &Path) -> CVResult<Status> {
    let out_dir = opt.cargo_toml.with_file_name("seaout").append(name);

    // Ignoring result. We don't care if it fails because the path doesn't
    // exist.
    fs::remove_dir_all(&out_dir).unwrap_or_default();
    if out_dir.exists() {
        Err(format!(
            "Directory or file '{}' already exists, and can't be removed",
            out_dir.to_string_lossy()
        ))?
    }
    fs::create_dir_all(&out_dir)?;

    info!("     Running Seahorn to verify {}", name);
    info!("      file: {}", bcfile.to_string_lossy());
    info!("      entry: {}", entry);
    info!("      results: {}", out_dir.to_string_lossy());

    run(&opt, &name, &entry, &bcfile, &out_dir)
}

/// Return an int indicating importance of a line from KLEE's output
/// Low numbers are most important, high numbers least important
///
/// -1: script error (always shown)
/// 1: brief description of error
/// 2: long details about an error
/// 3: warnings
/// 4: non-Seahorn output
/// 5: any other Seahorn output
fn importance(line: &str, expect: &Option<&str>, name: &str) -> i8 {
    if line.starts_with("VERIFIER_EXPECT:") {
        4
    } else if line == "sat" {
        1
    } else if line.starts_with("Warning: Externalizing function:")
        || line.starts_with("Warning: not lowering an initializer for a global struct:")
    {
        4
    } else if backends_common::is_expected_panic(&line, &expect, &name) || line == "unsat" {
        5
    } else if line.starts_with("Warning:") {
        // Really high priority to force me to categorize it
        0
    } else {
        // Remaining output is probably output from the application, stack dumps, etc.
        3
    }
}

/// Run Seahorn and analyse its output.
fn run(opt: &Opt, name: &str, entry: &str, bcfile: &Path, out_dir: &Path) -> CVResult<Status> {
    let mut cmd = Command::new("sea");
    cmd.args(&["bpf",
               // The following was extracted from `sea yama -y VCC/seahorn/sea_base.yaml`
               "-O3",
               "--inline",
               "--enable-loop-idiom",
               "--enable-indvar",
               "--no-lower-gv-init-struct",
               "--externalize-addr-taken-functions",
               "--no-kill-vaarg",
               "--with-arith-overflow=true",
               "--horn-unify-assumes=true",
               "--horn-gsa",
               "--no-fat-fns=bcmp,memcpy,assert_bytes_match,ensure_linked_list_is_allocated,sea_aws_linked_list_is_valid",
               "--dsa=sea-cs-t",
               "--devirt-functions=types",
               "--bmc=opsem",
               "--horn-vcgen-use-ite",
               "--horn-vcgen-only-dataflow=true",
               "--horn-bmc-coi=true",
               "--sea-opsem-allocator=static",
               "--horn-explicit-sp0=false",
               "--horn-bv2-lambdas",
               "--horn-bv2-simplify=true",
               "--horn-bv2-extra-widemem",
               "--horn-stats=true",
               "--keep-temps",
    ]);

    cmd.arg(OsString::from("--temp-dir=").append(out_dir))
        .arg(String::from("--entry=") + entry)
        .args(&opt.backend_flags)
        .arg(&bcfile);
    // .args(&opt.args)

    let (stdout, stderr, _) = cmd.output_info_ignore_exit(&opt, 3)?;

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
        .chain(stdout.lines())
        .find_map(|l| {
            if l.starts_with("VERIFIER_EXPECT:") {
                // don't confuse this line with an error!
                None
            } else if backends_common::is_expected_panic(&l, &expect, &name) {
                Some(Status::Verified)
            } else if l == "sat" {
                Some(Status::Error)
            } else if l == "unsat" {
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

    info!(
        "Status: '{}' expected: '{}'",
        status,
        expect.unwrap_or("---")
    );

    // TODO: Scan for statistics

    for l in stderr.lines() {
        if importance(&l, &expect, &name) < opt.verbose as i8 {
            println!("{}", l);
        }
    }

    Ok(status)
}
