use std::{
  collections::{HashSet, VecDeque}, fs, path::PathBuf, sync::{
    atomic::{AtomicU64, AtomicUsize, Ordering},
    Arc,
  }, thread, time::{Duration, Instant}
};

use colored::Colorize;
use crossbeam::channel::bounded;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use walkdir::WalkDir;

use crate::{args_cli::Arguments, progress_helpers::{
  finish_progress, setup_spinner, PROGERSS_BAR_TASK
}, scanner, util::bytes_to_str, Task, CHANNEL_CAPACITY};

pub fn run(args: Arguments, step_prefix: String) {
  let target = if args.target.is_some() {
    args.target.unwrap()
  } else {
    panic!("Target path cannot be None on execution.");
  };

  println!("{}", format!(
    "{}    Sync: {} → {}",
    step_prefix.on_cyan(),
    args.source.to_str().unwrap().cyan(),
    target.to_str().unwrap().cyan()
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

  // Bounded channel (inter-thread communication): blocks on send() until there is room for the message
  let (tx, rx) = bounded::<Task>(CHANNEL_CAPACITY);
  
  // Atomic value lets multiple threads read/write the same data safely
  let num_scanned_positive = Arc::new(AtomicUsize::new(0));
  // Keeps track of how many files have to be deleted
  let num_scanned_delete = Arc::new(AtomicUsize::new(0));
  // This value keeps track of how many files actually need to be copied; for worker progress bar
  let bytes_to_copy_total = Arc::new(AtomicU64::new(0));

  // Prepare progress
  let progress = MultiProgress::new();
  let scan_progress = progress.add(ProgressBar::new(total_files as u64));
  scan_progress.set_style(
    ProgressStyle::with_template("Scanned:      {wide_bar} {pos:>10} / {len:>10}   {msg}").unwrap()
    .progress_chars(PROGERSS_BAR_TASK)
  );
  scan_progress.enable_steady_tick(Duration::from_millis(100));

  let mut work_progress = progress.add(ProgressBar::new(1));
  work_progress.set_style(
    ProgressStyle::with_template("{msg} {wide_bar} {bytes:>10} / {total_bytes:>10}   ETA: {eta:<10}").unwrap()
    .progress_chars(PROGERSS_BAR_TASK)
  );
  work_progress.set_message("Bytes copied:");
  work_progress.enable_steady_tick(Duration::from_millis(100));
  let mut filename_progress = progress.add(ProgressBar::new_spinner());
  setup_spinner(&mut filename_progress, "");
  
  // Scanner thread: processes file metadata and creates tasks for worker thread
  let src_clone = args.source.clone();
  let dst_clone = target.clone();
  let num_positive_clone = num_scanned_positive.clone();
  let num_delete_clone = num_scanned_delete.clone();
  let num_bytes_clone = bytes_to_copy_total.clone();
  thread::spawn(move || scanner::scanner(
    src_clone,
    dst_clone,
    tx,
    num_positive_clone,
    num_delete_clone,
    num_bytes_clone,
    &scan_progress,
    args.exclude_dirs,
    args.exclude_files,
    args.exclude_patterns,
    args.no_delete
  ));

  let mut is_delete_step = false; // deletes ALWAYS get processed after copies, making this safe
  let mut deleted_count = 0;

  // Prepare buffer for file name logging, if enabled
  let mut filename_buffer: VecDeque<Task> = VecDeque::with_capacity(20);
  let mut last_filename_log = Instant::now();
  let filename_log_interval = Duration::from_millis(500);

  // Flushes the buffer of `Task`s that need to be logged.
  // Returns "now" which should be assigned to `last_filename_log`.
  let log_files = |buffer: &mut VecDeque<Task>| {
    for file in buffer.drain(..) {
      let _ = progress.println(format!(
        "{:>10}: {}",
        match &file {
          Task::Copy(task) => bytes_to_str(task.bytes).dimmed().bold(),
          Task::Delete(_) => "DEL".dimmed().bold()
        },
        file.relative().dimmed()
      ));
    }
    Instant::now()
  };

  for task in rx {
    filename_progress.set_message(format!(
      "{}",
      task.relative().dimmed()
    ));

    if args.log_files && (
      filename_buffer.len() >= 20 || 
      last_filename_log.elapsed() >= filename_log_interval
    ) { last_filename_log = log_files(&mut filename_buffer); }

    match task {
      Task::Copy(task) => {
        work_progress.set_length(bytes_to_copy_total.load(Ordering::SeqCst));
        let result = if task.bytes > (1024*1024*50) {
          task.execute_with_progress(&progress, &work_progress)
        } else {
          let res = task.execute();
          work_progress.inc(task.bytes as u64);
          res
        };
        if result.is_err() {
          let _ = progress.println(format!("{}", format!(
            "Copy failed: {} -> {}",
            task.from.display(),
            task.to.display()
          ).bright_red()));
        }
        if args.log_files {
          filename_buffer.push_back(Task::Copy(task));
        }
      }
      Task::Delete(task) => {
        if !is_delete_step {
          is_delete_step = true;
          finish_progress(work_progress, format!(
            "Copied {} files, {}.",
            num_scanned_positive.load(Ordering::SeqCst).to_string().cyan(),
            bytes_to_str(bytes_to_copy_total.load(Ordering::SeqCst)).cyan()
          ));
          work_progress = progress.add(ProgressBar::new_spinner());
          setup_spinner(&mut work_progress, "Deleting files...");
        }

        let _ = fs::remove_file(&task.path);

        deleted_count += 1;
        let deleted_count_colored = deleted_count.to_string().cyan();
        work_progress.set_message(format!("Deleted {} files", deleted_count_colored));

        if args.log_files {
          filename_buffer.push_back(Task::Delete(task));
        }
      }
    }
  }

  // Flush logs finally
  log_files(&mut filename_buffer);

  if is_delete_step {
    work_progress.finish_with_message(format!(
      "Deleted {} files.",
      deleted_count.to_string().cyan()
    ));
  } else {
    finish_progress(work_progress, format!(
      "Copied {} files, {}.",
      num_scanned_positive.load(Ordering::SeqCst).to_string().cyan(),
      bytes_to_str(bytes_to_copy_total.load(Ordering::SeqCst)).cyan()
    ));
  }
  filename_progress.finish_and_clear();
  progress.remove(&filename_progress);

  work_progress = progress.add(ProgressBar::new_spinner());
  setup_spinner(&mut work_progress, "Finding directories to delete...");

  // Find all directories (and their relative paths) in source
  let source_dirs: HashSet<PathBuf> = WalkDir::new(&args.source)
    .into_iter()
    .filter_map(Result::ok)
    .filter(|e| e.file_type().is_dir())
    .map(|e| e.path().strip_prefix(&args.source).unwrap().to_path_buf())
    .collect();
  // Find directories in destination that have no relative-path-equivalent in source
  let mut dst_dirs: Vec<PathBuf> = WalkDir::new(&target)
    .into_iter()
    .filter_map(Result::ok)
    .filter(|e| e.file_type().is_dir())
    .map(|e| e.path().strip_prefix(&target).unwrap().to_path_buf())
    .filter(|rel| !source_dirs.contains(rel))
    .map(|rel| target.join(rel))
    .collect();

  // Sort to be bottom-up, to prevent "can't delete non-empty dir"
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