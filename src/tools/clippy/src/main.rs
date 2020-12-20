#![feature(bool_to_option)]
#![feature(command_access)]
#![cfg_attr(feature = "deny-warnings", deny(warnings))]
// warn on lints, that are included in `rust-lang/rust`s bootstrap
#![warn(rust_2018_idioms, unused_lifetimes)]

use rustc_tools_util::VersionInfo;
use std::env;
use std::ffi::OsString;
use std::path::PathBuf;
use std::process::{self, Command};

const CARGO_CLIPPY_HELP: &str = r#"Checks a package to catch common mistakes and improve your Rust code.

Usage:
    cargo clippy [options] [--] [<opts>...]

Common options:
    -h, --help               Print this message
    -V, --version            Print version info and exit

Other options are the same as `cargo check`.

To allow or deny a lint from the command line you can use `cargo clippy --`
with:

    -W --warn OPT       Set lint warnings
    -A --allow OPT      Set lint allowed
    -D --deny OPT       Set lint denied
    -F --forbid OPT     Set lint forbidden

You can use tool lints to allow or deny lints from your code, eg.:

    #[allow(clippy::needless_lifetimes)]
"#;

fn show_help() {
    println!("{}", CARGO_CLIPPY_HELP);
}

fn show_version() {
    let version_info = rustc_tools_util::get_version_info!();
    println!("{}", version_info);
}

pub fn main() {
    // Check for version and help flags even when invoked as 'cargo-clippy'
    if env::args().any(|a| a == "--help" || a == "-h") {
        show_help();
        return;
    }

    if env::args().any(|a| a == "--version" || a == "-V") {
        show_version();
        return;
    }

    if let Err(code) = process(env::args().skip(2)) {
        process::exit(code);
    }
}

struct ClippyCmd {
    unstable_options: bool,
    cargo_subcommand: &'static str,
    args: Vec<String>,
    clippy_args: Option<String>,
}

impl ClippyCmd {
    fn new<I>(mut old_args: I) -> Self
    where
        I: Iterator<Item = String>,
    {
        let mut cargo_subcommand = "check";
        let mut unstable_options = false;
        let mut args = vec![];

        for arg in old_args.by_ref() {
            match arg.as_str() {
                "--fix" => {
                    cargo_subcommand = "fix";
                    continue;
                },
                "--" => break,
                // Cover -Zunstable-options and -Z unstable-options
                s if s.ends_with("unstable-options") => unstable_options = true,
                _ => {},
            }

            args.push(arg);
        }

        if cargo_subcommand == "fix" && !unstable_options {
            panic!("Usage of `--fix` requires `-Z unstable-options`");
        }

        // Run the dogfood tests directly on nightly cargo. This is required due
        // to a bug in rustup.rs when running cargo on custom toolchains. See issue #3118.
        if env::var_os("CLIPPY_DOGFOOD").is_some() && cfg!(windows) {
            args.insert(0, "+nightly".to_string());
        }

        let mut clippy_args = old_args.collect::<Vec<String>>().join(" ");
        if cargo_subcommand == "fix" && !clippy_args.contains("--no-deps") {
            clippy_args = format!("{} --no-deps", clippy_args);
        }

        let has_args = !clippy_args.is_empty();
        ClippyCmd {
            unstable_options,
            cargo_subcommand,
            args,
            clippy_args: has_args.then_some(clippy_args),
        }
    }

    fn path_env(&self) -> &'static str {
        if self.unstable_options {
            "RUSTC_WORKSPACE_WRAPPER"
        } else {
            "RUSTC_WRAPPER"
        }
    }

    fn path() -> PathBuf {
        let mut path = env::current_exe()
            .expect("current executable path invalid")
            .with_file_name("clippy-driver");

        if cfg!(windows) {
            path.set_extension("exe");
        }

        path
    }

    fn target_dir() -> Option<(&'static str, OsString)> {
        env::var_os("CLIPPY_DOGFOOD")
            .map(|_| {
                env::var_os("CARGO_MANIFEST_DIR").map_or_else(
                    || std::ffi::OsString::from("clippy_dogfood"),
                    |d| {
                        std::path::PathBuf::from(d)
                            .join("target")
                            .join("dogfood")
                            .into_os_string()
                    },
                )
            })
            .map(|p| ("CARGO_TARGET_DIR", p))
    }

    fn into_std_cmd(self, rustflags: Option<String>) -> Command {
        let mut cmd = Command::new("cargo");

        cmd.env(self.path_env(), Self::path())
            .envs(ClippyCmd::target_dir())
            .arg(self.cargo_subcommand)
            .args(&self.args);

        // HACK: pass Clippy args to the driver *also* through RUSTFLAGS.
        // This guarantees that new builds will be triggered when Clippy flags change.
        if let Some(clippy_args) = self.clippy_args {
            cmd.env(
                "RUSTFLAGS",
                rustflags.map_or(clippy_args.clone(), |flags| format!("{} {}", clippy_args, flags)),
            );
            cmd.env("CLIPPY_ARGS", clippy_args);
        }

        cmd
    }
}

