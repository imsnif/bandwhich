#[test]
fn binary_help_still_starts() {
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_bandwhich"))
        .arg("--help")
        .output()
        .unwrap();

    assert!(output.status.success());
    assert!(String::from_utf8_lossy(&output.stdout).contains("bandwhich"));
}
