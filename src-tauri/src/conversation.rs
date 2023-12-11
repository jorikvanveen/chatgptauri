use super::gpt;
use crate::gpt::{MessageDelta, Request, Role};
use crate::settings::Model;
use anyhow::{Context, Result};
use directories::BaseDirs;
use gpt::Message;
use rand::prelude::*;
use reqwest_eventsource::CannotCloneRequestError;
use serde::{Deserialize, Serialize};
use std::time::{self, Duration};
use std::{
    path::PathBuf,
    sync::{
        atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering},
        Arc,
    },
};
use thiserror::Error;
use tokio::fs;
use tokio::sync::Mutex;
use tokio::time::timeout;
use tokio_stream::StreamExt;

#[derive(Error, Debug)]
pub enum PromptError {
    #[error("There is already a request in progress")]
    ConversationLocked,

    #[error("Something went wrong while making the API request")]
    RequestError(#[from] CannotCloneRequestError),
}

#[derive(Clone)]
pub struct Conversation {
    is_locked: Arc<AtomicBool>, // Conversation will be locked while the server is streaming a response
    // to us.
    messages: Arc<Mutex<Vec<Message>>>,
    name: Arc<Mutex<Option<String>>>,
    id: Arc<AtomicU32>,
    date_created: Arc<AtomicU64>,
}

#[derive(Clone)]
pub struct CancelState(Arc<AtomicBool>);

impl CancelState {
    pub fn new() -> Self {
        Self(AtomicBool::new(false).into())
    }

    pub fn transmit_cancel(&self) {
        self.0.store(true, Ordering::SeqCst);
    }

