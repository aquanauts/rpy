#![deny(warnings)]

use std::fs;
use std::os::unix::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::process::exit;

use clap::{crate_authors, crate_description, crate_version, Parser};
use eyre::{ContextCompat, eyre, Report, Result};
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

fn run() -> Result<()> {
    let args = Opts::parse();
    let path = Path::new(&args.script);
    if !path.is_file() {
        return Err(eyre!("Unable to open input file: {}", path.display()));
    }
    let toml = find_toml(path.parent().wrap_err("Unable to get parent dir")?)
        .expect(format!("Unable to find a pyproject.toml for {}", path.display()).as_str());
    let project_root = toml.parent().unwrap();
    let toml_doc = fs::read_to_string(toml.as_path())?;
    let config: Config = toml::from_str(&toml_doc)
        .expect("Unable to read toml document or find the rpy.tool configuration in it");
    let py_config = config.tool.rpy;
    let python = project_root.join(Path::new(&py_config.interpreter));
    let src_root = project_root.join(Path::new(&py_config.source_root));
    if args.verbose {
        println!("python: {}", python.display());
        println!("src_root: {}", src_root.display());
    }
    match py_config.pre_run {
        Some(str) => {
            if args.verbose {
                println!("running pre_run: {}", str);
            }
            let args = ["-eu", "-o", "pipefail", "-c", &str];
            let res = Command::new("bash")
                .args(args)
                .current_dir(project_root)
                .status()?;
            if !res.success() {
                return Err(eyre!("Pre-run step '{}' failed with exit code {}", str, res.code().unwrap()));
            }
        }
        None => {}
    }

    // would be nice if I could work out how to prepend in-place which I know must be possible, like
    // a ranges thing I can pass to args. I nearly got it working with slices but I suck.
    // .args(&[&path[..], &args.args[..]].concat()) something like that?
    let mut zomg = args.args.clone();
    zomg.insert(0, String::from(path.to_str().unwrap()));
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
