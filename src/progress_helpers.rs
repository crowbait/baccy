use std::time::Duration;

use indicatif::{ProgressBar, ProgressStyle};

pub const PROGERSS_BAR_TASK: &str = "##-";
pub const PROGERSS_BAR_FILE: &str = "=> ";
pub const PROGRESS_SPINNER_TICKRATE: Duration = Duration::from_millis(150);

pub fn spinner_style() -> ProgressStyle {
  ProgressStyle::with_template("{spinner:.blue} {msg}")
  .unwrap()
  .tick_strings(&[
    "▸▹▹",
    "▹▸▹",
    "▹▹▸",
    "▹▹▹",
    "▹▹▹",
    "▪▪▪",
  ])
}
/// Sets defaults on a passed spinner, including template, steady tick and the given message.
pub fn setup_spinner(progress: &mut ProgressBar, msg: &'static str) {
  progress.set_style(spinner_style());
  progress.enable_steady_tick(Duration::from_millis(150));
  progress.set_message(msg);
}

/// Replaces a progress bar with a "finished" spinner and sets a message.
/// Explicitely consumes the progress bar; it must not be used after it has been finished.
pub fn finish_progress(progress: ProgressBar, msg: String) {
  progress.set_style(spinner_style());
  progress.finish_with_message(msg);
}