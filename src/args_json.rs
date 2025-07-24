use serde::Deserialize;

use crate::args_cli::Arguments;

#[derive(Debug, Deserialize)]
pub struct JSONConfig {
  /// Exclude exactly matching directory names globally (for all operations)
  #[serde(default)]
  pub exclude_dirs: Vec<String>,

  /// Exclude exactly matching file names globally (for all operations)
  #[serde(default)]
  pub exclude_files: Vec<String>,

  /// Exclude matching patterns globally (for all operations)
  #[serde(default)]
  pub exclude_patterns: Vec<String>,

  /// Defines sync operations to run
  #[serde(default)]
  pub operations: Vec<Arguments>,
}