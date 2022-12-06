#![deny(warnings)]

use std::path::{Path, PathBuf};
use std::{env, fs};

use eyre::{eyre, ContextCompat, Result, WrapErr};

#[derive(Debug, PartialEq)]
pub enum InvocationType {
    Interactive,
    Module(String),
    Command(String),
    File(String),
}

#[derive(Debug, PartialEq)]
pub struct Opts {
    python_args: Vec<String>,
    command_args: Vec<String>,
    invocation_type: InvocationType,
    // print_banner: bool TODO
}

enum PythonArg {
    SingleChars(String),
    LongArg(String),
    Module(String),
    Command(String),
    File(String),
    Error,
}

impl Opts {
    pub fn parse(options_orig: Vec<String>) -> Opts {
        let mut options = options_orig.clone();
        let mut python_args: Vec<String> = vec![];
        let mut invocation_type: Option<InvocationType> = None;
        while invocation_type.is_none() && !options.is_empty() {
            let (arg, num_consumed) = Self::parse_args(options.first().unwrap(), options.get(1));
            python_args.extend(options.drain(0..num_consumed));
            match arg {
                PythonArg::Module(module) => invocation_type = Some(InvocationType::Module(module)),
                PythonArg::Command(cmd) => invocation_type = Some(InvocationType::Command(cmd)),
                PythonArg::File(file) => invocation_type = Some(InvocationType::File(file)),
                PythonArg::SingleChars(_) => {}
                PythonArg::LongArg(_) => {}
                PythonArg::Error => invocation_type = Some(InvocationType::Interactive),
            }
        }
        Opts {
            python_args,
            command_args: options,
            invocation_type: invocation_type.unwrap_or(InvocationType::Interactive),
        }
    }

    fn long_arg_count(arg: &String) -> usize {
        match arg.as_str() {
            "--check-hash-based-pycs" => 2,
            _ => 1,
        }
    }

    fn parse_args(arg: &String, next_arg: Option<&String>) -> (PythonArg, usize) {
        if arg.chars().nth(0) != Some('-') || arg == "-" {
            return (PythonArg::File(arg.clone()), 1);
        }
        if arg == "--" {
            return (PythonArg::Error, 1);
        }
        let second_char = arg.chars().nth(1);
        match second_char {
            Some('-') => (PythonArg::LongArg(arg.clone()), Self::long_arg_count(arg)),
            Some('c') => {
                if arg.len() > 2 {
                    return (PythonArg::Command(arg[2..].to_string()), 1);
                }
                if next_arg.is_none() {
                    return (PythonArg::Error, 1);
                }
                (PythonArg::Command(next_arg.unwrap().clone()), 2)
            }
            Some('m') => {
                if arg.len() > 2 {
                    return (PythonArg::Module(arg[2..].to_string()), 1);
                }
                if next_arg.is_none() {
                    return (PythonArg::Error, 1);
                }
                (PythonArg::Module(next_arg.unwrap().clone()), 2)
            }
            _ => (PythonArg::SingleChars(arg.clone()), 1),
        }
    }

    fn find_toml_for_path(path: &Path) -> Option<PathBuf> {
        let toml = Path::new("pyproject.toml");
        let toml_path = path.join(toml);
        if toml_path.is_file() {
            return Some(toml_path);
        }
        match path.parent() {
            Some(path) => Self::find_toml_for_path(path),
            None => None,
        }
    }
    pub fn find_toml(&self) -> Result<PathBuf> {
        let path = match &self.invocation_type {
            InvocationType::Interactive
            | InvocationType::Module(_)
            | InvocationType::Command(_) => env::current_dir().wrap_err("Unable to get cwd")?,
            InvocationType::File(filename) => {
                let script_path = fs::canonicalize(Path::new(&filename))
                    .wrap_err("unable to resolve project root")?;
                if !script_path.is_file() {
                    return Err(eyre!(
                        "Unable to open input file: {}",
                        script_path.display()
                    ));
                }
                script_path
                    .parent()
                    .wrap_err("Unable to get script parent dir")?
                    .to_path_buf()
            }
        };
        Self::find_toml_for_path(&path).wrap_err(format!("Unable to find pyproject.toml from {}", path.display()))
    }

    pub fn make_args(&self) -> Vec<&String> {
        let mut args = vec![];
        args.extend(&self.python_args[..]);
        args.extend(&self.command_args[..]);
        args
    }
}

