use std::fs::{File, self};

use ahash::AHashMap;
use once_cell::sync::OnceCell;
use std::io::Read;

use crate::configuration::CONFIGURATION;
// "Translation" manager, more for formatting
pub struct TranslationManager {
    keys: AHashMap<String, String>,
}
impl Default for TranslationManager {
    fn default() -> Self {
        let mut keys = AHashMap::new();
        keys.insert("multiplayer.player.joined".to_string(), "%s joined the game.".to_string());
        keys.insert("multiplayer.player.left".to_string(), "%s left the game.".to_string());
        keys.insert("multiplayer.disconnect.server_shutdown".to_string(), "Server closed".to_string());
        keys.insert("chat.type.text".to_string(), "<%s> %s".to_string());
        Self { keys }
    }
}
impl TranslationManager {
    pub fn initialize() -> anyhow::Result<Self> {
        let us = match TranslationManager::from_file(&CONFIGURATION.translation_file) {
            Ok(t) => t,
            Err(_) => {
                let us = TranslationManager::default();
                us.to_file(&CONFIGURATION.translation_file)?;
                us
            } 
        };
        log::info!("Loaded {} translation keys", us.keys.len());
        Ok(us)
    }
    pub fn to_file(&self, file: &str) -> anyhow::Result<()> {
        let string = serde_json::to_string_pretty(&self.keys)?;
        fs::write(file, string)?;
        Ok(())
    }
    pub fn from_file(file: &str) -> anyhow::Result<Self> {
        let mut file = File::open(&file)?;
        let mut string = String::new();
        file.read_to_string(&mut string)?;
        let keys: AHashMap<String, String> = serde_json::from_str(&string)?;
        Ok(Self { keys })
    }
    pub fn translate(&self, input: &str, format: Option<Vec<String>>) -> String {
        if let Some(value) = self.keys.get(input) {
            if let Some(format) = format {
                let mut value = value.to_owned();
                for entry in format.iter() {
                    value = value.replacen("%s", &entry.to_string(), 1);
                }
                value
            } else {
                return value.to_string();
            }
        } else {
            return input.to_string();
        }
    }
}