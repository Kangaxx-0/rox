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
    run_test_contains("print 1;", "1")
}

#[test]
fn rox_arithmetic_plus() -> TestResult {
    run_test_contains("print 1+2;", "3")
}

#[test]
fn rox_arithmetic_minus() -> TestResult {
    run_test_contains("print 1-2;", "-1")
}

#[test]
fn rox_arithmetic_multiply() -> TestResult {
    run_test_contains("print 2*2;", "4")
}

#[test]
fn rox_arithmetic_negative() -> TestResult {
    run_test_contains("print -2;", "-2")
}
#[test]
fn rox_arithmetic_grouping() -> TestResult {
    run_test_contains("print (1+2)*3;", "9")
}

#[test]
fn rox_arithmetic_complex_grouping() -> TestResult {
    run_test_contains("print -((1+2)*2);", "-6")
}

#[test]
fn rox_false() -> TestResult {
    run_test_contains("print false;", "false")
}

#[test]
fn rox_true() -> TestResult {
    run_test_contains("print true;", "true")
}

#[test]
fn rox_falsey_false() -> TestResult {
    run_test_contains("print !false;", "true")
}

#[test]
fn rox_falsey_true() -> TestResult {
    run_test_contains("print !true;", "false")
}

#[test]
fn rox_greater_true() -> TestResult {
    run_test_contains("print 2 > 1;", "true")
}

#[test]
fn rox_bang_bang() -> TestResult {
    run_test_contains("print 1 != 2;", "true")
}
#[test]
fn rox_greater_false() -> TestResult {
    run_test_contains("print 1 > 2;", "false")
}

#[test]
fn rox_greater_equal_true() -> TestResult {
    run_test_contains("print 2 >= 2;", "true")
}

#[test]
fn rox_greater_equal_true2() -> TestResult {
    run_test_contains("print 2 > 1;", "true")
}

#[test]
fn rox_less_equal_true() -> TestResult {
    run_test_contains("print 1 <= 2;", "true")
}

#[test]
fn rox_less_equal_true2() -> TestResult {
    run_test_contains("print 2 <= 2;", "true")
}

#[test]
fn rox_less_false() -> TestResult {
    run_test_contains("print 2 < 1;", "false")
}

#[test]
fn rox_compare_equal() -> TestResult {
    run_test_contains("print (1 == 1) == true;", "true")
}

#[test]
fn rox_compare_group_equal() -> TestResult {
    run_test_contains("print (1 == 1) == (2 == 2);", "true")
}

#[test]
fn rox_nil() -> TestResult {
    run_test_contains("print nil;", "Nil")
}
#[test]
fn rox_nagative_string() -> TestResult {
    fail_test("print -a;", "undefined variable 'a'")
}

#[test]
fn rox_string() -> TestResult {
    run_test_contains("print \"a\";", "a")
}

