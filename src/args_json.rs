use serde::Deserialize;

use crate::args_cli::Arguments;

#[derive(Debug, Deserialize)]
pub struct JSONConfig {
  /// Exclude exactly matching directory names globally (for all operations).
  #[serde(default)]
  pub exclude_dirs: Vec<String>,

  /// Exclude exactly matching file names globally (for all operations).
  #[serde(default)]
  pub exclude_files: Vec<String>,

  /// Exclude matching patterns globally (for all operations).
  #[serde(default)]
  pub exclude_patterns: Vec<String>,

  /// Include only exactly matching directory names globally (for all operations).
  /// Checked after exclusions.
  #[serde(default)]
  pub include_dirs: Vec<String>,

  /// Include only exactly matching file names globally (for all operations).
  /// Checked after exclusions.
  #[serde(default)]
  pub include_files: Vec<String>,

  /// Include only matching patterns globally (for all operations).
  /// Checked after exclusions.
  #[serde(default)]
  pub include_patterns: Vec<String>,

  /// Sets "print files copied and deleted" (for all operations).
  /// Sets all operations to "true" if set, no effect if set to "false".
  #[serde(default)]
  pub log_files: bool,

  /// Defines sync operations to run.
  #[serde(default)]
  pub operations: Vec<Arguments>,
}