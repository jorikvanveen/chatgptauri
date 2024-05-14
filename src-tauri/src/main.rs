// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use std::sync::atomic::AtomicBool;

use conversation::SerializedConversation;
use serde::Serialize;
use tauri::async_runtime::Mutex;

mod conversation;
mod gpt;
mod settings;

use crate::conversation::{Conversation, CancelState};
use settings::Settings;

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
    cancel_state: tauri::State<'_, CancelState>,
    window: tauri::Window,
) -> Result<(), String> {
    let cancel_state = cancel_state.inner().clone();
    let settings = settings.lock().await;
    let api_key = {
        match settings.get_key().as_ref() {
            Some(key) => key.clone(),
            None => return Err("Please provide an API key in the settings menu".to_string()),
        }
    };

    let model = settings.get_model();
    let prompt_result = conversation.prompt(prompt, &api_key, model.clone(), &window, cancel_state).await;

    if let Err(e) = prompt_result {
        return Err(e.to_string());
    }

    Ok(())
}

#[tauri::command]
async fn clear_messages(conversation: tauri::State<'_, Conversation>) -> Result<(), String> {
    if let Err(e) = conversation.clear().await {
        return Err(e.to_string());
    }

    Ok(())
}

#[tauri::command]
async fn save(
    conversation: tauri::State<'_, Conversation>,
    settings: tauri::State<'_, Mutex<Settings>>,
) -> Result<(), String> {
    let settings = settings.lock().await;

    let api_key = {
        match settings.get_key().as_ref() {
            Some(key) => key.clone(),
            None => return Err("Please provide an API key in the settings menu".to_string()),
        }
    };

    if let Err(e) = conversation.save(&api_key).await {
        return Err(e.to_string());
    };

    Ok(())
}

#[tauri::command]
async fn list_conversations() -> Result<Vec<SerializedConversation>, String> {
    match Conversation::list_conversations().await {
        Ok(conversations) => Ok(conversations),
        Err(e) => Err(e.to_string()),
    }
}

#[tauri::command]
fn get_current_conversation_id(conversation: tauri::State<'_, Conversation>) -> Result<u32, ()> {
    Ok(conversation.get_id())
}

#[tauri::command]
async fn load_conversation(
    conversation: tauri::State<'_, Conversation>,
    window: tauri::Window,
    new_conversation_id: u64,
) -> Result<(), String> {
    if let Err(e) = conversation.load(new_conversation_id).await {
        return Err(e.to_string());
    };

    let messages = conversation.get_messages().lock().await;

    window.emit("refresh_messages", &*messages).unwrap();

    Ok(())
}

#[tauri::command]
async fn reset_conversation(
    conversation: tauri::State<'_, Conversation>,
    window: tauri::Window,
) -> Result<(), ()> {
    conversation.reset().await;
    let messages = conversation.get_messages().lock().await;
    window.emit("refresh_messages", &*messages).unwrap();
    Ok(())
}

#[tauri::command]
async fn cancel(
    cancel_state: tauri::State<'_, CancelState>
) -> Result<(), ()> {
    cancel_state.transmit_cancel();
    Ok(())
}

fn main() {
    use settings::{get_settings, update_settings};
    // Load settings
    let settings = Settings::load().expect("Failed to load settings");

    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            prompt,
            cancel,
            clear_messages,
            get_settings,
            update_settings,
            save,
            list_conversations,
            get_current_conversation_id,
            load_conversation,
            reset_conversation,
        ])
        //.manage(Arc::new(Mutex::new(settings)))
        .manage(Conversation::new())
        .manage(Mutex::new(settings))
        .manage(CancelState::new())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
