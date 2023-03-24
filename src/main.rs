#![deny(warnings)]

use std::os::unix::process::CommandExt;
use std::path::Path;
use std::process::exit;
use std::process::Command;
use std::{env, fs};

use eyre::{eyre, ContextCompat, Report, Result, WrapErr};
use serde::Deserialize;

use crate::rpy::Rpy;

mod rpy;

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
    source_root: Option<String>,
    pre_run: Option<String>,
}

fn pre_run(run_dir: &Path, pre_run_cmd: &str, verbose: bool) -> Result<()> {
    if verbose {
        println!("running pre_run: {pre_run_cmd}");
    }
    let args = ["-eu", "-o", "pipefail", "-c", pre_run_cmd];
    let res = Command::new("bash")
        .args(args)
        .current_dir(run_dir)
        .status()?;
    if !res.success() {
        return Err(eyre!(
            "Pre-run step '{}' failed with exit code {}",
            pre_run_cmd,
            res.code().unwrap()
        ));
    }
    Ok(())
}

fn run() -> Result<()> {
    let cmdline_args = Rpy::parse(env::args().skip(1).collect());
    if cmdline_args.print_banner {
        println!("Running under rpy version {}", env!("CARGO_PKG_VERSION"));
    }

    let verbose = env::var("RPY_VERBOSE").map_or(false, |x| x != "0");
    let toml = cmdline_args.find_toml()?;
    let project_root = toml.parent().wrap_err("Unable to get project root")?;
    if verbose {
        println!("project root: {}", project_root.display());
        println!("toml: {}", toml.display());
    }
    let toml_doc = fs::read_to_string(toml.as_path()).wrap_err("Unable to read pyproject.toml")?;
    let config: Config = toml::from_str(&toml_doc)
        .wrap_err("Unable to read toml document or find the rpy.tool configuration in it")?;
    let py_config = config.tool.rpy;
    let python = project_root.join(Path::new(&py_config.interpreter));
    let src_root = project_root.join(Path::new(
        &py_config.source_root.unwrap_or_else(|| ".".to_string()),
    ));
    if verbose {
        println!("python: {}", python.display());
        println!("src_root: {}", src_root.display());
    }
    if let Some(str) = py_config.pre_run {
        pre_run(project_root, &str, verbose).wrap_err("Unable to run pre_run step")?;
    }

    Err(Report::new(
        Command::new(python)
            .args(cmdline_args.make_args())
            .env("PYTHONPATH", &src_root)
            .env("PYTHONNOUSERSITE", "1")
            .exec(),
    ))
    .wrap_err("Unable to launch process")
}

fn main() {
    if let Err(e) = run() {
        eprintln!("[rpy] Error: {e}");
        exit(1);
    }
}
