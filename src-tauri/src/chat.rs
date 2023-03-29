use tauri::async_runtime::Mutex;
use std::sync::Arc;
use openai::chat::ChatCompletionMessage;

#[derive(Debug)]
pub struct ChatState(Arc<Mutex<Vec<ChatCompletionMessage>>>);

impl ChatState {
    pub fn new() -> Self {
        Self(Arc::new(Mutex::new(vec![])))
    }

    pub async fn get(&self) -> Vec<ChatCompletionMessage> {
        self.0.lock().await.clone() 
    }

    pub async fn push_message(&self, message: ChatCompletionMessage) {
        self.0.lock().await.push(message)
    }

    pub async fn clear(&self) {
        *self.0.lock().await = vec![]
    }
}


