use std::{ffi::OsStr, fs, path::PathBuf, io::{Read, Write}};

use filetime::FileTime;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};

use crate::progress_helpers::PROGERSS_BAR_FILE;

pub struct Copy {
  pub from: PathBuf,
  pub to: PathBuf,
}

impl Copy {
  pub fn new(from: PathBuf, to: PathBuf) -> Copy {
    Copy{from, to}
  }

  pub fn get_filesize(&self) -> u64 {
    fs::metadata(&self.from).map(|m| m.len()).unwrap_or(0)
  }

  fn copy_mtime(&self) {
    if let Ok(meta) = fs::metadata(&self.from) {
      if let Ok(mtime) = meta.modified() {
        let _ = filetime::set_file_mtime(&self.to, FileTime::from_system_time(mtime));
      }
    }
  }

  pub fn execute(&self) -> std::io::Result<()> {
    let res = fs::copy(&self.from, &self.to).map(|_| ());
    self.copy_mtime();
    res
  }

  pub fn execute_with_progress(&self, progress: &MultiProgress) -> std::io::Result<()> {
    let file_progress = progress.add(ProgressBar::new(self.get_filesize()));
    file_progress.set_style(
      ProgressStyle::with_template("Copying: {msg} {wide_bar} {bytes} / {total_bytes} ({bytes_per_sec})")
        .unwrap()
        .progress_chars(PROGERSS_BAR_FILE)
    );
    file_progress.set_message(format!("{}",
      self.from.file_name().unwrap_or(OsStr::new("unknown")).to_str().unwrap()
    ));

    let mut reader = fs::File::open(&self.from)?;
    let mut writer = fs::File::create(&self.to)?;
    // 1MiB buffer is too big for stack (1MiB total stack size...)
    // let mut buffer = [0u8; 1024*1024];
    // 4MiB vector-buffer lives on heap, better performance overall
    let mut buffer = vec![0u8; 1024 * 1024 * 4];
    let mut copied: u64 = 0;

    loop {
      let num_bytes = reader.read(&mut buffer)?;
      if num_bytes == 0 {break;}
      writer.write_all(&buffer[..num_bytes])?;
      copied += num_bytes as u64;
      file_progress.set_position(copied);
    };

    self.copy_mtime();
    file_progress.finish_and_clear();
    progress.remove(&file_progress);
    Ok(())
  }
}