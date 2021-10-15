use chrono::Local;
use env_logger::Builder;
use log::LevelFilter;
use std::io::Write;
use termion::*;
pub fn setup_logging() {
    Builder::new()
    .format(|buf, record| {
      match record.level() {
        log::Level::Error => {
          writeln!(
            buf,
            "{}{}[{} {}] [{}] - {}{}{}",
            style::Bold,
            color::Fg(color::Red),
            Local::now().format("%H:%M:%S"),
            record.level(),
            record.target(),
            record.args(),
            color::Fg(color::Reset),
            style::Reset,
          )
        }
        log::Level::Warn => {
          writeln!(
            buf,
            "{}{}[{} {}] [{}] - {}{}{}",
            style::Bold,
            color::Fg(color::Yellow),
            Local::now().format("%H:%M:%S"),
            record.level(),
            record.target(),
            record.args(),
            color::Fg(color::Reset),
            style::Reset,
          )
        }
        _ => {
          writeln!(
            buf,
            "[{} {}] {}[{}]{} - {}{}{}{}",
            Local::now().format("%H:%M:%S"),
            record.level(),
            color::Fg(color::LightBlack),
            record.target(),
            color::Fg(color::Reset),
            color::Fg(color::LightWhite),
            record.args(),
            color::Fg(color::Reset),
            style::Reset,
          )
        }
      }
    })
    .filter(None, LevelFilter::Info)
    .init();
}