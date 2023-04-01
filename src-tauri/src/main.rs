// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use serde::Serialize;
use tauri::async_runtime::Mutex;

mod gpt;

mod settings;
use settings::Settings;

#[derive(Clone, Debug, Serialize)]
struct PromptResponse {
    content: String,
    cost: f32
}

#[tauri::command]
async fn prompt(prompt: &str, messages: tauri::State<'_, Mutex<Vec<gpt::Message>>>, settings: tauri::State<'_, Mutex<Settings>>) -> Result<PromptResponse, String> {
    let mut messages = messages.lock().await;
    messages.push(gpt::Message::user(prompt.into()));
    
    let settings = settings.lock().await;
    let api_key = match settings.get_key().as_ref() {
        Some(key) => key,
        None => return Err("Please provide an API key in the settings menu".to_string())
    };
    let model = settings.get_model();
    let response = gpt::Request::new(messages.to_vec(), &model.to_string()).do_request(api_key).await;

    println!("{:#?}", messages);
    let response = match response {
        Ok(response) => response,
        Err(e) => {
            eprintln!("{:#?}", e);
            return Err(e.to_string())
        }
    };

    let content = response.choices[0].message.get_content().to_string();
    messages.push(gpt::Message::assistant((&content).into()));

    let usage = response.usage;
    let cost = model.calculate_cost(usage.prompt_tokens, usage.completion_tokens);
    
    Ok(PromptResponse {
        content, cost
    })
}

#[tauri::command]
async fn clear_messages(messages: tauri::State<'_, Mutex<Vec<gpt::Message>>>) -> Result<(), ()> {
    println!("Clearing message state");
    *messages.lock().await = vec![];
    Ok(())
}

#[tauri::command]
async fn get_settings(settings: tauri::State<'_, Mutex<Settings>>) -> Result<Settings, ()> {
    println!("Getting settings");
    Ok(settings.lock().await.clone())
}

#[tauri::command]
async fn update_settings(settings_old: tauri::State<'_, Mutex<Settings>>, settings_new: Settings) -> Result<(), ()> {
    println!("Updating settings");
    *settings_old.lock().await = settings_new;
    settings_old.lock().await.save().expect("Failed to write to config file");
    Ok(())
}

fn main() {
    // Load settings
    let settings = Settings::load().expect("Failed to load settings");

    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![prompt, clear_messages, get_settings, update_settings])
        //.manage(Arc::new(Mutex::new(settings)))
        .manage(Mutex::new(Vec::<gpt::Message>::new()))
        .manage(Mutex::new(settings))
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
