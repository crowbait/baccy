#![allow(dead_code)]

use std::process::{Command, Stdio};
#[cfg(target_family = "windows")]
const SHELL: &str = "powershell";
#[cfg(target_family = "windows")]
const FLAG: &str = "";
#[cfg(target_family = "unix")]
const SHELL: &str = "sh";
#[cfg(target_family = "unix")]
const FLAG: &str = "-c";

pub fn run_command<S: AsRef<str>>(cmd: S) -> i32 {
  let status = Command::new(SHELL)
    .args([FLAG, format!("{}", cmd.as_ref()).as_ref()])
    .stdout(Stdio::inherit())
    .stderr(Stdio::inherit())
    .status()
    .expect("Failed to execute command.");

  status.code().unwrap_or(1)
}