use std::{task::Poll, pin::Pin};
use futures_core::{Stream, Future};
use serde::{Serialize, Deserialize};
use thiserror::Error;
use reqwest_eventsource::{self as reqwest_es, EventSource, CannotCloneRequestError};
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
    content: String
}

impl Message {
    pub fn add_content(&mut self, content: &str) {
        self.content.push_str(content);
    }
}

impl Message {
    pub fn new(role: Role, content: String) -> Self { Self { role, content } }

    pub fn get_content(&self) -> &str {
        &self.content
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Request {
    pub model: String,
    pub messages: Vec<Message>,
    pub stream: bool
}

impl Request {
    pub fn new(messages: Vec<Message>, model: &str) -> Self {
        let mut messages = messages;
        messages.insert(0, Message::new(Role::system, "Anything to do with math in your responses must be formatted in TeX surrounded by either one or two dollar signs ($)".to_string()));

        Self {
            model: model.to_string(),
            messages,
            stream: true
        }
    }

    pub fn do_request(self, api_key: &str) -> Result<MessageDeltaStream, CannotCloneRequestError> {
        let client = reqwest::Client::new();

        let source = EventSource::new(
            client.post("https://api.openai.com/v1/chat/completions")
                .header("Authorization", format!("Bearer {}", api_key))
                .json(&self)
        )?;

        return Ok(MessageDeltaStream::new(source));
    }
}

#[derive(Debug, Error)]
pub enum StreamError {
    #[error("Error while reading response stream")]
    StreamReadFailed(#[from] reqwest_eventsource::Error),

    #[error("Invalid response from openai api")]
    InvalidJson(#[from] serde_json::Error),

    #[error("Invalid response from openai api")]
    InvalidEvent
}

pub struct MessageDeltaStream {
    event_source: EventSource,
}

impl Stream for MessageDeltaStream {
    type Item = Result<MessageDelta, StreamError>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> Poll<Option<Self::Item>> {
        let next = self.event_source.next();

        // Poll next event
        return match Pin::new(&mut Box::pin(next)).poll(cx) {
            Poll::Ready(x) => match x {
                Some(x) => match x {
                    Ok(event) => match event {
                        reqwest_es::Event::Open => {
                            cx.waker().wake_by_ref();
                            Poll::Pending
                        },
                        reqwest_es::Event::Message(message) => {
                            Poll::Ready(Some(MessageDeltaStream::process_event(message)))
                        },
                    },  
                    Err(error) => {
                        Poll::Ready(Some(Err(error.into())))
                    },
                },
                None => {
                    Poll::Ready(None)
                },
           },
           Poll::Pending => {
               cx.waker().wake_by_ref();
               Poll::Pending
           },
        };
    }
}

impl MessageDeltaStream {
    pub fn new(event_source: EventSource) -> Self {
        Self {
            event_source,
        }
    }

    fn process_event(event: eventsource_stream::Event) -> Result<MessageDelta, StreamError> {
        let data = event.data;

        if data == "[DONE]" {
            return Ok(MessageDelta::Done);
        }

        dbg!(&data);

        // Parse data
        let data: openai_types::EventData = serde_json::from_str(&data)?;

        dbg!(&data);

        if data.choices.len() == 0 {
            return Err(StreamError::InvalidEvent)
        }

        match &data.choices[0].delta {
            openai_types::Delta::Role { role } => Ok(MessageDelta::Role(role.clone())),
            openai_types::Delta::Content { content } => Ok(MessageDelta::Delta(content.into())),
            openai_types::Delta::NoData {} => Ok(MessageDelta::NoData)
        }
    }
}

mod openai_types {
    use serde::Deserialize;

    #[derive(Debug, Deserialize)]
    pub struct EventData {
        pub object: String,
        pub created: isize,
        pub model: String,
        pub choices: Vec<Choice>
    }

    #[derive(Debug, Deserialize)]
    pub struct Choice {
        pub finish_reason: Option<String>,
        pub index: isize,
        pub delta: Delta
    }

    #[derive(Debug, Deserialize)]
    #[serde(untagged)]
    pub enum Delta {
        Role { role: super::Role },
        Content { content: String },
        NoData {}
    }
}

#[derive(Debug)]
pub enum MessageDelta {
    Delta(String),
    Role(Role),
    NoData,
    Done
}

