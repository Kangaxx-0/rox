use assert_cmd::prelude::*;
use std::io::Write;
use std::process::Command;
use tempfile::NamedTempFile;

type TestResult = Result<(), Box<dyn std::error::Error>>;

fn run_test_contains(input: &str, expected: &str) -> TestResult {
    let mut file = NamedTempFile::new()?;
    let name = file.path();

    let mut cmd = Command::cargo_bin("rox")?;
    cmd.arg(name);

    writeln!(file, "{}", input)?;

    let output = cmd.output()?;

    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();

    println!("stdout: {}", stdout);
    println!("stderr: {}", stderr);

    assert!(output.status.success());

    assert!(stdout.contains(expected));

    Ok(())
}
#[test]
fn rox_constant_number() -> TestResult {
    run_test_contains("1", "Number(1.0)")
}

#[test]
fn rox_arithmetic_plus() -> TestResult {
    run_test_contains(r"1+2", "Number(3.0)")
}

#[test]
fn rox_arithmetic_minus() -> TestResult {
    run_test_contains(r"1-2", "Number(-1.0)")
}
