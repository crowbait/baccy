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

/// This function passes its parameter as a command to the system shell.
/// The invoked shell uses the stdout and stderr from this program.
/// Returns the exit code of the executed command.
pub fn run_command<S: AsRef<str>>(cmd: S) -> i32 {
  let status = Command::new(SHELL)
    .args([FLAG, format!("{}", cmd.as_ref()).as_ref()])
    .stdout(Stdio::inherit())
    .stderr(Stdio::inherit())
    .status()
    .expect("Failed to execute command.");

  status.code().unwrap_or(1)
}