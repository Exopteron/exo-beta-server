use chrono::Local;
use env_logger::Builder;
use log::LevelFilter;
use std::io::Write;
pub fn setup_logging() {
    Builder::new()
    .format(|buf, record| {
      writeln!(
        buf,
        "[{} {}] [{}] - {}",
        Local::now().format("%H:%M:%S"),
        record.level(),
        record.target(),
        record.args(),
      )
    })
    .filter(None, LevelFilter::Info)
    .init();
}