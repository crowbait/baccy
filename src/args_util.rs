use std::path::PathBuf;

use glob::{Pattern, PatternError};

pub fn args_validate_existing_path(s: &str) -> Result<PathBuf, String> {
  let path = PathBuf::from(s);
  if path.exists() {
    Ok(path)
  } else {
    Err(format!("Path '{}' does not exist.", s))
  }
}

pub fn args_parse_path(s: &str) -> Result<PathBuf, String> {
  Ok(PathBuf::from(s))
}

// clap parses multi-value arguments one at a time, not the entire vector at once
pub fn args_parse_pattern(s: &str) -> Result<Pattern, PatternError> {
  Pattern::new(s)
}