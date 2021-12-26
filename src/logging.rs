use chrono::Local;
use env_logger::Builder;
use log::LevelFilter;
use std::io::Write;
use termion::*;

use self::file::LogManager;
pub mod file;
pub fn setup_logging() -> LogManager {
    let mut appender = LogManager::new("log/").unwrap();
    appender.open().unwrap();
    let a2 = appender.clone();
    Builder::new()
        .format(move |buf, record| {
            let record = record.to_owned();
            let args = record.args().to_string();
            let format = format!(
                "[{} {}] [{}] - {}",
                Local::now().format("%H:%M:%S"),
                record.level(),
                record.target(),
                args,
            );
            appender.append(format);
            let args = args.replace("§4", &color::Fg(color::Red).to_string());
            let args = args.replace("§c", &color::Fg(color::LightRed).to_string());
            let args = args.replace("§6", &color::Fg(color::Yellow).to_string());
            let args = args.replace("§e", &color::Fg(color::LightYellow).to_string());
            let args = args.replace("§2", &color::Fg(color::Green).to_string());
            let args = args.replace("§a", &color::Fg(color::LightGreen).to_string());
            let args = args.replace("§b", &color::Fg(color::LightBlue).to_string());
            let args = args.replace("§3", &color::Fg(color::LightBlue).to_string());
            let args = args.replace("§1", &color::Fg(color::Blue).to_string());
            let args = args.replace("§9", &color::Fg(color::Blue).to_string());
            let args = args.replace("§d", &color::Fg(color::LightMagenta).to_string());
            let args = args.replace("§5", &color::Fg(color::Magenta).to_string());
            let args = args.replace("§f", &color::Fg(color::White).to_string());
            let args = args.replace("§7", &color::Fg(color::LightWhite).to_string());
            let args = args.replace("§8", &color::Fg(color::LightBlack).to_string());
            let args = args.replace("§0", &color::Fg(color::Black).to_string());

            let args = args.replace("§l", &style::Bold.to_string());
            let args = args.replace("§m", &style::CrossedOut.to_string());
            let args = args.replace("§n", &style::Underline.to_string());
            let args = args.replace("§o", &style::Italic.to_string());
            let args = args.replace(
                "§r",
                &format!("{}{}", &color::Fg(color::Reset), &style::Reset.to_string()),
            );
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
                        args,
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
                        args,
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
                        args,
                        color::Fg(color::Reset),
                        style::Reset,
                    )
                }
            }
        })
        .filter(None, LevelFilter::Info)
        .init();
    a2
}
