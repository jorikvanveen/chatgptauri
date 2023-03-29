use std::io;
use std::fs;
use std::path::PathBuf;
use directories::BaseDirs;
use toml;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct Settings {
    openai_key: String,
    model: Model
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Model {
    Gpt4,
    Gpt3
}

impl Settings {
    pub fn get_key(&self) -> &str {
        return &self.openai_key;
    }

    pub fn get_model(&self) -> &Model {
        &self.model
    }

    pub fn save(&self) -> Result<(), io::Error> {
        let settings_file = Self::get_settings_file();
        let serialized = toml::to_string(self).expect("Failed to serialize settings");
        fs::write(settings_file, serialized)?;
        Ok(())
    }

    pub fn load() -> Result<Self, io::Error> {
        let settings_file = Self::get_settings_file();
        let settings_file_contents = fs::read_to_string(settings_file)?;
        let deserialized: Self = toml::from_str(&settings_file_contents).expect("Failed to serialize settings");
        Ok(deserialized)
    }

    fn get_settings_file() -> PathBuf {
        let base_dirs = BaseDirs::new().expect("Failed to get base directories");
        let mut config_file = base_dirs.config_dir().to_path_buf();
        config_file.push(".chatgptauri.toml");

        // Check if this file exists
        todo!();
        config_file.to_path_buf()
    }
}

