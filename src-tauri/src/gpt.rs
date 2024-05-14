use futures_core::Stream;
use reqwest_eventsource::{self as reqwest_es, CannotCloneRequestError, EventSource};
use serde::{Deserialize, Serialize};
use std::pin::Pin;
use thiserror::Error;
use tokio_stream::StreamExt;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[allow(non_camel_case_types)]
pub enum Role {
    user,
    system,
    assistant,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Message {
    role: Role,
    content: String,
    cost_dollars: Option<f32>,
}

// This is the type that will be sent to the API
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ApiMessage {
    role: Role,
    content: String,
}

impl Into<ApiMessage> for Message {
    fn into(self) -> ApiMessage {
        ApiMessage {
            role: self.role,
            content: self.content,
        }
    }
}

impl Message {
    pub fn add_content(&mut self, content: &str) {
        self.content.push_str(content);
    }
}

impl Message {
    pub fn new(role: Role, content: String) -> Self {
        Self {
            role,
            content,
            cost_dollars: None,
        }
    }

    pub fn set_cost(&mut self, cost: f32) {
        self.cost_dollars = Some(cost);
    }

    pub fn get_content(&self) -> &str {
        &self.content
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Request {
    pub model: String,
    pub messages: Vec<ApiMessage>,
    pub stream: bool,
}

impl Request {
    pub fn new(messages: Vec<Message>, model: &str) -> Self {
        let mut messages = messages;
        messages.insert(0, Message::new(Role::system, "You are about to enter a conversation with a user, they may or may not ask you questions about math. If you are trying to express a formula or variable or any other math concept that can be expressed in LaTeX, please do so. You can create an inline LaTeX block with a single dollar sign, for example: $a$. If you want to create a block that is centered, please use double dollar signs: $$a$$. If your output happens to contain a dollar sign, but you do not want the dollar sign to be interpreted as the start of a LaTeX block, please escape it using a backslash like this: \\$".to_string()));

        Self {
            model: model.to_string(),
            messages: messages.into_iter().map(|message| message.into()).collect(),
            stream: true,
        }
    }

    pub fn do_request(
        self,
        api_key: &str,
    ) -> Result<
        Pin<Box<impl Stream<Item = Result<MessageDelta, StreamError>>>>,
        CannotCloneRequestError,
    > {
        let client = reqwest::Client::new();

        dbg!(&self);

        let source = EventSource::new(
            client
                .post("https://api.openai.com/v1/chat/completions")
                .header("Authorization", format!("Bearer {}", api_key))
                .json(&self),
        )?;

        Ok(Box::pin(source.then(Request::handle_eventsource_event)))
    }

    async fn handle_eventsource_event(
        event: Result<reqwest_eventsource::Event, reqwest_eventsource::Error>,
    ) -> Result<MessageDelta, StreamError> {
        match event {
            Ok(event) => match event {
                reqwest_es::Event::Open => Ok(MessageDelta::NoData),
                reqwest_es::Event::Message(event) => Self::process_event(event),
            },
            Err(e) => Err(e.into()),
        }
    }

    fn process_event(event: eventsource_stream::Event) -> Result<MessageDelta, StreamError> {
        let data = event.data;

        if data == "[DONE]" {
            return Ok(MessageDelta::Done);
        }

        // Parse data
        let data: openai_types::EventData = serde_json::from_str(&data)?;

        if data.choices.len() == 0 {
            return Err(StreamError::InvalidEvent);
        }

        match &data.choices[0].delta {
            openai_types::Delta::Role { role } => Ok(MessageDelta::Role(role.clone())),
            openai_types::Delta::Content { content } => Ok(MessageDelta::Delta(content.into())),
            openai_types::Delta::NoData {} => Ok(MessageDelta::NoData),
        }
    }
}

#[derive(Debug, Error)]
pub enum StreamError {
    #[error("Error while reading response stream")]
    StreamReadFailed(#[from] reqwest_eventsource::Error),

    #[error("Invalid response from openai api")]
    InvalidJson(#[from] serde_json::Error),

    #[error("Invalid response from openai api")]
    InvalidEvent,
}

mod openai_types {
    use serde::Deserialize;

    #[derive(Debug, Deserialize)]
    pub struct EventData {
        pub object: String,
        pub created: isize,
        pub model: String,
        pub choices: Vec<Choice>,
    }

    #[derive(Debug, Deserialize)]
    pub struct Choice {
        pub finish_reason: Option<String>,
        pub index: isize,
        pub delta: Delta,
    }

    #[derive(Debug, Deserialize)]
    #[serde(untagged)]
    pub enum Delta {
        Role { role: super::Role },
        Content { content: String },
        NoData {},
    }
}

#[derive(Debug, Clone, Serialize)]
pub enum MessageDelta {
    Delta(String),
    Role(Role),
    NoData,
    Done,
}
