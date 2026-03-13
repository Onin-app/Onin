use async_trait::async_trait;
use futures::stream::BoxStream;
use serde::{Deserialize, Serialize};
use std::error::Error;

/// Represents the request payload for a chat completion
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ChatRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    pub messages: Vec<ChatMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ChatMessage {
    pub role: String,
    /// Content is always an array of content parts (text, images, etc.)
    /// This matches the OpenAI API format and TypeScript SDK
    pub content: Vec<ContentPart>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type")]
pub enum ContentPart {
    #[serde(rename = "text")]
    Text { text: String },

    #[serde(rename = "image_url")]
    ImageUrl { image_url: ImageUrl },

    #[serde(rename = "image_base64")]
    ImageBase64 {
        image_base64: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        media_type: Option<String>,
    },
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ImageUrl {
    pub url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ValidationResult {
    pub valid: bool,
    pub message: Option<String>,
    pub models_count: Option<usize>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ModelInfo {
    pub id: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context_window: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProviderCapabilities {
    /// Whether the provider supports image inputs (both URL and base64)
    pub supports_images: bool,
    /// Whether the provider supports streaming responses
    pub supports_streaming: bool,
    /// Whether the provider supports function calling
    pub supports_function_calling: bool,
    /// Maximum context window in tokens
    pub max_context_tokens: Option<u32>,
    /// Maximum number of images allowed per message (None = unlimited)
    pub max_images_per_message: Option<u32>,
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
    ) -> Result<
        BoxStream<'static, Result<String, Box<dyn Error + Send + Sync>>>,
        Box<dyn Error + Send + Sync>,
    >;

    /// Validate the provider configuration (check API key, connectivity)
    async fn validate(&self) -> Result<ValidationResult, Box<dyn Error + Send + Sync>>;

    /// List available models from the provider
    async fn list_models(&self) -> Result<Vec<ModelInfo>, Box<dyn Error + Send + Sync>>;

    /// Get static capabilities of this provider
    fn capabilities(&self) -> ProviderCapabilities;
}
