use crate::gpt::{MessageDelta, Role};
use reqwest_eventsource::CannotCloneRequestError;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use tauri::async_runtime as tokio;
use tokio::Mutex;
use tokio_stream::StreamExt;

use super::gpt;
use gpt::Message;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum PromptError {
    #[error("There is already a request in progress")]
    ConversationLocked,

    #[error("Something went wrong while making the API request")]
    RequestError(#[from] CannotCloneRequestError),
}

pub struct Conversation {
    is_locked: Arc<AtomicBool>, // Conversation will be locked while the server is streaming a response
    // to us.
    messages: Arc<Mutex<Vec<Message>>>,
}

impl Conversation {
    pub fn new() -> Self {
        Self {
            is_locked: Arc::new(AtomicBool::new(false)),
            messages: Arc::new(Mutex::new(vec![])),
        }
    }

    pub fn get_messages(&self) -> &Arc<Mutex<Vec<Message>>> {
        &self.messages
    }

    pub async fn clear(&self) -> Result<(), PromptError> {
        if self.is_locked.load(Ordering::SeqCst) {
            return Err(PromptError::ConversationLocked);
        }

        *self.messages.lock().await = vec![];
        Ok(())
    }

    pub async fn prompt(
        &self,
        prompt: &str,
        api_key: &str,
        model: &str,
        window: &tauri::Window,
    ) -> Result<(), PromptError> {
        dbg!(model);
        // Check if conversation is locked
        if self.is_locked.load(Ordering::SeqCst) {
            return Err(PromptError::ConversationLocked);
        };

        {
            let mut messages = self.messages.lock().await;
            // Add the prompt to the messages
            messages.push(Message::new(Role::user, prompt.into()));

            // Add empty assistant message that the deltas will be applied to
            messages.push(Message::new(Role::assistant, "".into()));
        }

        // Start background task that makes the openai request and applies the received deltas to
        // the messages.

        {
            let is_locked = Arc::clone(&self.is_locked);
            let messages = Arc::clone(&self.messages);
            let mut delta_stream = gpt::Request::new(self.messages.lock().await.clone(), model)
                .do_request(api_key)?;
            let window = window.clone();

            tokio::spawn(async move {
                println!("Running delta task");
                is_locked.store(true, Ordering::SeqCst);
                window.emit("lock", true).unwrap();

                while let Some(delta) = delta_stream.next().await {
                    match delta {
                        Ok(delta) => {
                            if let MessageDelta::Delta(content) = delta {
                                // Add this delta to the latest message
                                let mut messages = messages.lock().await;
                                let last_message_index = messages.len() - 1;
                                messages[last_message_index].add_content(&content);
                                window
                                    .emit("add_message_content", content.to_owned())
                                    .unwrap();
                            }
                        }
                        Err(error) => match error {
                            gpt::StreamError::StreamReadFailed(_) => break,
                            gpt::StreamError::InvalidJson(_) => break,
                            gpt::StreamError::InvalidEvent => break,
                        },
                    }
                }

                is_locked.store(false, Ordering::SeqCst);
                window.emit("lock", false).unwrap();
            });
        }

        Ok(())
    }
}
