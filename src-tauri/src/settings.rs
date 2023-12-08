use directories::BaseDirs;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io;
use std::path::PathBuf;
use tauri::async_runtime::Mutex;
use toml;

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "lowercase")]
pub enum Model {
    Gpt4Turbo,
    Gpt432K,
    Gpt4,
    Gpt3,
}

impl Model {
    pub fn calculate_cost(&self, prompt_tokens: usize, completion_tokens: usize) -> f32 {
        match self {
            Self::Gpt3 => (prompt_tokens + completion_tokens) as f32 * 0.000002,
            Self::Gpt4 => prompt_tokens as f32 * 0.00003 + completion_tokens as f32 * 0.00006,
            Self::Gpt432K => prompt_tokens as f32 * 0.00006 + completion_tokens as f32 * 0.00012,
            Self::Gpt4Turbo => prompt_tokens as f32 * 0.00001 + completion_tokens as f32 * 0.00003,
        }
    }
}

impl Model {
    pub fn to_string(&self) -> &str {
        match self {
            Self::Gpt4Turbo => "gpt-4-1106-preview",
            Self::Gpt432K => "gpt-4-32k",
            Self::Gpt4 => "gpt-4",
            Self::Gpt3 => "gpt-3.5-turbo",
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Settings {
    openai_key: Option<String>,
    model: Model,
}

impl Settings {
    pub fn _default() -> Self {
        Self {
            openai_key: None,
            model: Model::Gpt3,
        }
    }

    pub fn get_key(&self) -> &Option<String> {
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
        let deserialized: Self =
            toml::from_str(&settings_file_contents).expect("Failed to deserialize settings");
        Ok(deserialized)
    }

    fn get_settings_file() -> PathBuf {
        let base_dirs = BaseDirs::new().expect("Failed to get base directories");
        let mut config_file = base_dirs.config_dir().to_path_buf();
        config_file.push(".chatgptauri.toml");

        // Check if this file exists
        if let Err(_) = fs::metadata(&config_file) {
            // Create the file
            fs::write(&config_file, "model = \"gpt3\"")
                .expect("Failed to write to config directory");
        };
        config_file.to_path_buf()
    }
}

#[tauri::command]
pub async fn get_settings(settings: tauri::State<'_, Mutex<Settings>>) -> Result<Settings, ()> {
    println!("Getting settings");
    Ok(settings.lock().await.clone())
}

#[tauri::command]
pub async fn update_settings(
    settings_old: tauri::State<'_, Mutex<Settings>>,
    settings_new: Settings,
) -> Result<(), ()> {
    println!("Updating settings");
    *settings_old.lock().await = settings_new;
    settings_old
        .lock()
        .await
        .save()
        .expect("Failed to write to config file");
    Ok(())
}
