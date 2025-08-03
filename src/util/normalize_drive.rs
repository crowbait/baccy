pub fn normalize_drive(s: String) -> String {
  #[cfg(windows)]
  {
    let mut s = s;
    if !s.ends_with(":\\") {
      if s.ends_with(':') {
        s.push_str("\\");
      } else if !s.ends_with(":\\") {
        s.push_str(":\\");
      }
    }
    s
  }

  #[cfg(not(windows))]
  {
    s
  }
}