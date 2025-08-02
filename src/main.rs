use std::{
  fs, process
};

use clap::Parser;

use crate::{
  args_cli::Arguments,
  args_json::JSONConfig
};

mod args_cli;
mod args_json;
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
      
      if args.no_delete { op.no_delete = true }
      if config.log_files { op.log_files = true }
      if args.log_files { op.log_files = true }
      

      println!();
      //dbg!(&op);
      run::run(op, format!(" {} / {} ", i, num_ops));
    }
    println!();
    println!("Completed {} operations.", num_ops);
    println!();
  } else {
    // Not in JSON-config-mode, just run on arguments
    println!();
    run::run(args, String::from(""));
    println!();
  }
}