fn process<I>(old_args: I) -> Result<(), i32>
where
    I: Iterator<Item = String>,
{
    let cmd = ClippyCmd::new(old_args);

    let mut cmd = cmd.into_std_cmd(env::var("RUSTFLAGS").ok());

    let exit_status = cmd
        .spawn()
        .expect("could not run cargo")
        .wait()
        .expect("failed to wait for cargo?");

    if exit_status.success() {
        Ok(())
    } else {
        Err(exit_status.code().unwrap_or(-1))
    }
}

#[cfg(test)]
mod tests {
    use super::ClippyCmd;
    use std::ffi::OsStr;

    #[test]
    #[should_panic]
    fn fix_without_unstable() {
        let args = "cargo clippy --fix".split_whitespace().map(ToString::to_string);
        let _ = ClippyCmd::new(args);
    }

    #[test]
    fn fix_unstable() {
        let args = "cargo clippy --fix -Zunstable-options"
            .split_whitespace()
            .map(ToString::to_string);
        let cmd = ClippyCmd::new(args);

        assert_eq!("fix", cmd.cargo_subcommand);
        assert_eq!("RUSTC_WORKSPACE_WRAPPER", cmd.path_env());
        assert!(cmd.args.iter().any(|arg| arg.ends_with("unstable-options")));
    }

    #[test]
    fn fix_implies_no_deps() {
        let args = "cargo clippy --fix -Zunstable-options"
            .split_whitespace()
            .map(ToString::to_string);
        let cmd = ClippyCmd::new(args);

        assert!(cmd.clippy_args.unwrap().contains("--no-deps"));
    }

    #[test]
    fn no_deps_not_duplicated_with_fix() {
        let args = "cargo clippy --fix -Zunstable-options -- --no-deps"
            .split_whitespace()
            .map(ToString::to_string);
        let cmd = ClippyCmd::new(args);

        assert_eq!(1, cmd.clippy_args.unwrap().matches("--no-deps").count());
    }

    #[test]
    fn check() {
        let args = "cargo clippy".split_whitespace().map(ToString::to_string);
        let cmd = ClippyCmd::new(args);

        assert_eq!("check", cmd.cargo_subcommand);
        assert_eq!("RUSTC_WRAPPER", cmd.path_env());
    }

    #[test]
    fn check_unstable() {
        let args = "cargo clippy -Zunstable-options"
            .split_whitespace()
            .map(ToString::to_string);
        let cmd = ClippyCmd::new(args);

        assert_eq!("check", cmd.cargo_subcommand);
        assert_eq!("RUSTC_WORKSPACE_WRAPPER", cmd.path_env());
    }

    #[test]
    fn clippy_args_into_rustflags() {
        let args = "cargo clippy -- -W clippy::as_conversions"
            .split_whitespace()
            .map(ToString::to_string);
        let cmd = ClippyCmd::new(args);

        let rustflags = None;
        let cmd = cmd.into_std_cmd(rustflags);

        assert!(cmd
            .get_envs()
            .any(|(key, val)| key == "RUSTFLAGS" && val == Some(OsStr::new("-W clippy::as_conversions"))));
    }

    #[test]
    fn clippy_args_respect_existing_rustflags() {
        let args = "cargo clippy -- -D clippy::await_holding_lock"
            .split_whitespace()
            .map(ToString::to_string);
        let cmd = ClippyCmd::new(args);

        let rustflags = Some(r#"--cfg feature="some_feat""#.into());
        let cmd = cmd.into_std_cmd(rustflags);

        assert!(cmd.get_envs().any(|(key, val)| key == "RUSTFLAGS"
            && val == Some(OsStr::new(r#"-D clippy::await_holding_lock --cfg feature="some_feat""#))));
    }

    #[test]
    fn no_env_change_if_no_clippy_args() {
        let args = "cargo clippy".split_whitespace().map(ToString::to_string);
        let cmd = ClippyCmd::new(args);

        let rustflags = Some(r#"--cfg feature="some_feat""#.into());
        let cmd = cmd.into_std_cmd(rustflags);

        assert!(!cmd
            .get_envs()
            .any(|(key, _)| key == "RUSTFLAGS" || key == "CLIPPY_ARGS"));
    }

    #[test]
    fn no_env_change_if_no_clippy_args_nor_rustflags() {
        let args = "cargo clippy".split_whitespace().map(ToString::to_string);
        let cmd = ClippyCmd::new(args);

        let rustflags = None;
        let cmd = cmd.into_std_cmd(rustflags);

        assert!(!cmd
            .get_envs()
            .any(|(key, _)| key == "RUSTFLAGS" || key == "CLIPPY_ARGS"))
    }
}
