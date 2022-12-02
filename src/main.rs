#![deny(warnings)]

use std::fs;
use std::os::unix::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::process::exit;

use clap::{crate_authors, crate_description, crate_version, Parser};
use eyre::{ContextCompat, eyre, Report, Result, WrapErr};
use serde::Deserialize;

#[derive(Parser, Debug)]
#[clap(
about = crate_description ! (),
version = crate_version ! (),
author = crate_authors ! (),
allow_hyphen_values = true
)]
struct Opts {
    #[clap(short = 'v', long)]
    verbose: bool,
    /// python script to run
    script: String,
    /// args
    #[clap(trailing_var_arg = true)]
    args: Vec<String>,
}

fn find_toml(path: &Path) -> Option<PathBuf> {
    let toml = Path::new("pyproject.toml");
    let toml_path = path.join(toml);
    if toml_path.is_file() {
        return Some(toml_path);
    }
    match path.parent() {
        Some(path) => find_toml(path),
        None => None,
    }
}

#[derive(Deserialize, Debug)]
struct Config {
    tool: Tool,
}

#[derive(Deserialize, Debug)]
struct Tool {
    rpy: PyConfig,
}

#[derive(Deserialize, Debug)]
struct PyConfig {
    interpreter: String,
    source_root: String,
    pre_run: Option<String>,
}

fn pre_run(run_dir: &Path, pre_run_cmd: &str, verbose: bool) -> Result<()> {
    if verbose {
        println!("running pre_run: {}", pre_run_cmd);
    }
    let args = ["-eu", "-o", "pipefail", "-c", &pre_run_cmd];
    let res = Command::new("bash")
        .args(args)
        .current_dir(run_dir)
        .status()?;
    if !res.success() {
        return Err(eyre!("Pre-run step '{}' failed with exit code {}", pre_run_cmd, res.code().unwrap()));
    }
    Ok(())
}

fn run() -> Result<()> {
    let cmdline_args = Opts::parse();
    let script_path = Path::new(&cmdline_args.script);
    if !script_path.is_file() {
        return Err(eyre!("Unable to open input file: {}", script_path.display()));
    }
    let toml = find_toml(script_path.parent().wrap_err("Unable to get script parent dir")?)
        .wrap_err_with(|| format!("Unable to find a pyproject.toml for {}", script_path.display()))?;
    let project_root = toml.parent().wrap_err("Unable to get project root")?;
    let toml_doc = fs::read_to_string(toml.as_path()).wrap_err("Unable to read pyproject.toml")?;
    let config: Config = toml::from_str(&toml_doc)
        .wrap_err("Unable to read toml document or find the rpy.tool configuration in it")?;
    let py_config = config.tool.rpy;
    let python = project_root.join(Path::new(&py_config.interpreter));
    let src_root = project_root.join(Path::new(&py_config.source_root));
    if cmdline_args.verbose {
        println!("python: {}", python.display());
        println!("src_root: {}", src_root.display());
    }
    match py_config.pre_run {
        Some(str) => { pre_run(project_root, &str, cmdline_args.verbose)?; }
        None => {}
    }

    // would be nice if I could work out how to prepend in-place which I know must be possible, like
    // a ranges thing I can pass to args. I nearly got it working with slices but I suck.
    // .args(&[&path[..], &args.args[..]].concat()) something like that?
    let mut zomg = cmdline_args.args.clone();
    zomg.insert(0, String::from(script_path.to_str().unwrap()));
    Err(Report::new(Command::new(python)
        .args(&zomg)
        .env("PYTHONPATH", &src_root)
        .exec()))
}

fn main() {
    if let Err(e) = run() {
        eprintln!("[rpy] Error: {e}");
        exit(1);
    }
}
