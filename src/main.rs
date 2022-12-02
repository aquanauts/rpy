#![deny(warnings)]

use std::{fs};
use std::os::unix::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::process::exit;

use clap::{crate_authors, crate_description, crate_version, Parser};
use eyre::eyre;
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
    python_interpreter: String,
    source_root: String,
    pre_run: Option<String>,
}

fn run() -> eyre::Result<()> {
    let args = Opts::parse();
    let path = Path::new(&args.script);
    if !path.is_file() {
        return Err(eyre!("Unable to open input file: {}", path.display()));
    }
    let toml = find_toml(path.parent().unwrap())
        .expect(format!("Unable to find a pyproject.toml for {}", path.display()).as_str());
    let project_root = toml.parent().unwrap();
    let toml_doc = fs::read_to_string(toml.as_path())?;
    let config: Config = toml::from_str(&toml_doc)
        .expect("Unable to read toml document or find the rpy.tool configuration in it");
    let py_config = config.tool.rpy;
    let python = project_root.join(Path::new(&py_config.python_interpreter));
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
            let res = Command::new("false")
                .args(args)
                .current_dir(project_root)
                .status();
                // .expect(&format!("pre_run command '{}' failed", str));
            println!("{:?}", res);
        }
        None => {}
    }
    let mut zomg = args.args.clone();
    zomg.insert(0, String::from(path.to_str().unwrap()));
    // args.
    // let path : Vec<String> = Vec::from(&[&path.to_str().unwrap()]);
    Command::new(python)
        // .args(&[&path[..], &args.args[..]].concat())
        .args(&zomg)
        .env("PYTHONPATH", &src_root)
        .exec();
    Ok(())
}

fn main() {
    if let Err(e) = run() {
        eprintln!("[rpy] Error: {e}");
        exit(1);
    }
}
