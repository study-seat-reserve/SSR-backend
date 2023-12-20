use crate::utils::*;
use ansi_term::Colour;
use env_logger::Target;
use log::{Level, LevelFilter};
use regex;
use std::{
  env,
  fs::{self, OpenOptions},
  io::Write,
  path::Path,
};

pub fn init_logger(level: LevelFilter) {
  let root = env::var("ROOT").expect("Failed to get root path");
  let logfiles = format!("{}/logfiles/", root);
  let path = Path::new(logfiles.as_str());

  if !path.exists() {
    fs::create_dir_all(&path).expect("Failed to create logfiles");
  }

  let now = get_today();
  let file_name = format!("{}/logfiles/{}.txt", root, now);

  let file = OpenOptions::new()
    .create(true)
    .append(true)
    .open(&file_name)
    .expect("Failed to open log file");

  let target = Target::Pipe(Box::new(file));

  env_logger::Builder::new()
    .filter(None, level)
    .target(target)
    .format(|buf, record| {
      let level_style = match record.level() {
        Level::Error => Colour::Red.bold().paint("Error"),
        Level::Warn => Colour::Yellow.bold().paint("Warn"),
        Level::Info => Colour::Green.bold().paint("Info"),
        Level::Debug => Colour::Blue.bold().paint("Debug"),
        Level::Trace => Colour::White.bold().paint("Trace"),
      };

      let message = format!(
        "[{}] [{}] {}",
        get_datetime().format("%Y-%m-%d %H:%M:%S%.3f"),
        level_style,
        record.args()
      );
      println!("{}", &message);

      let message = remove_ansi_escape_codes(&message);
      writeln!(buf, "{}", message)
    })
    .init();
}

fn remove_ansi_escape_codes(s: &str) -> String {
  let pattern = "\x1B\\[[^m]*m";
  regex::Regex::new(pattern)
    .unwrap()
    .replace_all(s, "")
    .to_string()
}

#[cfg(test)]
mod tests {
  use super::*;
  use std::{
    fs,
    io::{Read, Seek, Write},
  };

  #[test]
  fn test_init_logger() {
    let tmp_dir = tempfile::TempDir::new().expect("create temp dir");
    std::env::set_var("ROOT", tmp_dir.path());

    init_logger(LevelFilter::Info);

    assert!(tmp_dir.path().join("logfiles").exists());

    let today = chrono::Local::now().format("%Y-%m-%d").to_string();
    let log_file = tmp_dir.path().join(format!("logfiles/{}.txt", today));
    assert!(log_file.exists());

    let mut file = fs::OpenOptions::new()
      .write(true)
      .open(log_file)
      .expect("open log file");

    let log_msg = "[INFO] [2023-03-10 15:16:17.012] Test log message\n";
    writeln!(&mut file, "{}", log_msg).expect("write log");

    let mut contents = String::new();
    file
      .seek(std::io::SeekFrom::Start(0))
      .expect("seek to start");
    file.read_to_string(&mut contents).expect("read file");

    assert_eq!(contents, log_msg);

    std::env::remove_var("ROOT");
  }

  #[test]
  fn test_remove_ansi_escape_codes() {
    let input = "";
    assert_eq!(remove_ansi_escape_codes(input), "");

    let input = "軟工測試!";
    assert_eq!(remove_ansi_escape_codes(input), "軟工測試!");

    let input = "\x1B[31m軟工\x1B[0m\x1B[32m測試\x1B[0m";
    assert_eq!(remove_ansi_escape_codes(input), "軟工測試!");
  }
}
