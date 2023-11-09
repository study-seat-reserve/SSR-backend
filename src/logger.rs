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
  let pattern = "\x1B\\[[^m]*m"; // 匹配 ANSI 转义码的正则表达式
  regex::Regex::new(pattern)
    .unwrap()
    .replace_all(s, "")
    .to_string()
}
