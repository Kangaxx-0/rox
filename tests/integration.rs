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

pub fn fail_test(input: &str, expected: &str) -> TestResult {
    let mut file = NamedTempFile::new()?;
    let name = file.path();

    let mut cmd = Command::cargo_bin("rox")?;
    cmd.arg(name);
    cmd.env(
        "PWD",
        std::env::current_dir().expect("Can't get current dir"),
    );

    writeln!(file, "{}", input)?;

    let output = cmd.output()?;

    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();

    println!("stdout: {}", stdout);
    println!("stderr: {}", stderr);

    assert!(!stderr.is_empty() && stderr.contains(expected));

    Ok(())
}
#[test]
fn rox_constant_number() -> TestResult {
    run_test_contains("1", "Number(1.0)")
}

#[test]
fn rox_arithmetic_plus() -> TestResult {
    run_test_contains("1+2", "Number(3.0)")
}

#[test]
fn rox_arithmetic_minus() -> TestResult {
    run_test_contains("1-2", "Number(-1.0)")
}

#[test]
fn rox_arithmetic_multiply() -> TestResult {
    run_test_contains("2*2", "Number(4.0)")
}

#[test]
fn rox_arithmetic_negative() -> TestResult {
    run_test_contains("-2", "Number(-2.0)")
}
#[test]
fn rox_arithmetic_grouping() -> TestResult {
    run_test_contains("(1+2)*3", "Number(9.0)")
}

#[test]
fn rox_arithmetic_complex_grouping() -> TestResult {
    run_test_contains("-((1+2)*2)", "Number(-6.0)")
}

#[test]
fn rox_false() -> TestResult {
    run_test_contains("false", "Bool(false)")
}

#[test]
fn rox_true() -> TestResult {
    run_test_contains("true", "Bool(true)")
}

#[test]
fn rox_falsey_false() -> TestResult {
    run_test_contains("!false", "Bool(true)")
}

#[test]
fn rox_falsey_true() -> TestResult {
    run_test_contains("!true", "Bool(false)")
}

#[test]
fn rox_greater_true() -> TestResult {
    run_test_contains("2 > 1", "Bool(true)")
}

#[test]
fn rox_bang_bang() -> TestResult {
    run_test_contains("1 != 2", "Bool(true)")
}
#[test]
fn rox_greater_false() -> TestResult {
    run_test_contains("1 > 2", "Bool(false)")
}

#[test]
fn rox_greater_equal_true() -> TestResult {
    run_test_contains("2 >= 2", "Bool(true)")
}

#[test]
fn rox_greater_equal_true2() -> TestResult {
    run_test_contains("2 > 1", "Bool(true)")
}

#[test]
fn rox_less_equal_true() -> TestResult {
    run_test_contains("1 <= 2", "Bool(true)")
}

#[test]
fn rox_less_equal_true2() -> TestResult {
    run_test_contains("2 <= 2", "Bool(true)")
}

#[test]
fn rox_less_false() -> TestResult {
    run_test_contains("2 < 1", "Bool(false)")
}

#[test]
fn rox_compare_equal() -> TestResult {
    run_test_contains("(1 == 1) == true", "Bool(true)")
}

#[test]
fn rox_compare_group_equal() -> TestResult {
    run_test_contains("(1 == 1) == (2 == 2)", "Bool(true)")
}

#[test]
fn rox_nil() -> TestResult {
    run_test_contains("nil", "Nil")
}
#[test]
fn rox_nagative_string() -> TestResult {
    fail_test("-a", "unknown type found")
}

#[test]
fn rox_string() -> TestResult {
    run_test_contains("\"a\"", "String(\"a\")")
}

#[test]
fn rox_string_concate() -> TestResult {
    run_test_contains(r#" "a" + "b" "#, r#"String("ab")"#)
}

// #[test]
// fn rox_print() -> TestResult {
//     run_test_contains("print true;", "true")
// }
//
// #[test]
// fn rox_print_boolen() -> TestResult {
//     run_test_contains("print true;", "true")
// }
//
// #[test]
// fn rox_print_number() -> TestResult {
//     run_test_contains("print 1;", "1")
// }
//
// #[test]
// fn rox_print_string() -> TestResult {
//     run_test_contains(r#"print "hello";"#, "hello")
// }
//
// #[test]
// fn rox_print_arithmetic() -> TestResult {
//     run_test_contains("print 1+2*3+(1+1);", "a")
// }

#[test]
fn rox_add_failed() -> TestResult {
    fail_test("1 + true", "operands must be two numbers or two strings")
}

#[test]
fn rox_multiply_failed() -> TestResult {
    fail_test(r#"1 * "a""#, "operands must be two numbers")
}

#[test]
fn rox_subtract_failed() -> TestResult {
    fail_test(r#"1 - "a""#, "operands must be two numbers")
}

#[test]
fn rox_divided_failed() -> TestResult {
    fail_test(r#"1 / "a""#, "operands must be two numbers")
}
//FIXME - this test is failing
#[test]
#[ignore = "This test is failing"]
fn rox_falsey_nil() -> TestResult {
    run_test_contains("!nil", "Bool(true)")
}
