use std::sync::{Arc, Mutex};
use openai::chat::ChatCompletionMessage;

pub struct ChatState(Arc<Mutex<Vec<ChatCompletionMessage>>>);

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

    pub fn clear(&self) {
        *self.0.lock().unwrap() = vec![]
    }
}


