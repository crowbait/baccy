use std::{fs, path::PathBuf, sync::{atomic::{AtomicUsize, Ordering}, Arc}};

use colored::Colorize;
use crossbeam::channel::Sender;
use indicatif::ProgressBar;
use walkdir::WalkDir;

use crate::{progress_helpers::{spinner_style, PROGRESS_SPINNER_TICKRATE}, task_copy, Task};

pub fn scanner(
  src: PathBuf,
  dst: PathBuf,
  tx: Sender<Task>,
  num_positive: Arc<AtomicUsize>,
  num_delete: Arc<AtomicUsize>,
  progress: ProgressBar
) {
  let mut scanned_total: u64 = 0;
  for entry in WalkDir::new(&src).into_iter().filter_map(Result::ok) {
    let relative_path = entry.path().strip_prefix(&src).unwrap();
    let path_in_dst = dst.join(relative_path);

    // if entry is directory, create (with parents) in destination and skip rest of iteration
    if entry.file_type().is_dir() {
      fs::create_dir_all(&path_in_dst).ok();
      continue;
    }

    // check whether modification time on source is newer -> copy needed
    let needs_copy = match fs::metadata(&path_in_dst) {
      Ok(metadata) => {
        let src_mtime = entry.metadata().unwrap().modified().unwrap();
        let dst_mtime = metadata.modified().unwrap();
        src_mtime > dst_mtime
      }
      Err(_) => true // file missing in destination, copy
    };

    if needs_copy {
      // increment positive match count (for worker progress) and send task
      num_positive.fetch_add(1, Ordering::SeqCst);
      tx.send(Task::Copy(task_copy::Copy::new(entry.path().to_path_buf(), path_in_dst))).unwrap();
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
        tx.send(Task::Delete(entry.path().to_path_buf())).unwrap();
      }
    }
  }

  let num_pos = num_positive.load(Ordering::SeqCst) as u64;
  progress.finish_with_message(format!(
    "Scanned {} files: {} skipped, {} to copy, {} marked for deletion.",
    scanned_total.to_string().cyan(),
    (scanned_total - num_pos).to_string().cyan(),
    num_pos.to_string().cyan(),
    num_delete.load(Ordering::SeqCst).to_string().cyan()
  ));
}