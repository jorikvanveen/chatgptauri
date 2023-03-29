use serde::{Serialize, Deserialize};
use reqwest;
use thiserror::Error;
use serde_json::from_str;

#[derive(Serialize, Deserialize, Debug)]
pub enum Role {
    User,
    System,
    Assistant
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "role", content = "content")]
#[allow(non_camel_case_types)]
pub enum Message {
    system(String),
    user(String),
    assistant(String)
}

impl Message {
    pub fn get_content(&self) -> &str {
        match self {
            Message::system(s) => &s,
            Message::user(s) => &s,
            Message::assistant(s) => &s,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Request {
    pub model: &'static str,
    pub messages: Vec<Message>
}

#[derive(Debug, Error)]
pub enum RequestError {
    #[error("Something went wrong making the HTTP request")]
    HTTPError(#[from] reqwest::Error),

    #[error("The OpenAI API threw an error")]
    OpenAIError(String),

    #[error("Failed to parse openai response body")]
    BodyParse
}

impl RequestError {
    pub fn to_string(self) -> String {
        match self {
            Self::HTTPError(e) => e.to_string(),
            Self::OpenAIError(s) => s,
            Self::BodyParse => self.to_string()
        }
    }
}

#[derive(Deserialize, Debug)]
pub struct OpenAIAPIError {
    error: OpenAIAPIErrorInfo
}

#[derive(Deserialize, Debug)]
pub struct OpenAIAPIErrorInfo {
    message: String,
}

impl Request {
    pub fn new(messages: Vec<Message>) -> Self {
        Self {
            model: "gpt-4",
            messages
        }
    }

    pub async fn do_request(self, api_key: &str) -> Result<Response, RequestError> {
        let client = reqwest::Client::new();

        let response = client.post("https://api.openai.com/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", api_key))
            .json(&self)
            .send()
            .await?;

        let response_text = response.text().await?;

        if let Ok(response) = from_str::<Response>(&response_text) {
            return Ok(response)
        };

        if let Ok(error) = from_str::<OpenAIAPIError>(&response_text) {
            return Err(RequestError::OpenAIError(error.error.message))
        }

        Err(RequestError::BodyParse)
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Response {
    pub id: String,
    pub object: String,
    pub created: i64,
    pub model: String,
    pub usage: Usage,
    pub choices: Vec<Choice>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Usage {
    pub prompt_tokens: i32,
    pub completion_tokens: i32,
    pub total_tokens: i32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Choice {
    pub message: Message,
    pub finish_reason: String,
    pub index: i32
}
