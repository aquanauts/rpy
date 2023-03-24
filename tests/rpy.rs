use std::{env, path::Path, process::Command};

const SRC_ROOT: &'static str = env!("CARGO_MANIFEST_DIR");
const RPY_EXE: &'static str = env!("CARGO_BIN_EXE_rpy");

#[test]
fn should_fail_with_no_pyproject_toml() {
    let output = Command::new(RPY_EXE).current_dir("/").output().unwrap();
    let stdout = std::str::from_utf8(&output.stdout).unwrap();
    let stderr = std::str::from_utf8(&output.stderr).unwrap();
    assert_eq!(stdout, "");
    assert_eq!(
        stderr,
        "[rpy] Error: Unable to find pyproject.toml from /\n"
    );
    assert_eq!(output.status.code().unwrap(), 1);
}

#[test]
fn should_fail_with_empty_pyproject_toml() {
    let output = Command::new(RPY_EXE)
        .current_dir(Path::new(SRC_ROOT).join("test_data"))
        .output()
        .unwrap();
    let stdout = std::str::from_utf8(&output.stdout).unwrap();
    let stderr = std::str::from_utf8(&output.stderr).unwrap();
    assert_eq!(stdout, "");
    assert_eq!(
        stderr,
        "[rpy] Error: Unable to read toml document or find the rpy.tool configuration in it\n"
    );
    assert_eq!(output.status.code().unwrap(), 1);
}

#[test]
fn should_work_with_simple_pyproject_toml() {
    let output = Command::new(RPY_EXE)
        .current_dir(Path::new(SRC_ROOT).join("test_data/simple"))
        .arg("badger.sh")
        .output()
        .unwrap();
    let stdout = std::str::from_utf8(&output.stdout).unwrap();
    let stderr = std::str::from_utf8(&output.stderr).unwrap();
    assert_eq!(
        stdout,
        "badger\n".to_string()
            + SRC_ROOT
            + "/test_data/simple/\n1\n1\n"
            + &env::var("PATH").unwrap()
            + "\n"
    );
    assert_eq!(stderr, "");
    assert_eq!(output.status.code().unwrap(), 0);
}

#[test]
fn should_work_with_pre_run_pyproject_toml() {
    let output = Command::new(RPY_EXE)
        .current_dir(Path::new(SRC_ROOT).join("test_data/pre_run"))
        .arg("badger.sh")
        .output()
        .unwrap();
    let stdout = std::str::from_utf8(&output.stdout).unwrap();
    let stderr = std::str::from_utf8(&output.stderr).unwrap();
    assert_eq!(
        stdout,
        "prerun ".to_string() + SRC_ROOT + "/test_data/pre_run\nbadger\n"
    );
    assert_eq!(stderr, "");
    assert_eq!(output.status.code().unwrap(), 0);
}

#[test]
fn should_work_with_source_root_pyproject_toml() {
    let output = Command::new(RPY_EXE)
        .current_dir(Path::new(SRC_ROOT).join("test_data/source_root"))
        .arg("badger.sh")
        .output()
        .unwrap();
    let stdout = std::str::from_utf8(&output.stdout).unwrap();
    let stderr = std::str::from_utf8(&output.stderr).unwrap();
    assert_eq!(stderr, "");
    assert_eq!(
        stdout,
        "badger\n".to_string() + SRC_ROOT + "/test_data/source_root/src\n"
    );
    assert_eq!(output.status.code().unwrap(), 0);
}

#[test]
fn should_work_with_bin_path_pyproject_toml() {
    let output = Command::new(RPY_EXE)
        .current_dir(Path::new(SRC_ROOT).join("test_data/bin_path"))
        .arg("badger.sh")
        .output()
        .unwrap();
    let stdout = std::str::from_utf8(&output.stdout).unwrap();
    let stderr = std::str::from_utf8(&output.stderr).unwrap();
    assert_eq!(
        stdout,
        "badger\n".to_string()
            + SRC_ROOT
            + "/test_data/bin_path/bin:"
            + &env::var("PATH").unwrap()
            + "\n"
    );
    assert_eq!(stderr, "");
    assert_eq!(output.status.code().unwrap(), 0);
}

#[test]
fn should_work_with_rel_interp_pyproject_toml() {
    let output = Command::new(RPY_EXE)
        .current_dir(Path::new(SRC_ROOT).join("test_data/rel_interp"))
        .arg("badger.sh")
        .arg("foo")
        .output()
        .unwrap();
    let stdout = std::str::from_utf8(&output.stdout).unwrap();
    let stderr = std::str::from_utf8(&output.stderr).unwrap();
    assert_eq!(stderr, "");
    assert_eq!(stdout, "interp\nbadger.sh foo\n");
    assert_eq!(output.status.code().unwrap(), 0);
}

#[test]
fn should_work_with_rel_rpy_interpreter_environment_override() {
    let output = Command::new(RPY_EXE)
        .arg(Path::new(SRC_ROOT).join("test_data/rel_interp/badger.sh"))
        .arg("foo")
        .env("RPY_INTERPRETER", "bin/interp2")
        .output()
        .unwrap();
    let stdout = std::str::from_utf8(&output.stdout).unwrap();
    let stderr = std::str::from_utf8(&output.stderr).unwrap();
    assert_eq!(stderr, "");
    assert_eq!(output.status.code().unwrap(), 0);
    assert_eq!(
        stdout,
        "interp2\n".to_string()
            + &Path::new(SRC_ROOT)
                .join("test_data/rel_interp/badger.sh")
                .display()
                .to_string()
            + " foo\n"
    );
}

#[test]
fn should_work_with_abs_rpy_interpreter_environment_override() {
    let output = Command::new(RPY_EXE)
        .arg(Path::new(SRC_ROOT).join("test_data/rel_interp/badger.sh"))
        .arg("foo")
        .env("RPY_INTERPRETER", "interp2")
        .env(
            "PATH",
            Path::new(SRC_ROOT).join("test_data/rel_interp/bin").display().to_string()
                + ":"
                + &env::var("PATH").unwrap(),
        )
        .output()
        .unwrap();
    let stdout = std::str::from_utf8(&output.stdout).unwrap();
    let stderr = std::str::from_utf8(&output.stderr).unwrap();
    assert_eq!(stderr, "");
    assert_eq!(output.status.code().unwrap(), 0);
    assert_eq!(
        stdout,
        "interp2\n".to_string()
            + &Path::new(SRC_ROOT)
                .join("test_data/rel_interp/badger.sh")
                .display()
                .to_string()
            + " foo\n"
    );
}

#[test]
fn should_work_using_simple_shebang() {
    let output = Command::new(Path::new(SRC_ROOT).join("test_data/shebang/bin/badger"))
        .current_dir("/")
        .env(
            "PATH",
            Path::new(RPY_EXE).parent().unwrap().display().to_string()
                + ":"
                + &env::var("PATH").unwrap(),
        )
        .output()
        .unwrap();
    let stdout = std::str::from_utf8(&output.stdout).unwrap();
    let stderr = std::str::from_utf8(&output.stderr).unwrap();
    assert_eq!(
        stdout,
        "interp\n".to_string()
            + &Path::new(SRC_ROOT)
                .join("test_data/shebang/bin/badger")
                .display()
                .to_string()
            + "\n"
    );
    assert_eq!(stderr, "");
    assert_eq!(output.status.code().unwrap(), 0);
}
