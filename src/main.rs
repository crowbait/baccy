// #![allow(unused_assignments)]

use std::{
  collections::HashSet, fs, path::{Path, PathBuf}, process, sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
  }, thread
};

use colored::Colorize;
use crossbeam::channel::bounded;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use walkdir::WalkDir;

use crate::{options::Exclude, progress_helpers::{finish_progress, setup_spinner, PROGERSS_BAR_TASK}, scanner::scanner};

mod options;
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

fn main() {
  let src = Path::new("D:/cloud.crowbait");
  let dst = Path::new("Y:/backup/cloud.crowbait");
  let exclusions: Vec<Exclude> = vec![
    Exclude::Pattern(String::from(".*")),
    Exclude::DirName(String::from("node_modules"))
  ];

  if !src.exists() {
    eprintln!("{} {} {}", "Source directory ".bright_red().bold(), src.display(), " not found.".bright_red().bold());
    process::exit(1);
  }

  println!();
  println!("{}", format!(
    "Sync: {} → {}",
    src.to_str().unwrap().cyan(),
    dst.to_str().unwrap().cyan()
  ).bold());

  // Count total files - progress spinner
  let mut progress = ProgressBar::new_spinner();
  setup_spinner(&mut progress, "Counting files...");

  // Count total files
  let total_files = WalkDir::new(src)
    .into_iter()
    .filter_map(Result::ok)
    .filter(|e| e.file_type().is_file())
    .count();
  progress.finish_with_message(format!("Found {total_files} files."));

  // Bounded channel (inter-thread communicaiton): blocks on send() until there is room for the message
  let (tx, rx) = bounded::<Task>(10000);
  
  // Atomic value lets multiple threads read/write the same data safely
  // This value keeps track of how many files actually need to be copied; for worker progress bar
  let num_scanned_positive = Arc::new(AtomicUsize::new(0));
  // Keeps track of how many files have to be deleted
  let num_scanned_delete = Arc::new(AtomicUsize::new(0));

  // Prepare progress
  let progress = MultiProgress::new();
  let scan_progress = progress.add(ProgressBar::new(total_files as u64));
  scan_progress.set_style(
    ProgressStyle::with_template("Scanned:      {wide_bar} {pos} / {len}").unwrap()
    .progress_chars(PROGERSS_BAR_TASK)
  );
  let mut work_progress = progress.add(ProgressBar::new(total_files as u64));
  work_progress.set_style(
    ProgressStyle::with_template("{msg} {wide_bar} {pos} / {len} {eta}").unwrap()
    .progress_chars(PROGERSS_BAR_TASK)
  );
  work_progress.set_message("Files copied:");
  let mut filename_progress = progress.add(ProgressBar::new_spinner());
  setup_spinner(&mut filename_progress, "");
  
  // Scanner thread: processes file metadata and creates tasks for worker thread
  let src_clone = src.to_path_buf().clone();
  let dst_clone = dst.to_path_buf().clone();
  let num_positive_clone = num_scanned_positive.clone();
  let num_delete_clone = num_scanned_delete.clone();
  thread::spawn(move || scanner(
    src_clone,
    dst_clone,
    tx,
    num_positive_clone,
    num_delete_clone,
    &scan_progress,
    exclusions
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
  let source_dirs: HashSet<PathBuf> = WalkDir::new(&src)
    .into_iter()
    .filter_map(Result::ok)
    .filter(|e| e.file_type().is_dir())
    .map(|e| e.path().strip_prefix(&src).unwrap().to_path_buf())
    .collect();
  // find directories in destination that have no relative-path-equivalent in source
  let mut dst_dirs: Vec<PathBuf> = WalkDir::new(&dst)
    .into_iter()
    .filter_map(Result::ok)
    .filter(|e| e.file_type().is_dir())
    .map(|e| e.path().strip_prefix(&dst).unwrap().to_path_buf())
    .filter(|rel| !source_dirs.contains(rel))
    .map(|rel| dst.join(rel))
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
