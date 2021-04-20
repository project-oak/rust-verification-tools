use std::{io::Write, iter, str::Lines};

use log::info;

use crate::*;

/// Trait for wrapping `std::process::Command::output()` with logging.
pub trait OutputInfo {
    fn output_info(&mut self, opt: &Opt, lvl: Verbosity) -> CVResult<(String, String)> {
        self.output_info_helper(
            &opt,
            lvl,
            |v| String::from(from_utf8(v).expect("not UTF-8")),
            false,
        )
        .map(|(stdout, stderr, _)| (stdout, stderr))
    }

    fn latin1_output_info(&mut self, opt: &Opt, lvl: Verbosity) -> CVResult<(String, String)> {
        self.output_info_helper(&opt, lvl, utils::from_latin1, false)
            .map(|(stdout, stderr, _)| (stdout, stderr))
    }

    fn output_info_ignore_exit(
        &mut self,
        opt: &Opt,
        lvl: Verbosity,
    ) -> CVResult<(String, String, bool)> {
        self.output_info_helper(
            &opt,
            lvl,
            |v| String::from(from_utf8(v).expect("not UTF-8")),
            true,
        )
    }

    fn latin1_output_info_ignore_exit(
        &mut self,
        opt: &Opt,
        lvl: Verbosity,
    ) -> CVResult<(String, String, bool)> {
        self.output_info_helper(&opt, lvl, utils::from_latin1, true)
    }

    fn output_info_helper(
        &mut self,
        opt: &Opt,
        lvl: Verbosity,
        trans: impl Fn(&[u8]) -> String,
        ignore_exit: bool,
    ) -> CVResult<(String, String, bool)>;
}

impl OutputInfo for Command {
    fn output_info_helper(
        &mut self,
        opt: &Opt,
        lvl: Verbosity,
        trans: impl Fn(&[u8]) -> String,
        ignore_exit: bool,
    ) -> CVResult<(String, String, bool)> {
        info_cmd(&self);

        if let Err(e) = add_to_script(&self, &opt) {
            eprintln!("Cannot write to script: {:?}", e);
        }

        let output = self.output()?;

        let stdout = trans(&output.stdout);
        info_lines(&opt, lvl, "STDOUT: ", stdout.lines());

        let stderr = trans(&output.stderr);
        info_lines(&opt, lvl, "STDERR: ", stderr.lines());

        if !ignore_exit && !output.status.success() {
            match output.status.code() {
                Some(code) => Err(format!(
                    "FAILED: '{}' terminated with exit code {}.",
                    self.get_program().to_string_lossy(),
                    code
                ))?,
                None => Err(format!(
                    "FAILED: '{}' terminated by a signal.",
                    self.get_program().to_string_lossy()
                ))?,
            }
        }

        Ok((stdout, stderr, output.status.success()))
    }
}

/// Log `cmd` nicely.
fn info_cmd(cmd: &Command) {
    info!(
        "Running '{}' in '{}' with command:\n{}",
        cmd.get_program().to_string_lossy(),
        cmd.get_current_dir()
            .unwrap_or(&PathBuf::from("."))
            .to_string_lossy(),
        iter::once(cmd.get_program())
            .chain(cmd.get_args())
            .map(|s| shell_escape::escape(s.to_string_lossy()))
            .collect::<Vec<_>>()
            .join(" ")
    );

    let envs = cmd.get_envs();
    if envs.len() > 0 {
        info!(
            "with environment variables:\n{}",
            envs.map(|(var, val)| match val {
                Some(val) => format!(
                    "{}={}",
                    var.to_string_lossy(),
                    shell_escape::escape(val.to_string_lossy())
                ),
                None => format!("{}=''", var.to_string_lossy()), // explicitly removed
            })
            .collect::<Vec<_>>()
            .join("\n")
        );
    }
}

/// Add `cmd` to the script.
fn add_to_script(cmd: &Command, opt: &Opt) -> CVResult<()> {
    match &opt.script {
        None => Ok(()),
        Some(script) => {
            let cd_str = cmd
                .get_current_dir()
                .map(|dir| format!("cd '{}'", dir.to_string_lossy()));
            let envs = cmd
                .get_envs()
                .map(|(env, val)| {
                    match val {
                        Some(val) => {
                            // TODO: escape val properly?
                            format!(
                                "export {}='{}'",
                                env.to_string_lossy(),
                                val.to_string_lossy()
                            )
                        }
                        None => {
                            format!("export {}=''", env.to_string_lossy())
                        }
                    }
                })
                .collect::<Vec<_>>();

            let cmd_str = iter::once(cmd.get_program())
                .chain(cmd.get_args())
                .map(|s| shell_escape::escape(s.to_string_lossy()))
                .collect::<Vec<_>>()
                .join(" ");

            let mut file = script
                .lock()
                .map_err(|_| "Cannot acquire the script lock")?;

            let mut indent = 0;

            if cd_str != None || !envs.is_empty() {
                file.write_all(format!("{:ind$}(\n", "", ind = indent).as_bytes())?;
                indent += 4;
            }

            if let Some(cd_str) = &cd_str {
                file.write_all(format!("{:ind$}{}\n", "", cd_str, ind = indent).as_bytes())?;
            }

            for env_str in &envs {
                file.write_all(format!("{:ind$}{}\n", "", env_str, ind = indent).as_bytes())?;
            }

            file.write_all(format!("{:ind$}{}\n", "", cmd_str, ind = indent).as_bytes())?;

            if cd_str != None || !envs.is_empty() {
                indent -= 4;
                file.write_all(format!("{:ind$})\n", "", ind = indent).as_bytes())?;
            }

            Ok(())
        }
    }
}

