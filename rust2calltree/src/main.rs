// Copyright 2020-2021 The Propverify authors
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use anyhow::{Context, Result};
use log::info;
use rustc_demangle::demangle;
use std::fs::File;
use std::io::{prelude::*, BufReader, BufWriter};
use std::path::PathBuf;
use structopt::StructOpt;

// Command line argument parsing
#[derive(StructOpt)]
#[structopt(
    name = "rust2calltree",
    about = "Convert Rust-derived profiles to kcachegrind's calltree input format by demangling function names"
)]
struct Opt {
    /// Input file.
    #[structopt(name = "INPUT", parse(from_os_str))]
    input: PathBuf,

    /// Output file
    #[structopt(
        short,
        long,
        name = "OUTPUT",
        parse(from_os_str),
        default_value = "callgrind.out"
    )]
    output: PathBuf,

    /// Increase message verbosity
    #[structopt(short, long, parse(from_occurrences))]
    verbosity: usize,
}

fn main() -> Result<()> {
    let opt = Opt::from_args();

    #[rustfmt::skip]
    stderrlog::new()
        .verbosity(opt.verbosity)
        .init()
        .unwrap();

    // Open the input file
    let input = opt.input.to_str().unwrap();
    info!("Reading input from {}", input);
    let input = File::open(input).with_context(|| format!("can't open input file '{}'", input))?;
    let input = BufReader::new(input);

    // Open the output file
    let output = opt.output.to_str().unwrap();
    info!("Writing demangled output to {}", output);
    let output =
        File::create(&output).with_context(|| format!("can't open output file '{}'", output))?;
    let mut output = BufWriter::new(&output);

    for line in input.lines() {
        let mut line = line?;
        if let Some(s) = line.strip_prefix("fn=") {
            line = format!("fn={:#}", demangle(s));
        } else if let Some(s) = line.strip_prefix("cfn=") {
            line = format!("cfn={:#}", demangle(s));
        }
        writeln!(output, "{}", line)?;
    }

    Ok(())
}
