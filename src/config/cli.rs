use clap::{Parser, ValueHint};
use serde::Deserialize;
use std::path::PathBuf;

#[derive(Debug, Parser, Deserialize)]
#[command(name = "baccy", version, about = "Efficient and informative directory sync")]
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

  /// Include only directories (recursively) that have an exactly matching name.
  /// Accepts one or multiple values. This is checked after exclusions.
  /// In JSON-config-mode, this will be merged with the global excludes defined in the JSON.
  #[arg(
    long = "include-dirs",
    alias = "id",
    num_args = 0.. // 0 makes it optional, 1.. would have been "required with at least 1 argument"
  )]
  #[serde(default)]
  pub include_dirs: Vec<String>,

  /// Include only files that have an exactly matching name.
  /// Accepts one or multiple values. This is checked after exclusions.
  /// In JSON-config-mode, this will be merged with the global excludes defined in the JSON.
  #[arg(
    long = "include-files",
    alias = "if",
    num_args = 0.. // 0 makes it optional, 1.. would have been "required with at least 1 argument"
  )]
  #[serde(default)]
  pub include_files: Vec<String>,

  /// Include only paths that match a pattern.
  /// Accepts one or multiple glob-like patterns ('*', '**', '?' - eg: 'src/**/*.txt').
  /// Patterns are matched relative to the source directory. This is checked after exclusions.
  /// In JSON-config-mode, this will be merged with the global excludes defined in the JSON.
  #[arg(
    long = "include-patterns",
    alias = "ip",
    num_args = 0..
  )]
  #[serde(default)]
  pub include_patterns: Vec<String>,

  /// Forces inclusion of matching directory names, overriding all exclude and include rules.
  #[arg(
    long = "force-include-dirs",
    alias = "fid",
    num_args = 0.. // 0 makes it optional, 1.. would have been "required with at least 1 argument"
  )]
  #[serde(default)]
  pub force_include_dirs: Vec<String>,

  /// Forces inclusion of matching file names, overriding all exclude and include rules.
  /// In JSON-config-mode, this will be merged with the global excludes defined in the JSON.
  #[arg(
    long = "force-include-files",
    alias = "fif",
    num_args = 0.. // 0 makes it optional, 1.. would have been "required with at least 1 argument"
  )]
  #[serde(default)]
  pub force_include_files: Vec<String>,

  /// Forces inclusion of paths matching a pattern, overriding all exclude and include rules.
  /// Accepts one or multiple glob-like patterns ('*', '**', '?' - eg: 'src/**/*.txt').
  /// Patterns are matched relative to the source directory.
  /// In JSON-config-mode, this will be merged with the global excludes defined in the JSON.
  #[arg(
    long = "force-include-patterns",
    alias = "fip",
    num_args = 0..
  )]
  #[serde(default)]
  pub force_include_patterns: Vec<String>,


  /// Skips the "delete files from target that are not present in source" step.
  /// If in JSON-config mode: sets no-delete for all operations in JSON, overriding per-operation setting.
  #[arg(
    long = "no-delete",
    alias = "nd",
    action // = false if not given, true if present
  )] 
  #[serde(default)] // defaults to false
  pub no_delete: bool,
  
  /// Logs all copied and deleted files.
  /// If in JSON-config-mode: sets log-files for all operations in JSON, overriding per-operation setting.
  #[arg(
    long = "log-files",
    short = 'l',
    action // = false if not given, true if present
  )] 
  #[serde(default)] // defaults to false
  pub log_files: bool,

  /// Logs exclude-, include-, and force-include rules for each operation.
  /// If in JSON-config-mode: sets log-rules for all operations in JSON, overriding per-operation setting.
  #[arg(
    long = "log-rules",
    alias = "lr",
    action // = false if not given, true if present
  )]
  #[serde(default)] // defaults to false
  pub log_rules: bool,
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