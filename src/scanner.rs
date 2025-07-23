use std::{fs, os::windows::fs::MetadataExt, path::{Component, Path, PathBuf}, sync::{atomic::{AtomicU64, AtomicUsize, Ordering}, Arc}};

use colored::Colorize;
use crossbeam::channel::Sender;
use glob::Pattern;
use indicatif::ProgressBar;
use walkdir::WalkDir;

use crate::{progress_helpers::{spinner_style, PROGRESS_SPINNER_TICKRATE}, task_copy_delete, Task, CHANNEL_CAPACITY};

pub fn scanner(
  src: PathBuf,
  dst: PathBuf,
  tx: Sender<Task>,
  num_positive: Arc<AtomicUsize>,
  num_delete: Arc<AtomicUsize>,
  bytes_to_copy: Arc<AtomicU64>,
  progress: &ProgressBar,
  exclude_dirs: Vec<String>,
  exclude_files: Vec<String>,
  exclude_patterns: Vec<Pattern>,
) {
  let mut scanned_total: u64 = 0;

  for entry in WalkDir::new(&src).into_iter().filter_map(Result::ok) {
    let relative_path = entry.path().strip_prefix(&src).unwrap();
    let path_in_dst = dst.join(relative_path);

    // prepares an ancestry path, excluding the file name (if file, not dir), for dirname exclusion
    let dirs_path = if entry.file_type().is_dir() {
      relative_path
    } else {
      relative_path.parent().unwrap_or_else(|| Path::new(""))
    };
    
    // check exclusions
    let excluded: bool = 
      // dir name - exact
      dirs_path.components().any(|c| match c {
        Component::Normal(os) => 
          exclude_dirs.iter().any(|ex| ex == &os.to_string_lossy()),
        _ => false
      })
      || // file name - exact
      entry.file_type().is_file() && 
      entry.file_name()
        .to_str()
        .map(|s| exclude_files.iter().any(|ex| ex == s))
        .unwrap_or(false)
      || // pattern match
      exclude_patterns.iter().any(|pattern| pattern.matches_path(relative_path));

    // if is directory: perform checks and create, if appropriate
    if entry.file_type().is_dir() && !excluded {
      fs::create_dir_all(&path_in_dst).ok();
      continue;
    }

    let src_metadata = entry.metadata().unwrap();
    let bytes = src_metadata.file_size();

    let needs_copy = 
      if excluded {
        false
      } else {
        match fs::metadata(&path_in_dst) {
          Ok(metadata) => {
            let src_mtime = src_metadata.modified().unwrap();
            let dst_mtime = metadata.modified().unwrap();
            src_mtime > dst_mtime
          }
          Err(_) => true // file missing in destination, copy
        }
      };

    if needs_copy {
      // increment positive match count (for worker progress) and send task
      num_positive.fetch_add(1, Ordering::SeqCst);
      bytes_to_copy.fetch_add(bytes, Ordering::SeqCst);
      tx.send(Task::Copy(task_copy_delete::Copy::new(
        entry.path().to_path_buf(),
        path_in_dst,
        relative_path.display().to_string(),
        bytes
      ))).unwrap();
      progress.set_message(format!("{:<15}", format!(
        "Buffer: {:.2}%",
        (tx.len() as f64 / CHANNEL_CAPACITY as f64) * 100.0
      ).dimmed()));
    }

    progress.inc(1);
    scanned_total += 1;
  }

  // replace progress bar with spinner
  progress.set_style(spinner_style());
  progress.enable_steady_tick(PROGRESS_SPINNER_TICKRATE);
  progress.set_message("Finding files to delete...");

  // find files to delete
  for entry in WalkDir::new(&dst).into_iter().filter_map(Result::ok) {
    if entry.file_type().is_file() {
      let relative_path = entry.path().strip_prefix(&dst).unwrap();
      let path_in_src = src.join(relative_path);
      if !path_in_src.exists() {
        num_delete.fetch_add(1, Ordering::SeqCst);
        tx.send(Task::Delete(task_copy_delete::Delete::new(
          entry.path().to_path_buf(),
          relative_path.display().to_string()
        ))).unwrap();
      }
    }
  }

  let num_pos = num_positive.load(Ordering::SeqCst) as u64;
  progress.disable_steady_tick();
  progress.finish_with_message(format!(
    "Scanned {} files: {} skipped, {} to copy, {} marked for deletion.",
    scanned_total.to_string().cyan(),
    (scanned_total - num_pos).to_string().cyan(),
    num_pos.to_string().cyan(),
    num_delete.load(Ordering::SeqCst).to_string().cyan()
  ));
}