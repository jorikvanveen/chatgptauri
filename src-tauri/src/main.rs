// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use serde::Serialize;
use tauri::async_runtime::Mutex;

mod gpt;

mod settings;
use settings::Settings;

use crate::conversation::Conversation;
mod conversation;

#[derive(Clone, Debug, Serialize)]
struct PromptResponse {
    content: String,
    cost: f32,
}

#[tauri::command]
async fn prompt(
    prompt: &str,
    conversation: tauri::State<'_, Conversation>,
    settings: tauri::State<'_, Mutex<Settings>>,
    window: tauri::Window,
) -> Result<(), String> {
    let settings = settings.lock().await;
    let api_key = {
        match settings.get_key().as_ref() {
            Some(key) => key.clone(),
            None => return Err("Please provide an API key in the settings menu".to_string()),
        }
    };

    let model = settings.get_model().to_string();

    conversation
        .prompt(prompt, &api_key, &model, &window)
        .await
        .unwrap();
    Ok(())
}

#[tauri::command]
async fn clear_messages(conversation: tauri::State<'_, Conversation>) -> Result<(), ()> {
    let _ = conversation.clear().await;
    Ok(())
}

fn main() {
    use settings::{get_settings, update_settings};
    // Load settings
    let settings = Settings::load().expect("Failed to load settings");

    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            prompt,
            clear_messages,
            get_settings,
            update_settings
        ])
        //.manage(Arc::new(Mutex::new(settings)))
        .manage(Conversation::new())
        .manage(Mutex::new(settings))
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
