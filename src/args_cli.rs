use clap::{Parser, ValueHint};
use serde::Deserialize;
use std::path::PathBuf;

#[derive(Debug, Parser, Deserialize)]
#[command(name = "Backrust", version, about = "Efficient and informative directory sync")]
pub struct Arguments {
  /// First argument: path, either directory *or* JSON configuration:
  /// Directory: directory which will be synced.
  /// JSON file: JSON batch-mode configuration.
  /// See second argument for how this is interpreted.
  #[arg(index = 1, value_parser = args_validate_existing_path, value_hint = ValueHint::DirPath)]
  pub source: PathBuf,

  /// Directory into which to sync.
  /// IF this is given, the first argument is interpreted as source directory.
  /// If this is NOT given, the first argument is interpreted as JSON config.
  #[arg(index = 2, value_hint = ValueHint::DirPath)]
  pub target: Option<PathBuf>,

  /// Exclude all directories (recursively) that have an exactly matching name.
  /// Accepts one or multiple values.
  /// In JSON-config-mode, this will be merged with the global excludes defined in the JSON.
  #[arg(
    long = "exclude-dirs",
    alias = "xd",
    short = 'd',
    num_args = 0.. // 0 makes it optional, 1.. would have been "required with at least 1 argument"
  )]
  #[serde(default)]
  pub exclude_dirs: Vec<String>,

  /// Exclude all files that have an exactly matching name.
  /// Accepts one or multiple values.
  /// In JSON-config-mode, this will be merged with the global excludes defined in the JSON.
  #[arg(
    long = "exclude-files",
    alias = "xf",
    short = 'f',
    num_args = 0..
  )]
  #[serde(default)]
  pub exclude_files: Vec<String>,

  /// Exclude all paths that match a pattern.
  /// Accepts one or multiple glob-like patterns ('*', '**', '?' - eg: 'src/**/*.txt').
  /// Patterns are matched relative to the source directory.
  /// In JSON-config-mode, this will be merged with the global excludes defined in the JSON.
  #[arg(
    long = "exclude-patterns",
    alias = "xp",
    short = 'p',
    num_args = 0..
  )]
  #[serde(default)]
  pub exclude_patterns: Vec<String>,

  /// Skips the "delete files from target that are not present in source" step.
  /// Sets no-delete for all operations in JSON, if in JSON-config-mode.
  #[arg(
    long = "no-delete",
    alias = "nd",
    action // = false if not given, true if present
  )] 
  #[serde(default)] // defaults to false
  pub no_delete: bool,
}


impl Arguments {
  pub fn is_json_config(&self) -> bool {
    match &self.target {
      Some(_) => false,
      None => true
    }
  }
}


fn args_validate_existing_path(s: &str) -> Result<PathBuf, String> {
  let path = PathBuf::from(s);
  if path.exists() {
    Ok(path)
  } else {
    Err(format!("Path '{}' does not exist.", s))
  }
}