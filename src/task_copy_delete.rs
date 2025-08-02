use std::{fs, io::{Read, Write}, path::PathBuf};

use filetime::FileTime;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};

use crate::progress_helpers::PROGERSS_BAR_FILE;

pub struct Copy {
  pub from: PathBuf,
  pub to: PathBuf,
  pub relative: String,
  pub bytes: u64,
}

impl Copy {
  pub fn new(from: PathBuf, to: PathBuf, relative: String, bytes: u64) -> Self {
    Self{from, to, relative, bytes}
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

  pub fn execute_with_progress(&self, progress: &MultiProgress, worker_progress: &ProgressBar) -> std::io::Result<()> {
    let file_progress = progress.add(ProgressBar::new(self.bytes));
    file_progress.set_style(
      // ProgressStyle::with_template("Copying: {msg} {wide_bar} {bytes} / {total_bytes} ({bytes_per_sec})")
      ProgressStyle::with_template("Copying: {wide_bar} {bytes} / {total_bytes} ({bytes_per_sec})")
        .unwrap()
        .progress_chars(PROGERSS_BAR_FILE)
    );
    // file_progress.set_message(format!("{}",
    //   self.from.file_name().unwrap_or(OsStr::new("unknown")).to_str().unwrap()
    // ));

    let mut reader = fs::File::open(&self.from)?;
    let mut writer = fs::File::create(&self.to)?;
    // 1MiB buffer is too big for stack (1MiB total stack size...)
    // let mut buffer = [0u8; 1024*1024];
    // 4MiB vector-buffer lives on heap, better performance overall
    let mut buffer = vec![0u8; 1024 * 1024 * 4];
    let mut copied: u64 = 0;
    let worker_start_pos = worker_progress.position();

    loop {
      let num_bytes = reader.read(&mut buffer)?;
      if num_bytes == 0 {break;}
      writer.write_all(&buffer[..num_bytes])?;
      copied += num_bytes as u64;
      file_progress.set_position(copied);
      worker_progress.set_position(copied + worker_start_pos);
    };

    self.copy_mtime();
    file_progress.finish_and_clear();
    progress.remove(&file_progress);
    Ok(())
  }
}



pub struct Delete {
  pub path: PathBuf,
  pub relative: String,
}

impl Delete {
  pub fn new(path: PathBuf, relative: String) -> Self {
    Self{path, relative}
  }
}