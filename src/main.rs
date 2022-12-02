#![deny(warnings)]

use std::os::unix::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::exit;
use std::process::Command;
use std::{fmt, fs};

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
    py: PyConfig,
}

#[derive(Deserialize, Debug)]
struct PyConfig {
    python_interpreter: String,
    source_root: String,
    pre_run: Option<String>,
}

#[derive(Debug, Clone)]
struct PyError;

// Generation of an error is completely separate from how it is displayed.
// There's no need to be concerned about cluttering complex logic with the display style.
//
// Note that we don't store any extra info about the errors. This means we can't state
// which string failed to parse without modifying our types to carry that information.
impl fmt::Display for PyError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "invalid first item to double")
    }
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
        .expect("Unable to read toml document or find the py.tool configuration in it");
    let py_config = config.tool.py;
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
            let args = ["-c", &str];
            Command::new("bash")
                .args(args)
                .current_dir(project_root)
                .exec();
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
        eprintln!("[py] Error: {e}");
        exit(1);
    }
}