#[test]
fn rox_string_concate() -> TestResult {
    run_test_contains(r#"print "a" + "b";"#, "Printing value of ab")
}

#[test]
fn rox_print() -> TestResult {
    run_test_contains("print true;", "true")
}

#[test]
fn rox_print_number() -> TestResult {
    run_test_contains("print 1;", "1")
}

#[test]
fn rox_print_string() -> TestResult {
    run_test_contains(r#"print "hello";"#, "hello")
}

#[test]
fn rox_print_arithmetic() -> TestResult {
    run_test_contains("print 1+2*3+(1+1);", "9")
}

#[test]
fn rox_add_failed() -> TestResult {
    fail_test("1 + true;", "operands must be two numbers or two strings")
}

#[test]
fn rox_multiply_failed() -> TestResult {
    fail_test(r#"1 * "a";"#, "operands must be two numbers")
}

#[test]
fn rox_subtract_failed() -> TestResult {
    fail_test(r#"1 - "a";"#, "operands must be two numbers")
}

#[test]
fn rox_divided_failed() -> TestResult {
    fail_test(r#"1 / "a";"#, "operands must be two numbers")
}
#[test]
fn rox_falsey_nil() -> TestResult {
    run_test_contains("print !nil;", "true")
}

#[test]
fn rox_falsey_nil2() -> TestResult {
    run_test_contains("print nil == nil;", "true")
}

#[test]
fn rox_falsey_nil3() -> TestResult {
    run_test_contains("print nil != nil;", "false")
}

#[test]
fn rox_variable() -> TestResult {
    run_test_contains("var a = 1;", "1")
}

#[test]
fn rox_variable2() -> TestResult {
    run_test_contains(
        r#"var a = 1 + 1; 
        print a;"#,
        "Printing value of 2",
    )
}

#[test]
fn rox_variable3() -> TestResult {
    run_test_contains(
        r#"
            var a = 1 + 1; 
            var b = a + 1; 
            print b;"#,
        "Printing value of 3",
    )
}

#[test]
fn rox_variable_use_twice() -> TestResult {
    run_test_contains(
        r#"
            var a = 1;
            var b = a+1;
            var c = a+2;
            print c;"#,
        "Printing value of 3",
    )
}

#[test]
fn rox_variable_assign() -> TestResult {
    run_test_contains(
        r#"
            var a = 1 + 1; 
            var a = 3;
            print a;"#,
        "Printing value of 3",
    )
}

#[test]
fn rox_variable_assign2() -> TestResult {
    run_test_contains(
        r#"
            var a = 1;
            var b = 2;
            var c = 3;
            var d = a + c;
            print d;"#,
        "Printing value of 4",
    )
}

#[test]
fn rox_variable_assign_after_allocation() -> TestResult {
    run_test_contains(
        r#"
            var a = 1;
            var b = 2;
            var c = 3;
            var d = 4;
            var e = 5;
            var f = 6;
            var g = 7;
            var h = 8;
            var i = 9;
            var j = 10;
            var k = 11;
            var l = 12;
            var m = a+k+f;
            print m;"#,
        "Printing value of 18",
    )
}

#[test]
fn rox_variable_assign_after_allocation2() -> TestResult {
    run_test_contains(
        r#"
            var a = 1;
            var b = 2;
            var c = 3;
            var d = 4;
            var e = 5;
            var f = 6;
            var g = 7;
            var h = 8;
            var i = 9;
            var j = 10;
            var k = 11;
            var l = 12;
            var m = a+k+f;
            var n = m + 1;
            var o = 13;
            var p = 14;
            var q = 15;
            var r = 16;
            var s = 17;
            var z = c + g + m +q;
            print z;"#,
        "Printing value of 43",
    )
}

#[test]
fn rox_variable_assign_after_allocation3() -> TestResult {
    run_test_contains(
        r#"
            var a = 1;
            var b = 2;
            var c = 3;
            var d = 4;
            var e = 5;
            var f = 6;
            var g = 7;
            var h = 8;
            var i = 9;
            var j = 10;
            var k = 11;
            var l = 12;
            var m = a+k+f;
            var n = m + 1;
            var o = 13;
            var p = 14;
            var q = 15;
            var r = 16;
            var s = 17;
            var z = c + g + m +q;
            var x = z + 1;
            var y = 18;
            var w = 19;
            var v = 20;
            var u = 21;
            var t = 22;
            var ss = 23;
            var zz = 24;
            var xx = 25;
            var yy = 26;
            var ww = 27;
            var vv = 28;
            var uu = 29;
            var tt = 30;
            var sss = 31;
            var zzz = 32;
            var xxx = 33;
            var yyy = 34;
            var www = 35;
            var vvv = 36;
            var uuu = 37;
            var ttt = 38;
            var ssss = 39;
            var zzzz = 40;
            var xxxx = 41;
            var yyyy = 42;
            var wwww = 43;
            var vvvv = 44;
            var uuuu = 45;
            var tttt = 46;
            var sssss = 47;
            var zzzzz = 48;
            var xxxxx = 49;
            var yyyyy = 50;
            var wwwww = 51;
            var vvvvv = 52;
            var uuuuu = 53;
            var ttttt = 54;
            var ssssss = 55;
            var zzzzzz = 56;
            var xxxxxx = 57;
            var yyyyyy = a + xxxxxx;
            print yyyyyy;"#,
        "Printing value of 58",
    )
}

#[test]
fn rox_local_variable() -> TestResult {
    run_test_contains(
        r#"
            var a = 1;
            {
                var a = 2;
                print a;
            }
        "#,
        "Printing value of 2",
    )
}

#[test]
fn rox_local_variable2() -> TestResult {
    run_test_contains(
        r#"
            var a = 1;
            {
                var a = 2;
            }
            print a;
        "#,
        "Printing value of 1",
    )
}

#[test]
fn rox_local_variable3() -> TestResult {
    run_test_contains(
        r#"
            var a = 1;
            {
                var b = 2;
                var c = b + a;
                print c;
            }
        "#,
        "Printing value of 3",
    )
}

#[test]
fn rox_local_variable4() -> TestResult {
    run_test_contains(
        r#"
            var a = 1;
            var b = 2;
            var c = a+b;
            {
                var b = 12;
                var c = 13;
                var d = c + b;
            }
            print c;
        "#,
        "Printing value of 3",
    )
}

#[test]
fn rox_local_variable5() -> TestResult {
    run_test_contains(
        r#"
            var a = 1;
            var b = 2;
            var c = a+b;
            {
                var b = 12;
                var c = 13;
                var d = c + b;
                print d;
            }
        "#,
        "Printing value of 25",
    )
}

#[test]
fn rox_local_variable6() -> TestResult {
    run_test_contains(
        r#"
            var a = 1;
            var b = 2;
            var c = a+b;
            {
                var b = 12;
                var d = c + b;
                print d;
            }
        "#,
        "Printing value of 15",
    )
}

#[test]
fn rox_reassign_local() -> TestResult {
    run_test_contains(
        r#"
            var a = 1;
            {
                var a = 2;
                a =3;
                print a;
            }
        "#,
        "Printing value of 3",
    )
}

#[test]
fn rox_reassign_local2() -> TestResult {
    run_test_contains(
        r#"
            var a = 1;
            {
                var a = 2;
                a = 3;
                a = 4;
                print a;
            }
        "#,
        "Printing value of 4",
    )
}

#[test]
fn rox_duplicate_local() -> TestResult {
    fail_test(
        r#"
            var a = 1;
            {
                var a = 2;
                var a = 3;
                print a;
            }
        "#,
        "Variable with this name already declared in this scope",
    )
}

#[test]
fn rox_duplicate_local2() -> TestResult {
    fail_test(
        r#"
            var a = 1;
            var b = 2;
            {
                var a = 2;
                var b = 3;
                var a = 4;
                print a;
            }
        "#,
        "Variable with this name already declared in this scope",
    )
}

#[test]
fn rox_if_then() -> TestResult {
    run_test_contains(
        r#"
            var a = 1;
            if (a == 1) {
                print "a is 1";
            }
        "#,
        "a is 1",
    )
}

#[test]
fn rox_if_false() -> TestResult {
    run_test_contains(
        r#"
            var a = 2;
            if (a == 1) {
                print "a is 1";
            }
            print "done";
        "#,
        "done",
    )
}

#[test]
fn rox_if_else() -> TestResult {
    run_test_contains(
        r#"
            var a = 2;
            if (a == 1) {
                print "a is 1";
            } else {
                print "a is not 1";
            }
        "#,
        "a is not 1",
    )
}

#[test]
fn rox_if_else2() -> TestResult {
    run_test_contains(
        r#"
            var a = 1;
            if (a == 1) {
                print "a is 1";
            } else {
                print "a is not 1";
            }
        "#,
        "a is 1",
    )
}

#[test]
fn rox_and() -> TestResult {
    run_test_contains(
        r#"
            var a = 1;
            var b = 2;
            if (a == 1 and b == 2) {
                print "a is 1 and b is 2";
            }
        "#,
        "a is 1 and b is 2",
    )
}

#[test]
fn rox_and_falsey() -> TestResult {
    run_test_contains(
        r#"
            var a = 1;
            var b = 2;
            if (a == 1 and b == 3) {
                print "a is 1 and b is 2";
            }
            else{
                print "a is 1 and b is not 2";
            }
        "#,
        "a is 1 and b is not 2",
    )
}

#[test]
fn rox_or() -> TestResult {
    run_test_contains(
        r#"
            var a = 1;
            var b = 2;
            if (a == 1 or b == 3) {
                print "a is 1 or b is 3";
            }
        "#,
        "a is 1 or b is 3",
    )
}

#[test]
fn rox_or_2() -> TestResult {
    run_test_contains(
        r#"
            var a = 1;
            var b = 2;
            if (a == 2 or b == 3) {
                print "a is 2 or b is 3";
            }
            else{
                print "a is not 2 and b is not 3";
            }
        "#,
        "a is not 2 and b is not 3",
    )
}

#[test]
fn rox_and_or() -> TestResult {
    run_test_contains(
        r#"
            var a = 1;
            var b = 2;
            if (a == 1 and b == 2 or a == 2 and b == 3) {
                print "a is 1 and b is 2 or a is 2 and b is 3";
            }
        "#,
        "a is 1 and b is 2 or a is 2 and b is 3",
    )
}

#[test]
fn rox_while() -> TestResult {
    run_test_contains(
        r#"
            var a = 1;
            while (a < 5) {
                a = a + 1;
            }
            print a;
        "#,
        "5",
    )
}

#[test]
fn rox_while2() -> TestResult {
    run_test_contains(
        r#"
            var a = 1;
            while (a < 6 and a != 5) {
                a = a + 1;
            }
            print a;
        "#,
        "5",
    )
}

#[test]
fn rox_for() -> TestResult {
    run_test_contains(
        r#"
            var x = 1;
            for (var i = 0; i < 5; i = i + 1) {
                x = x + i;
            }
            print x;
        "#,
        "5",
    )
}

#[test]
fn rox_for2() -> TestResult {
    run_test_contains(
        r#"
            var x = 1;
            for (;x < 5;) {
                x = x + 1;
            }
            print x;
        "#,
        "5",
    )
}

#[test]
fn rox_for3() -> TestResult {
    run_test_contains(
        r#"
            var x = 1;
            for (; x < 5;x = x + 2) {
                print "inside the loop";
            }
            print x;
        "#,
        "5",
    )
}

#[test]
fn rox_for_local_scope() -> TestResult {
    fail_test(
        r#"
            for (var i = 0; i < 5; i = i + 1) {
                print i;
            }
            print i;
        "#,
        "undefined variable 'i'",
    )
}

#[test]
fn rox_func() -> TestResult {
    run_test_contains(
        r#"
            fun foo() {
                print "foo is a function";
            }
            print foo;
        "#,
        "foo",
    )
}

#[test]
fn rox_func_call() -> TestResult {
    run_test_contains(
        r#"
            fun foo() {
                print "foo is a function";
            }
            foo();
        "#,
        "foo is a function",
    )
}

#[test]
fn rox_func_call2() -> TestResult {
    run_test_contains(
        r#"
            fun foo() {
                print "foo is a function";
            }
            fun bar() {
                foo();
            }
            bar();
        "#,
        "foo is a function",
    )
}

#[test]
fn rox_func_call3() -> TestResult {
    run_test_contains(
        r#"
            fun foo() {
                var a=1;
                var b=2;
                print a+b;
            }
            fun bar() {
                foo();
            }
            bar();
        "#,
        "3",
    )
}

#[test]
fn rox_func_call4() -> TestResult {
    run_test_contains(
        r#"
            fun foo(x) {
                var b = 2;
                print x+b;
            }
            foo(2);
        "#,
        "4",
    )
}

#[test]
fn rox_func_call5() -> TestResult {
    run_test_contains(
        r#"
            fun foo(x, y) {
                var b = 2;
                print x+y+b;
            }
            fun bar(x) {
                foo(2, x);
            }
            bar(2);
        "#,
        "6",
    )
}

#[test]
fn rox_func_call_return() -> TestResult {
    run_test_contains(
        r#"
            fun foo(x) {
                return x+1;
            }
            print foo(2);
        "#,
        "3",
    )
}
#[test]
fn rox_func_call_return2() -> TestResult {
    run_test_contains(
        r#"
            fun foo(x, y) {
                var b = 2;
                return x+y+b;
            }
            fun bar(x) {
                return foo(2, x);
            }
            print bar(2);
        "#,
        "6",
    )
}

#[test]
fn rox_func_call_return3() -> TestResult {
    run_test_contains(
        r#"
            fun foo(x, y) {
                var b = 2;
                return x+y+b;
            }
            fun bar(x) {
                return foo(2, x);
            }
            var res1 = bar(2);
            print res1;
        "#,
        "6",
    )
}

#[test]
fn rox_func_call_return4() -> TestResult {
    run_test_contains(
        r#"
            fun foo(x, y) {
                if (x > y) {
                    return x;
                }
                else {
                    return y;
                }
            }
            fun bar(x) {
                return foo(2, x);
            }
            var res = bar(3);
            print res;
        "#,
        "Printing value of 3",
    )
}

#[test]
fn rox_nested_func_call() -> TestResult {
    run_test_contains(
        r#"
            fun foo(){
               fun bar(){
                   print "bar";
               }

               bar ();
            
            }
            foo();
        "#,
        "bar",
    )
}

#[test]
fn rox_func_dup_call() -> TestResult {
    run_test_contains(
        r#"
            fun foo(x) {
                return x;
            }
            var res1 = foo(2);
            var res2 = foo(3);
            print res1 + res2;
        "#,
        "5",
    )
}

#[test]
fn rox_func_dup_call2() -> TestResult {
    run_test_contains(
        r#"
            fun foo(x) {
                return x;
            }
            print foo(2) + foo(3);
        "#,
        "5",
    )
}

#[test]
fn rox_func_dup_call3() -> TestResult {
    run_test_contains(
        r#"
            fun foo(x) {
                return x;
            }
            fun bar(x) {
                return foo(x);
            }
            print bar(2) + bar(3);
        "#,
        "5",
    )
}

#[test]
fn rox_func_dup_call4() -> TestResult {
    run_test_contains(
        r#"
            fun foo(x) {
                return x;
            }
            fun bar(x) {
                return foo(x);
            }
            fun baz(x) {
                return bar(x);
            }
            print baz(2) + baz(3) + bar(4) + foo(5);
        "#,
        "14",
    )
}

#[test]
fn rox_native_func() -> TestResult {
    run_test_contains(
        r#"
            print clock();
        "#,
        "clock",
    )
}

#[test]
fn rox_closure() -> TestResult {
    run_test_contains(
        r#"
            var x = "global";
            fun outer() {
                var x = "outer";
                fun inner() {
                    print x + " here!";
                }
                inner();
            }
            outer();
        "#,
        "outer here!",
    )
}

#[test]
fn rox_nested_closure() -> TestResult {
    run_test_contains(
        r#"
            fun outer() {
                var x = "outer";
                fun middle(){
                    fun inner() {
                        print x;
                    }
                    inner();
                }
                middle();
            }
            outer();
        "#,
        "outer",
    )
}

#[test]
fn rox_closure_with_param() -> TestResult {
    run_test_contains(
        r#"
            fun outer(y) {
                var x = 1;
                fun middle(){
                    fun inner() {
                        print x + y + 1;
                    }
                    inner();
                }
                middle();
            }
            outer(2);
        "#,
        "4",
    )
}
