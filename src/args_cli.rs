use clap::{Parser, ValueHint};
use glob::Pattern;
use std::path::PathBuf;

use crate::args_util::{args_parse_path, args_parse_pattern, args_validate_existing_path};

#[derive(Debug, Parser)]
#[command(name = "Backrust", version, about = "Efficient and informative directory sync")]
pub struct Arguments {
  /// Directory which will be synced.
  #[arg(value_parser = args_validate_existing_path, value_hint = ValueHint::DirPath)]
  pub source: PathBuf,

  /// Directory into which to sync.
  /// This will NOT create a directory named (source), the path given here will be interpreted
  /// as the copy of (source) itself.
  #[arg(value_parser = args_parse_path, value_hint = ValueHint::DirPath)]
  pub target: PathBuf,

  /// Exclude all directories (recursively) that have an exactly matching name.
  /// Accepts one or multiple values.
  #[arg(
    long = "exclude-dirs",
    alias = "xd",
    short = 'd',
    num_args = 1..,
    default_value = ""
  )]
  pub exclude_dirs: Vec<String>,

  /// Exclude all files that have an exactly matching name.
  /// Accepts one or multiple values.
  #[arg(
    long = "exclude-files",
    alias = "xf",
    short = 'f',
    num_args = 1..,
    default_value = ""
  )]
  pub exclude_files: Vec<String>,

  /// Exclude all paths that match a pattern.
  /// Accepts one or multiple glob-like patterns ('*', '**', '?' - eg: 'src/**/*.txt').
  /// Patterns are matched relative to the source directory.
  #[arg(
    value_parser = args_parse_pattern,
    long = "exclude-patterns",
    alias = "xp",
    short = 'p',
    num_args = 1..,
    default_value = ""
  )]
  pub exclude_patterns: Vec<Pattern>,
}


