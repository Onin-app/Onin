use std::error::Error;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use futures::stream::BoxStream;

/// Represents the request payload for a chat completion
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ChatRequest {
    pub model: String,
    pub messages: Vec<ChatMessage>,
    pub temperature: Option<f32>,
    pub max_tokens: Option<u32>,
    pub stream: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

/// Trait that all AI providers must implement
#[async_trait]
pub trait AIProvider: Send + Sync {
    /// Unique identifier for the provider implementation (e.g., "openai_compatible", "ollama")
    fn id(&self) -> &str;

    /// Send a chat request and get a comprehensive response
    async fn ask(&self, request: ChatRequest) -> Result<String, Box<dyn Error + Send + Sync>>;

    /// Send a chat request and get a stream of response chunks
    async fn stream(
        &self,
        request: ChatRequest,
    ) -> Result<BoxStream<'static, Result<String, Box<dyn Error + Send + Sync>>>, Box<dyn Error + Send + Sync>>;
}
