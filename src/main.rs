use std::{
  fs, process
};

use clap::Parser;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use sysinfo::Disks;

use crate::{
  config::{cli::Arguments, json::JSONConfig},
  util::normalize_drive::normalize_drive
};

mod config;
mod progress_helpers;
mod run;
mod scanner;
mod task_copy_delete;
mod util;

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

pub const CHANNEL_CAPACITY: usize = 10000;

fn main() {
  let args = Arguments::parse();
  // dbg!(&args);
  
  if args.is_json_config() {
    // JSON config: read and parse
    let config = fs::read_to_string(&args.source).unwrap_or_else(|err| {
      eprintln!("Failed to read config file '{}': {}", args.source.display(), err);
      process::exit(1);
    });
    let mut config = serde_json::from_str::<JSONConfig>(&config).unwrap_or_else(|err| {
      eprintln!("Failed to parse JSON config: {}", err);
      process::exit(1);
    });

    // merge CLI excludes into JSON config
    let merge_sort_dedup = |a: &Vec<String>, b: &Vec<String>| {
      let mut out = a.iter().chain(b.iter()).cloned().collect::<Vec<_>>();
      out.sort();
      out.dedup();
      out
    };
    config.exclude_dirs = merge_sort_dedup(&config.exclude_dirs, &args.exclude_dirs);
    config.exclude_files = merge_sort_dedup(&config.exclude_files, &args.exclude_files);
    config.exclude_patterns = merge_sort_dedup(&config.exclude_patterns, &args.exclude_patterns);
    
    // merge CLI includes into JSON config
    config.include_dirs = merge_sort_dedup(&config.include_dirs, &args.include_dirs);
    config.include_files = merge_sort_dedup(&config.include_files, &args.include_files);
    config.include_patterns = merge_sort_dedup(&config.include_patterns, &args.include_patterns);

    // dbg!(&config);
    // run operations in loop
    let mut i = 0;
    let num_ops = config.operations.len();
    for mut op in config.operations {
      i += 1;
      // merge global excludes with op-specific
      op.exclude_dirs = merge_sort_dedup(&op.exclude_dirs, &config.exclude_dirs);
      op.exclude_files = merge_sort_dedup(&op.exclude_files, &config.exclude_files);
      op.exclude_patterns = merge_sort_dedup(&op.exclude_patterns, &config.exclude_patterns);
      // merge global includes with op-specific
      op.include_dirs = merge_sort_dedup(&op.include_dirs, &config.include_dirs);
      op.include_files = merge_sort_dedup(&op.include_files, &config.include_files);
      op.include_patterns = merge_sort_dedup(&op.include_patterns, &config.include_patterns);
      
      if args.no_delete { op.no_delete = true }
      if config.log_files { op.log_files = true }
      if args.log_files { op.log_files = true }
      
      println!();
      //dbg!(&op);
      run::run(op, format!(" {} / {} ", i, num_ops));
    }
    println!();
    println!("Completed {} operations.", num_ops);

    if config.drive_info.len() > 0 {
      // normalize drive paths
      config.drive_info = config.drive_info.iter().map(|d| normalize_drive(d.to_string())).collect();
      println!();
      println!("Drive info:");
      let longest_drive: usize = config.drive_info.iter().fold(0, |sum, cur| {
        if cur.len() > sum { cur.len() } else { sum }
      });
      let infos = MultiProgress::new();
      for disk in Disks::new_with_refreshed_list().iter() {
        // check if drive mount point is in provided list of mount points to print
        if let Some(mount_str) = disk.mount_point().to_str() {
          if !config.drive_info.contains(&mount_str.to_string()) {
            continue;
          }
        }
        // prepare progress bar
        let info = infos.add(ProgressBar::new(disk.total_space()));
        info.set_style(
          ProgressStyle::with_template("{msg}   {wide_bar}   {bytes:>10} / {total_bytes:>10}   {percent:>3} % ").unwrap()
          .progress_chars("▆▆▁")
        );
        info.set_message(format!("{:<width$}", disk.mount_point().display(), width = longest_drive));
        info.set_position(disk.total_space() - disk.available_space());
        info.abandon();
      }
    }

    println!();
  } else {
    // Not in JSON-config-mode, just run on arguments
    println!();
    run::run(args, String::from(""));
    println!();
  }
}