/// Print each line of `Lines` using `info!`, prefixed with `prefix`.
pub fn info_lines(opt: &Opt, lvl: Verbosity, prefix: &str, lines: Lines) {
    for l in lines {
        info_at!(&opt, lvl, "{}{}", prefix, l);
    }
}

/// Run `cargo clean`.
pub fn clean(opt: &Opt) {
    info_at!(&opt, Verbosity::Informative, "Running `cargo clean`");
    Command::new("cargo")
        .arg("clean")
        .arg("--manifest-path")
        .arg(&opt.cargo_toml)
        .output_info_ignore_exit(&opt, Verbosity::Major)
        .ok(); // Discarding the error on purpose.
}

/// Find the name of the crate.
pub fn get_meta_package_name(opt: &Opt) -> CVResult<String> {
    let name = MetadataCommand::new()
        .manifest_path(&opt.cargo_toml)
        .features(CargoOpt::SomeFeatures(opt.features.clone()))
        .exec()?
        .root_package()
        .ok_or("no root package")?
        .name
        .replace(
            |c| match c {
                // Allowed characters.
                'a'..='z' | 'A'..='Z' | '0'..='9' | '_' => false,
                // Anything else will be replaced with the '_' character.
                _ => true,
            },
            "_",
        );

    Ok(name)
}

/// Find the target directory.
pub fn get_meta_target_directory(opt: &Opt) -> CVResult<PathBuf> {
    // FIXME: add '--cfg=verify' to RUSTFLAGS?
    let dir = MetadataCommand::new()
        .manifest_path(&opt.cargo_toml)
        .features(CargoOpt::SomeFeatures(opt.features.clone()))
        .exec()?
        .target_directory;

    Ok(dir)
}

/// Get name of default_host.
/// This is passed to cargo using "--target=..." and will be the name of the
/// directory within the target directory.
pub fn get_default_host(opt: &Opt) -> CVResult<String> {
    let mut cmd = Command::new("rustup");
    cmd.arg("show");

    let crate_path = opt.cargo_toml.parent().unwrap_or(Path::new("."));
    if crate_path != Path::new("") {
        cmd.current_dir(crate_path);
    }

    Ok(cmd
        .output_info(&opt, Verbosity::Trivial)?
        .0
        .lines()
        .find_map(|l| l.strip_prefix("Default host:").and_then(|l| Some(l.trim())))
        .ok_or("Unable to determine default host")?
        .to_string())
}

/// Count how many functions in `f`s are present in `bcfile`.
pub fn count_symbols(opt: &Opt, bcfile: &Path, fs: &[&str]) -> CVResult<usize> {
    info_at!(
        &opt,
        Verbosity::Trivial,
        "    Counting symbols {:?} in {}",
        fs,
        bcfile.to_string_lossy()
    );

    let mut cmd = Command::new(format!("llvm-nm-{}", opt.llvm_version));
    cmd.arg("--defined-only").arg(bcfile);
    let (stdout, _) = cmd.output_info(&opt, Verbosity::Trivial)?;

    let count = stdout
        .lines()
        .map(|l| l.split(" ").collect::<Vec<_>>())
        .filter(|l| l.len() == 3 && l[1] == "T" && fs.iter().any(|f| f == &l[2]))
        .count();

    info_at!(&opt, Verbosity::Trivial, "    Found {} functions", count);
    Ok(count)
}

/// Generate a list of tests in the crate by parsing the output of
/// `cargo test -- --list`
pub fn list_tests(opt: &Opt, target: &str) -> CVResult<Vec<String>> {
    let mut cmd = Command::new("cargo");
    cmd.arg("test").arg("--manifest-path").arg(&opt.cargo_toml);

    if !opt.features.is_empty() {
        cmd.arg("--features").arg(opt.features.join(","));
    }

    // Surprisingly, the effect of the following line is to prevent
    // consideration of doc tests from the list of tests.
    // We ignore doc tests because installing rustdoc to enable this
    // causes additional build problems and because we don't currently
    // expect doc tests to be based on property-based testing.
    cmd.arg("--all-targets");

    cmd.arg(format!("--target={}", target))
        .args(vec!["-v"; opt.verbose])
        .envs(get_build_envs(&opt)?)
        .args(&["--", "--list"]);
    // .arg("--exclude-should-panic")
    // .env("PATH", ...)

    lazy_static! {
        static ref TEST: Regex = Regex::new(r"(\S+):\s+test\s*$").unwrap();
    }

    // TODO: Python ignores bad exit codes
    let tests = cmd
        .output_info(&opt, Verbosity::Minor)?
        .0
        .lines()
        .filter_map(|l| {
            TEST.captures(l)
                .map(|caps| caps.get(1).unwrap().as_str().into())
        })
        .collect();

    Ok(tests)
}
