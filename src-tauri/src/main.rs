// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use openai::chat::*;
use serde::Serialize;
use tauri::async_runtime::Mutex;
use std::sync::Arc;

mod gpt4;

mod settings;
use settings::Settings;

mod chat;
use chat::ChatState;

#[derive(Clone, Debug, Serialize)]
struct PromptResponse {
    content: String,
    cost: f32
}

// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
async fn prompt(prompt: &str, messages: tauri::State<'_, ChatState>) -> Result<PromptResponse, String> {
    println!("Pushing message: {}", prompt);
    messages.push_message(ChatCompletionMessage {
        role: ChatCompletionMessageRole::User,
        content: prompt.into(),
        name: None
    }).await;
    println!("Pushed message: {}", prompt);

    println!("Requesting completion: {}", prompt);
    let completion = ChatCompletionBuilder::default()
        .model("gpt-3.5-turbo")
        .messages(messages.inner().get().await)
        .create()
        .await;
    println!("Got completion: {}", prompt);

    let completion = match completion {
        Ok(completion) => match completion {
            Ok(completion) => completion.clone(),
            Err(e) => return Err(e.to_string())
        },
        Err(e) => return Err(e.to_string())
    };

    let completion_message = &completion.choices[0].message;
    let response = completion_message.content.clone();
    let usage = completion.usage.unwrap();
    let cost = usage.total_tokens as f32 * 0.000002;

    println!("Pushing completion: {}", prompt);

    messages.push_message(completion_message.clone()).await;
    Ok(PromptResponse {
        cost, content: response
    })
}

#[tauri::command]
async fn prompt_gpt4(prompt: &str, messages: tauri::State<'_, Arc<Mutex<Vec<gpt4::Message>>>>) -> Result<PromptResponse, String> {
    let mut messages = messages.lock().await;
    messages.push(gpt4::Message::user(prompt.into()));

    let api_key = std::env::var("OPENAI_KEY").expect("Please provide an OPENAI_KEY environment variable");
    let response = gpt4::Request::new(messages.to_vec()).do_request(&api_key).await;

    println!("{:#?}", messages);
    let response = match response {
        Ok(response) => response,
        Err(e) => {
            eprintln!("{:#?}", e);
            return Err(e.to_string())
        }
    };

    let content = response.choices[0].message.get_content().to_string();
    let usage = response.usage;
    let cost = usage.prompt_tokens as f32 * 0.00003 + usage.completion_tokens as f32 * 0.00006;
    
    Ok(PromptResponse {
        content, cost
    })
}

#[tauri::command]
async fn clear_messages(messages3: tauri::State<'_, ChatState>, messages4: tauri::State<'_, Arc<Mutex<Vec<gpt4::Message>>>>) -> Result<(), ()> {
    println!("Clearing message state");
    messages3.clear().await;
    *messages4.lock().await = vec![];
    Ok(())
}

fn main() {
    // Load settings
    //let settings = Settings::load().expect("Failed to load settings");

    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![greet, prompt, clear_messages, prompt_gpt4])
        //.manage(Arc::new(Mutex::new(settings)))
        .manage(ChatState::new())
        .manage(Arc::new(Mutex::new(Vec::<gpt4::Message>::new())))
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
