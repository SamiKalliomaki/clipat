use std::{
    path::Path, process::{self, Stdio}
};

use assert_cmd::Command;
use once_cell::sync::Lazy;

static BIN: Lazy<&Path> = Lazy::new(|| assert_cmd::cargo::cargo_bin!("clipat"));

fn get_test_port() -> u16 {
    15000 + rand::random::<u16>() % 10000
}

fn get_test_addr() -> String {
    format!("127.0.0.1:{}", get_test_port())
}

fn run_server(addr: &str) -> anyhow::Result<process::Child> {
    let mut cmd = process::Command::new(BIN.as_os_str());
    cmd.env("FAKE_CLIPBOARD", "1");
    cmd.arg("server").arg("--listen").arg(addr);
    cmd.stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    Ok(cmd.spawn()?)
}

fn client(addr: &str, subcmd: &str) -> Command {
    let mut cmd = Command::new(BIN.as_os_str());
    cmd.arg(subcmd).arg("--target").arg(addr);
    cmd
}

#[test]
fn test_copy_paste_text() -> anyhow::Result<()> {
    let addr = get_test_addr();
    let mut server = run_server(&addr)?;

    client(&addr, "copy")
        .write_stdin("Hello, world!\nLorem ipsum dolor sit amet.")
        .assert()
        .success();
    client(&addr, "paste")
        .assert()
        .success()
        .stdout("Hello, world!\nLorem ipsum dolor sit amet.");

    client(&addr, "copy")
        .write_stdin("Testing 1\n2\n3")
        .assert()
        .success();
    client(&addr, "paste")
        .assert()
        .success()
        .stdout("Testing 1\n2\n3");

    server.kill()?;
    Ok(())
}
