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

/// Check if SMACK is available.
pub fn check_install() -> bool {
    let output = Command::new("which").arg("smack").output().ok();

    match output {
        Some(output) => output.status.success(),
        None => false,
    }
}

/// Run SMACK
pub fn verify(opt: &Opt, name: &str, entry: &str, bcfile: &Path) -> CVResult<Status> {
    let out_dir = opt.cargo_toml.with_file_name("smackout").append(name);

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

    info!("     Running SMACK to verify {}", name);
    info!("      file: {}", bcfile.to_string_lossy());
    info!("      entry: {}", entry);
    info!("      results: {}", out_dir.to_string_lossy());

    run(&opt, &name, &entry, &bcfile, &out_dir)
}

/// Run Smack and analyse its output.
fn run(opt: &Opt, name: &str, entry: &str, bcfile: &Path, out_dir: &Path) -> CVResult<Status> {
    let mut cmd = Command::new("smack");

    let user_flags: Vec<_> = opt
        .backend_flags
        .iter()
        .map(|flag| backends_common::format_flag(&flag, &entry, &bcfile, &out_dir))
        .collect::<Result<_, _>>()?;

    cmd.arg("--verifier=boogie")
        .args(user_flags)
        .arg(String::from("--entry-points=") + entry)
        .arg(bcfile);
    let (stdout, stderr, _) = cmd.output_info_ignore_exit(&opt, Verbosity::Major)?;

    // Scan for result mesage
    let status = stderr
        .lines()
        .chain(stdout.lines())
        .find_map(|l| {
            if l.starts_with("SMACK found no errors") {
                Some(Status::Verified)
            } else if l.starts_with("SMACK found an error") {
                Some(Status::Error)
            } else {
                None
            }
        })
        .unwrap_or_else(|| {
            warn!("Unable to determine status of {}", name);
            Status::Unknown
        });

    Ok(status)
}
