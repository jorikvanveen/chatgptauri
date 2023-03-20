// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use std::sync::{Arc, Mutex};
use openai::chat::*;
// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

struct ChatState(Arc<Mutex<Vec<ChatCompletionMessage>>>);

impl ChatState {
    pub fn new() -> Self {
        Self(Arc::new(Mutex::new(vec![])))
    }

    pub fn get(&self) -> Vec<ChatCompletionMessage> {
        self.0.lock().unwrap().clone() 
    }

    pub fn push_message(&self, message: ChatCompletionMessage) {
        self.0.lock().unwrap().push(message)
    }
}

#[tauri::command]
async fn prompt(prompt: &str, messages: tauri::State<'_, ChatState>) -> Result<String, String> {
    println!("Pushing message: {}", prompt);
    messages.push_message(ChatCompletionMessage {
        role: ChatCompletionMessageRole::User,
        content: prompt.into(),
        name: None
    });
    println!("Pushed message: {}", prompt);

    println!("Requesting completion: {}", prompt);
    let completion = ChatCompletionBuilder::default()
        .model(openai::models::ModelID::Gpt3_5Turbo)
        .messages(messages.inner().get())
        .create()
        .await;
    println!("Got completion: {}", prompt);

    let completion_message = match completion {
        Ok(completion) => match completion {
            Ok(completion) => completion.choices[0].message.clone(),
            Err(e) => return Err(e.to_string())
        },
        Err(e) => return Err(e.to_string())
    };

    let response = completion_message.content.clone();

    println!("Pushing completion: {}", prompt);

    messages.push_message(completion_message);
    Ok(response)
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![greet, prompt])
        .manage(ChatState::new())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
