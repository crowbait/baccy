const KB: u64 = 1024;
const MB: u64 = 1024*1024;
const GB: u64 = 1024*1024*1024;
const TB: u64 = 1024*1024*1024*1024;

/// Returns a string, displaying bytes as an appropriate unit.
pub fn bytes_to_string(bytes: u64) -> String {
  match bytes {
    0..MB => {format!("{:.2} kiB", bytes as f64 / KB as f64)},
    MB..GB => {format!("{:.2} MiB", bytes as f64 / MB as f64)},
    GB..TB => {format!("{:.2} GiB", bytes as f64 / GB as f64)},
    _ => {format!("{:.2} kiB", bytes as f64 / TB as f64)},
  }
}