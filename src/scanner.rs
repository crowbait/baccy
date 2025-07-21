use std::{collections::HashSet, fs, path::{Component, Path, PathBuf}, sync::{atomic::{AtomicUsize, Ordering}, Arc}};

use colored::Colorize;
use crossbeam::channel::Sender;
use glob::Pattern;
use indicatif::ProgressBar;
use walkdir::WalkDir;

use crate::{options::Exclude, progress_helpers::{spinner_style, PROGRESS_SPINNER_TICKRATE}, task_copy, Task};

pub fn scanner(
  src: PathBuf,
  dst: PathBuf,
  tx: Sender<Task>,
  num_positive: Arc<AtomicUsize>,
  num_delete: Arc<AtomicUsize>,
  progress: ProgressBar,
  exclude: Vec<Exclude>
) {
  let mut scanned_total: u64 = 0;

  // prepare exclusion lists (as set, for easy ".contains()")
  let excluded_dir_names: HashSet<&str> = exclude.iter()
    .filter_map(|ex| match ex {
      Exclude::DirName(name) => Some(name.as_str()),
      _ => None
    })
    .collect();
  let excluded_file_names: HashSet<&str> = exclude.iter()
    .filter_map(|ex| match ex {
      Exclude::FileName(name) => Some(name.as_str()),
      _ => None
    })
    .collect();
  let excluded_patterns: HashSet<Pattern> = exclude.iter()
    .filter_map(|ex| match ex {
      Exclude::Pattern(pattern) => Pattern::new(&pattern).ok(),
      _ => None
    })
    .collect();

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
          excluded_dir_names.contains(os.to_string_lossy().as_ref()),
        _ => false
      })
      || // file name - exact
      entry.file_type().is_file() && 
      entry.file_name()
        .to_str()
        .map(|s| excluded_file_names.contains(s))
        .unwrap_or(false)
      || // pattern match
      excluded_patterns.iter().any(|pattern| {
        pattern.matches_path(relative_path)
      });

    // if is directory: perform checks and create, if appropriate
    if entry.file_type().is_dir() && !excluded {
      fs::create_dir_all(&path_in_dst).ok();
      continue;
    }

    let needs_copy = 
      if excluded {
        false
      } else {
        match fs::metadata(&path_in_dst) {
          Ok(metadata) => {
            let src_mtime = entry.metadata().unwrap().modified().unwrap();
            let dst_mtime = metadata.modified().unwrap();
            src_mtime > dst_mtime
          }
          Err(_) => true // file missing in destination, copy
        }
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