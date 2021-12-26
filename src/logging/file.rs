use std::{path::PathBuf, fs::{self, File, OpenOptions}, io::Write};

use anyhow::bail;
use chrono::Local;

#[derive(Clone)]
pub struct LogManager {
    directory: PathBuf,
    open_log: Option<PathBuf>
}
impl LogManager {
    pub fn new(path: impl Into<PathBuf>) -> anyhow::Result<Self> {
        let path = path.into();
        fs::create_dir_all(&path)?;
        Ok(Self { directory: path, open_log: None })
    }
    pub fn append(&self, to_append: String) -> anyhow::Result<()> {
        if !self.open_log.is_some() {
            bail!("No open log")
        }
        let mut file = OpenOptions::new().append(true).open(self.open_log.as_ref().unwrap())?;
        file.write_all(to_append.as_bytes())?;
        file.write_all(b"\n")?;
        Ok(())
    }
    pub fn open(&mut self) -> anyhow::Result<()> {
        let mut current = self.directory.clone();
        current.push("latest.log");
        self.open_log = Some(current.clone());
        File::create(current)?;
        Ok(())
    }
    pub fn close(&self) -> anyhow::Result<()> {
        let now = Local::now();
        let mut new = self.directory.clone();
        new.push(&format!("{}.log", now.to_rfc3339()));
        fs::copy(self.open_log.as_ref().unwrap(), new)?;
        Ok(())
    }
}