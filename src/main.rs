use std::{
  collections::HashSet, fs, path::PathBuf, sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
  }, thread
};

use clap::Parser;
use colored::Colorize;
use crossbeam::channel::bounded;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use walkdir::WalkDir;

use crate::{args_cli::Arguments, progress_helpers::{finish_progress, setup_spinner, PROGERSS_BAR_TASK}, scanner::scanner};

mod args_util;
mod args_cli;
mod progress_helpers;
mod scanner;
mod task_copy_delete;

// Represents a copy or delete task
enum Task {
  Copy(task_copy_delete::Copy),
  Delete(task_copy_delete::Delete),
}
impl Task {
  fn relative(&self) -> &String {
    match self {
      Task::Copy(c) => &c.relative,
      Task::Delete(d) => &d.relative,
    }
  }
}

pub const CHANNEL_CAPACITY: usize = 1000;

fn main() {
  let args = Arguments::parse();
  // dbg!(&args);
  
  println!();
  println!("{}", format!(
    "Sync: {} → {}",
    args.source.to_str().unwrap().cyan(),
    args.target.to_str().unwrap().cyan()
  ).bold());
  
  // Count total files - progress spinner
  let mut progress = ProgressBar::new_spinner();
  setup_spinner(&mut progress, "Counting files...");

  // Count total files
  let total_files = WalkDir::new(&args.source)
    .into_iter()
    .filter_map(Result::ok)
    .filter(|e| e.file_type().is_file())
    .count();
  progress.finish_with_message(format!("Found {total_files} files."));

  // Bounded channel (inter-thread communicaiton): blocks on send() until there is room for the message
  let (tx, rx) = bounded::<Task>(CHANNEL_CAPACITY);
  
  // Atomic value lets multiple threads read/write the same data safely
  // This value keeps track of how many files actually need to be copied; for worker progress bar
  let num_scanned_positive = Arc::new(AtomicUsize::new(0));
  // Keeps track of how many files have to be deleted
  let num_scanned_delete = Arc::new(AtomicUsize::new(0));

  // Prepare progress
  let progress = MultiProgress::new();
  let scan_progress = progress.add(ProgressBar::new(total_files as u64));
  scan_progress.set_style(
    ProgressStyle::with_template("Scanned:      {wide_bar} {pos:>6} / {len:>6}   {msg}").unwrap()
    .progress_chars(PROGERSS_BAR_TASK)
  );
  let mut work_progress = progress.add(ProgressBar::new(total_files as u64));
  work_progress.set_style(
    ProgressStyle::with_template("{msg} {wide_bar} {pos:>6} / {len:>6}   ETA: {eta:<10}").unwrap()
    .progress_chars(PROGERSS_BAR_TASK)
  );
  work_progress.set_message("Files copied:");
  let mut filename_progress = progress.add(ProgressBar::new_spinner());
  setup_spinner(&mut filename_progress, "");
  
  // Scanner thread: processes file metadata and creates tasks for worker thread
  let src_clone = args.source.clone();
  let dst_clone = args.target.clone();
  let num_positive_clone = num_scanned_positive.clone();
  let num_delete_clone = num_scanned_delete.clone();
  thread::spawn(move || scanner(
    src_clone,
    dst_clone,
    tx,
    num_positive_clone,
    num_delete_clone,
    &scan_progress,
    args.exclude_dirs,
    args.exclude_files,
    args.exclude_patterns
  ));

  let mut is_delete_step = false; // deletes ALWAYS get processed after copies, making this safe
  let mut deleted_count = 0;
  for task in rx {
    filename_progress.set_message(format!(
      "{}",
      task.relative().dimmed()
    ));
    match task {
      Task::Copy(task) => {
        work_progress.set_length(num_scanned_positive.load(Ordering::SeqCst) as u64);
        let result = if task.get_filesize() > (1024*1024*50) {
          task.execute_with_progress(&progress)
        } else {
          task.execute()
        };
        if result.is_err() {
          let _ = progress.println(format!(
            "Copy failed: {} -> {}",
            task.from.display(),
            task.to.display()
          ));
        }
        work_progress.inc(1);
      }
      Task::Delete(task) => {
        if !is_delete_step {
          is_delete_step = true;
          finish_progress(work_progress, format!(
            "Copied {} files.",
            num_scanned_positive.load(Ordering::SeqCst).to_string().cyan()
          ));
          work_progress = progress.add(ProgressBar::new_spinner());
          setup_spinner(&mut work_progress, "Deleting files...");
        }

        let _ = fs::remove_file(task.path);

        deleted_count += 1;
        let deleted_count_colored = deleted_count.to_string().cyan();
        work_progress.set_message(format!("Deleted {} files", deleted_count_colored));
      }
    }
  }

  if is_delete_step {
    work_progress.finish_with_message(format!(
      "Deleted {} files.",
      deleted_count.to_string().cyan()
    ));
  } else {
    finish_progress(work_progress, format!(
      "Copied {} files.",
      num_scanned_positive.load(Ordering::SeqCst).to_string().cyan()
    ));
  }
  filename_progress.finish_and_clear();
  progress.remove(&filename_progress);

  work_progress = progress.add(ProgressBar::new_spinner());
  setup_spinner(&mut work_progress, "Finding directories to delete...");

  // find all directories (and their relative paths) in source
  let source_dirs: HashSet<PathBuf> = WalkDir::new(&args.source)
    .into_iter()
    .filter_map(Result::ok)
    .filter(|e| e.file_type().is_dir())
    .map(|e| e.path().strip_prefix(&args.source).unwrap().to_path_buf())
    .collect();
  // find directories in destination that have no relative-path-equivalent in source
  let mut dst_dirs: Vec<PathBuf> = WalkDir::new(&args.target)
    .into_iter()
    .filter_map(Result::ok)
    .filter(|e| e.file_type().is_dir())
    .map(|e| e.path().strip_prefix(&args.target).unwrap().to_path_buf())
    .filter(|rel| !source_dirs.contains(rel))
    .map(|rel| args.target.join(rel))
    .collect();

  // sort to be bottom-up, to prevent "can't delete non-empty dir"
  dst_dirs.sort_by_key(|p| std::cmp::Reverse(p.components().count()));
  let dst_dirs_count = dst_dirs.len();

  for dir in dst_dirs {
    let _ = fs::remove_dir(&dir);
  }
  if dst_dirs_count > 0 {
    work_progress.finish_with_message(format!(
      "Deleted {} directories in destination not present in source.",
      dst_dirs_count.to_string().cyan()
    ));
  } else {
    work_progress.finish_and_clear();
    progress.remove(&work_progress);
  }
}