    /// When this returns true, it assumed that the caller handles the signal. And the internal
    /// state will be set to true.
    pub fn receive_cancel(&self) -> bool {
        self.0.swap(false, Ordering::SeqCst)
    }
}

impl Conversation {
    pub fn new() -> Self {
        Self {
            is_locked: Arc::new(AtomicBool::new(false)),
            messages: Arc::new(Mutex::new(vec![])),
            name: Arc::new(Mutex::new(None)),
            id: Arc::new(AtomicU32::new(thread_rng().gen())),
            date_created: Arc::new(AtomicU64::new(
                time::SystemTime::now()
                    .duration_since(time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
            )),
        }
    }

    pub async fn reset(&self) {
        self.is_locked.store(false, Ordering::SeqCst);
        *self.messages.lock().await = vec![];
        *self.name.lock().await = None;
        self.id.store(thread_rng().gen(), Ordering::Relaxed);
        self.date_created.store(
            time::SystemTime::now()
                .duration_since(time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            Ordering::Relaxed,
        );
    }

    pub fn get_messages(&self) -> &Arc<Mutex<Vec<Message>>> {
        &self.messages
    }

    pub fn get_id(&self) -> u32 {
        return self.id.load(Ordering::Relaxed);
    }

    pub async fn get_word_count(&self) -> usize {
        let messages = self.messages.lock().await;
        messages.iter().flat_map(|message| message.get_content().split(" ")).count()
    }

    pub async fn get_token_count(&self) -> usize {
        // TODO: Make this count actual tokens instead of an estimate
        // https://openai.com/pricing
        self.get_word_count().await * 1000 / 750
    }

    /// Clears all the messages in this conversation
    ///
    /// # Errors
    ///
    /// This function will return an error if the conversation is locked.
    pub async fn clear(&self) -> Result<(), PromptError> {
        if self.is_locked.load(Ordering::SeqCst) {
            return Err(PromptError::ConversationLocked);
        }

        *self.messages.lock().await = vec![];
        Ok(())
    }

    /// Converts this conversation into a `SerializedConversation` that can be converted to JSON.
    ///
    /// # Errors
    ///
    /// This function will return an error if `self.get_name()` fails.
    pub async fn serialize(&self, api_key: &str) -> Result<SerializedConversation> {
        Ok(SerializedConversation {
            name: self.get_name(api_key).await?,
            id: self.id.load(Ordering::Relaxed),
            messages: self.messages.lock().await.clone(),
            date_created: self.date_created.load(Ordering::Relaxed),
        })
    }

    /// Serializes this conversation and saves it in the data directory.
    ///
    /// # Errors
    ///
    /// This function will return an error if a name cannot be generated, or if the save directory
    /// cannot be acquired.
    pub async fn save(&self, api_key: &str) -> Result<()> {
        *self.name.lock().await = Some(self.generate_name(api_key).await?);
        let filename = self.id.load(Ordering::Relaxed).to_string();

        let serialized_conversation = self.serialize(api_key).await?;
        let file_contents = serde_json::to_string(&serialized_conversation)?;

        let mut path = Self::get_save_dir().await?;
        path.push(filename);

        fs::write(path, file_contents.as_bytes()).await?;
        Ok(())
    }

    pub async fn load(&self, id: u64) -> Result<()> {
        let loaded_conversation = Self::load_serialized(id).await?;
        self.id.store(loaded_conversation.id, Ordering::Relaxed);
        *self.messages.lock().await = loaded_conversation.messages;
        *self.name.lock().await = Some(loaded_conversation.name);
        Ok(())
    }

    pub async fn load_serialized(id: u64) -> Result<SerializedConversation> {
        let mut file_path = Self::get_save_dir().await?;
        file_path.push(id.to_string());
        let file_path = file_path;

        dbg!(&file_path);

        let serialized = fs::read_to_string(file_path).await?;
        let deserialized: SerializedConversation = serde_json::from_str(&serialized)?;

        Ok(deserialized)
    }

    pub async fn generate_name(&self, api_key: &str) -> Result<String> {
        let mut cloned_messages = self.messages.lock().await.clone();
        cloned_messages.push(Message::new(Role::user, "Write a name for this conversation, it should not be longer than a few words. Do not mention math if the user doesn't. Do not say anything except the name, do not put it in quotes and do not use a period.".into()));

        let mut stream = Request::new(cloned_messages, "gpt-3.5-turbo")
            .do_request(api_key)
            .context("Failed to make api request while generating name for conversation")?;

        let mut name = String::new();
        while let Some(Ok(delta)) = stream.next().await {
            match delta {
                MessageDelta::Delta(delta) => name.push_str(&delta),
                _ => continue,
            }
        }
        let name = name;
        Ok(name)
    }

    pub async fn get_name(&self, api_key: &str) -> Result<String> {
        let mut current_name = self.name.lock().await;

        if let Some(name) = &*current_name {
            return Ok(name.into());
        }

        let new_name = self.generate_name(api_key).await?;
        *current_name = Some(new_name.clone());
        Ok(new_name)
    }

    /// Returns the directory that conversations should be saved in. Automatically creates it if it
    /// does not exist.
    ///
    /// # Panics
    ///
    /// Panics if `BaseDirs::new()`` fails, in which case the users computer is scuffed as hell.
    ///
    /// # Errors
    ///
    /// This function will return an error if the directory does not exist and cannot be created.
    pub async fn get_save_dir() -> Result<PathBuf> {
        let base_dirs =
            BaseDirs::new().expect("Failed to get base dirs, your computer is weird af ngl");
        let mut data_dir = base_dirs.data_dir().to_path_buf();
        data_dir.push("chatgptauri/");
        data_dir.push("conversations/");
        let data_dir = data_dir;

        if !data_dir.exists() {
            fs::create_dir_all(&data_dir).await?;
        }

        Ok(data_dir)
    }

    pub async fn list_conversations() -> Result<Vec<SerializedConversation>> {
        let mut conversations: Vec<SerializedConversation> = vec![];
        for id in Self::get_conversation_ids().await? {
            let conversation = match Self::load_serialized(id).await {
                Ok(conversation) => conversation,
                Err(_) => continue,
            };

            conversations.push(conversation);
        }
        conversations.sort_by(|a, b| b.date_created.cmp(&a.date_created));
        let conversations = conversations;

        Ok(conversations)
    }

    pub async fn get_conversation_ids() -> Result<Vec<u64>> {
        let save_dir = Self::get_save_dir().await?;
        let mut files = fs::read_dir(&save_dir).await?;

        let mut ids: Vec<u64> = vec![];
        while let Some(file) = files.next_entry().await? {
            let id: u64 = match u64::from_str_radix(file.file_name().to_str().unwrap(), 10) {
                Ok(id) => id,
                Err(_) => continue,
            };

            ids.push(id);
        }
        let ids = ids;

        Ok(ids)
    }

    /// Submits a prompt to the conversation, requests a completion from the OpenAI api and spawns
    /// a task that streams
    /// the response. Every time a chunk is received, the `add_message_content` event is fired on
    /// the window.
    ///
    /// # Errors
    ///
    /// This function will return an error if the request cannot be made.
    pub async fn prompt(
        &self,
        prompt: &str,
        api_key: &str,
        model: Model,
        window: &tauri::Window,
        cancel_state: CancelState
    ) -> Result<()> {
        // Check if conversation is locked
        if self.is_locked.load(Ordering::SeqCst) {
            return Err(PromptError::ConversationLocked.into());
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
            let mut delta_stream =
                gpt::Request::new(self.messages.lock().await.clone(), model.to_string()).do_request(api_key)?;
            let window = window.clone();
            let api_key = api_key.to_string();
            let conversation = self.clone();
            let input_token_count = self.get_token_count().await;

            tokio::spawn(async move {
                is_locked.store(true, Ordering::SeqCst);
                window.emit("lock", true).unwrap();

                let mut output = String::new();

                // Await next message with a timeout of 5 seconds.
                println!("Waiting for next thing in stream");
                loop {
                    let content = match timeout(Duration::from_secs(5), delta_stream.next()).await {
                        Ok(delta) => match delta {
                            Some(delta) => match delta {
                                Ok(delta) => match delta {
                                    MessageDelta::Delta(delta) => {
                                        if cancel_state.receive_cancel() {
                                            break;
                                        }
                                        delta
                                    }, // We actually got some message content
                                    MessageDelta::Role(_) => continue,
                                    MessageDelta::NoData => continue,
                                    MessageDelta::Done => {
                                        println!("Got done message");
                                        break;
                                    },
                                },
                                Err(err) => match err {
                                    gpt::StreamError::StreamReadFailed(err) => {
                                        eprintln!("Failed to read from stream");
                                        eprintln!("{err}");
                                        break;
                                    },
                                    gpt::StreamError::InvalidJson(err) => {
                                        eprintln!("Got invalid json from API");
                                        eprintln!("{err}");
                                        break;
                                    },
                                    gpt::StreamError::InvalidEvent => {
                                        eprintln!("Got unknown/invalid event from API");
                                        break;
                                    },
                                },
                            },
                            None => {
                                eprintln!("Stream returned None, assuming it ended");
                                break;
                            },
                        },
                        Err(_) => {
                            println!("OpenAI API took too long to respond");
                            break;
                        },
                    };
                    println!("Got thing in stream");

                    println!("Locking messages");
                    let mut messages = messages.lock().await;
                    messages.last_mut().unwrap().add_content(&content);
                    output += &content;

                    window
                        .emit("add_message_content", content.to_owned())
                        .unwrap();
                }

                println!("Stream ended");

                let cost = model.calculate_cost(input_token_count, Self::count_tokens(&output));

                {
                    let mut messages = messages.lock().await;
                    messages.last_mut().unwrap().set_cost(cost);
                }

                let _ = conversation.save(&api_key).await;
                is_locked.store(false, Ordering::SeqCst);
                window.emit("lock", false).unwrap();
                println!("Got cost: {}", cost);
                window.emit("cost", cost).unwrap(); // Send the cost to the client
            });
        }

        Ok(())
    }

    fn count_tokens(string: &str) -> usize {
        string.split(' ').count() * 1000 / 750
    }
}

#[derive(Serialize, Deserialize)]
pub struct SerializedConversation {
    name: String,
    id: u32,
    date_created: u64,
    messages: Vec<Message>,
}
