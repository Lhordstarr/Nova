use std::process::Command;

fn bin() -> &'static str {
    env!("CARGO_BIN_EXE_nova")
}

#[test]
fn version_command() {
    let output = Command::new(bin()).arg("version").output().unwrap();
    assert!(output.status.success());
    assert!(String::from_utf8_lossy(&output.stdout).contains("0.1.2"));
}

#[test]
fn help_command() {
    let output = Command::new(bin()).arg("help").output().unwrap();
    assert!(output.status.success());
    assert!(String::from_utf8_lossy(&output.stdout).contains("Usage:"));
}

#[test]
fn run_example_program() {
    let output = Command::new(bin())
        .args(["run", "examples/hello.nova"])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        "Hello, World!\nSharif\n25\n"
    );
}

#[test]
fn check_valid_program() {
    let output = Command::new(bin())
        .args(["check", "examples/hello.nova"])
        .output()
        .unwrap();
    assert!(output.status.success());
}

#[test]
fn run_invalid_syntax_fails() {
    let path = "/tmp/opencode/invalid.nova";
    std::fs::write(path, "print +\n").unwrap();
    let output = Command::new(bin()).args(["run", path]).output().unwrap();
    assert!(!output.status.success());
    assert!(!output.stderr.is_empty());
}

#[test]
fn run_undefined_variable_fails() {
    let path = "/tmp/opencode/undefined.nova";
    std::fs::write(path, "print x + 1\n").unwrap();
    let output = Command::new(bin()).args(["run", path]).output().unwrap();
    assert!(!output.status.success());
}

#[test]
fn run_division_by_zero_fails() {
    let path = "/tmp/opencode/divzero.nova";
    std::fs::write(path, "print 1 / 0\n").unwrap();
    let output = Command::new(bin()).args(["run", path]).output().unwrap();
    assert!(!output.status.success());
}
