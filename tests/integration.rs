use assert_cmd::prelude::*;
use std::process::Command;
use tempfile::NamedTempFile;

pub type TestResult = Result<(), Box<dyn std::error::Error>>;

#[test]
fn lox_constant() -> TestResult {
    let file = NamedTempFile::new()?;
    let name = file.path();

    let mut cmd = Command::cargo_bin("rox")?;
    cmd.arg(name);
    cmd.env(
        "PWD",
        std::env::current_dir().expect("Can't get current dir"),
    );
    let output = cmd.output()?;

    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();

    println!("stdout: {}", stdout);
    println!("stderr: {}", stderr);

    assert!(output.status.success());

    assert!(stdout.trim().contains(
        "OP CODE:OP_CONSTANT - Line number 1 - Constant pool index:0 and the value:Number(1.2)"
    ));

    Ok(())
}