#[cfg(test)]
mod tests {
    use super::{InvocationType, Opts};

    #[test]
    fn should_parse_no_args() {
        assert_eq!(
            Opts::parse(vec![]),
            Opts {
                python_args: vec![],
                command_args: vec![],
                invocation_type: InvocationType::Interactive,
            }
        );
    }

    #[test]
    fn should_parse_simple_filename() {
        assert_eq!(
            Opts::parse(vec!["some_file.py".into()]),
            Opts {
                python_args: vec!["some_file.py".into()],
                command_args: vec![],
                invocation_type: InvocationType::File("some_file.py".into()),
            }
        );
        assert_eq!(
            Opts::parse(vec!["some_file.py".into(), "arg".into()]),
            Opts {
                python_args: vec!["some_file.py".into()],
                command_args: vec!["arg".into()],
                invocation_type: InvocationType::File("some_file.py".into()),
            }
        );
    }

    #[test]
    fn should_parse_filename_with_args_and_opts() {
        assert_eq!(
            Opts::parse(vec![
                "-i".into(),
                "-d".into(),
                "-s".into(),
                "some_file.py".into(),
                "arg1".into(),
                "arg2".into()
            ]),
            Opts {
                python_args: vec!["-i".into(), "-d".into(), "-s".into(), "some_file.py".into()],
                command_args: vec!["arg1".into(), "arg2".into()],
                invocation_type: InvocationType::File("some_file.py".into()),
            }
        );
    }

    #[test]
    fn should_parse_double_dash_even_with_nothing_else() {
        // Should pass this combo on to python unmolested as an "interactive" at least so python
        // can give its error message.
        assert_eq!(
            Opts::parse(vec!["--".into(), "moo".into(), "foo".into()]),
            Opts {
                python_args: vec!["--".into()],
                command_args: vec!["moo".into(), "foo".into()],
                invocation_type: InvocationType::Interactive,
            }
        );
    }

    #[test]
    fn should_parse_simple_module() {
        assert_eq!(
            Opts::parse(vec!["-m".into(), "some.module".into()]),
            Opts {
                python_args: vec!["-m".into(), "some.module".into()],
                command_args: vec![],
                invocation_type: InvocationType::Module("some.module".into()),
            }
        );
        assert_eq!(
            Opts::parse(vec!["-msome.module".into()]),
            Opts {
                python_args: vec!["-msome.module".into()],
                command_args: vec![],
                invocation_type: InvocationType::Module("some.module".into()),
            }
        );
    }

    #[test]
    fn should_parse_module_with_args_and_opts() {
        assert_eq!(
            Opts::parse(vec![
                "-i".into(),
                "-d".into(),
                "-s".into(),
                "-msome.module".into(),
                "arg1".into(),
                "arg2".into()
            ]),
            Opts {
                python_args: vec![
                    "-i".into(),
                    "-d".into(),
                    "-s".into(),
                    "-msome.module".into()
                ],
                command_args: vec!["arg1".into(), "arg2".into()],
                invocation_type: InvocationType::Module("some.module".into()),
            }
        );
    }

    #[test]
    fn should_parse_simple_command() {
        let cmd = "print(\"hello world\")";
        assert_eq!(
            Opts::parse(vec!["-c".into(), cmd.into()]),
            Opts {
                python_args: vec!["-c".into(), cmd.into()],
                command_args: vec![],
                invocation_type: InvocationType::Command(cmd.into()),
            }
        );
        assert_eq!(
            Opts::parse(vec!["-c".to_string() + &cmd]),
            Opts {
                python_args: vec!["-c".to_string() + &cmd],
                command_args: vec![],
                invocation_type: InvocationType::Command(cmd.into()),
            }
        );
    }

    #[test]
    fn should_parse_commande_with_args_and_opts() {
        let cmd = "print(\"hello world\")";
        assert_eq!(
            Opts::parse(vec![
                "-i".into(),
                "-d".into(),
                "-s".into(),
                "-c".to_string(),
                cmd.into(),
                "arg1".into(),
                "arg2".into()
            ]),
            Opts {
                python_args: vec![
                    "-i".into(),
                    "-d".into(),
                    "-s".into(),
                    "-c".to_string(),
                    cmd.into()
                ],
                command_args: vec!["arg1".into(), "arg2".into()],
                invocation_type: InvocationType::Command(cmd.into()),
            }
        );
    }
}
