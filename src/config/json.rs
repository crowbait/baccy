use serde::Deserialize;

use crate::config::cli::Arguments;

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

  /// Forces inclusion of exactly matching directory names globally (for all operations),
  /// overriding all exclude and include rules.
  #[serde(default)]
  pub force_include_dirs: Vec<String>,

  /// Forces inclusion of exactly matching file names globally (for all operations),
  /// overriding all exclude and include rules.
  #[serde(default)]
  pub force_include_files: Vec<String>,

  /// Forces inclusion of matching patterns globally (for all operations),
  /// overriding all exclude and include rules.
  #[serde(default)]
  pub force_include_patterns: Vec<String>,

  /// Sets "print files copied and deleted" (for all operations).
  /// Sets all operations to "true" if set, no effect if set to "false".
  #[serde(default)]
  pub log_files: bool,

  /// Prints information about disk usage after all operations have completed.
  /// Expects an array of mount points / drive letters.
  #[serde(default)]
  pub drive_info: Vec<String>,

  /// Runs commands on the system shell after all operations have completed.
  #[serde(default)]
  pub post_commands: Vec<String>,

  /// Waits ("Press Enter to continue") after all operations have completed.
  #[serde(default)]
  pub wait_on_end: bool,

  /// Defines sync operations to run.
  #[serde(default)]
  pub operations: Vec<Arguments>,
}