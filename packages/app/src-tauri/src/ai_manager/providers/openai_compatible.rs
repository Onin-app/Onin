use crate::ai_manager::provider::{AIProvider, ChatRequest, ValidationResult, ModelInfo, ProviderCapabilities};

use async_trait::async_trait;
use futures::{stream, StreamExt};
use reqwest::Client;
use serde::{Deserialize};
use std::error::Error;
use futures::stream::BoxStream;

pub struct OpenAICompatibleProvider {
    base_url: String,
    api_key: Option<String>,
    client: Client,
}

impl OpenAICompatibleProvider {
    pub fn new(base_url: String, api_key: Option<String>) -> Self {
        Self {
            base_url,
            api_key,
            client: Client::new(),
        }
    }
}

#[derive(Debug, Deserialize)]
struct OpenAIResponse {
    choices: Vec<OpenAIChoice>,
}

#[derive(Debug, Deserialize)]
struct OpenAIChoice {
    message: OpenAIMessage,
}

#[derive(Debug, Deserialize)]
struct OpenAIMessage {
    content: Option<String>,
}

#[derive(Debug, Deserialize)]
struct OpenAIStreamResponse {
    choices: Vec<OpenAIStreamChoice>,
}

#[derive(Debug, Deserialize)]
struct OpenAIStreamChoice {
    delta: OpenAIStreamDelta,
}

#[derive(Debug, Deserialize)]
struct OpenAIStreamDelta {
    content: Option<String>,
}

#[async_trait]
impl AIProvider for OpenAICompatibleProvider {
    fn id(&self) -> &str {
        "openai_compatible"
    }

    async fn ask(&self, request: ChatRequest) -> Result<String, Box<dyn Error + Send + Sync>> {
        let url = format!("{}/chat/completions", self.base_url.trim_end_matches('/'));
        
        // request already has the correct structure (MessageContent is untagged enum)
        // so it serializes to either "content": "string" or "content": [...]
        
        let mut builder = self.client.post(&url).json(&request);
        
        if let Some(key) = &self.api_key {
            builder = builder.header("Authorization", format!("Bearer {}", key));
        }

        let resp = builder.send().await?;
        
        if !resp.status().is_success() {
            let error_text = resp.text().await?;
            return Err(format!("API Error: {}", error_text).into());
        }

        let data: OpenAIResponse = resp.json().await?;
        
        if let Some(choice) = data.choices.first() {
            Ok(choice.message.content.clone().unwrap_or_default())
        } else {
            Ok("".to_string())
        }
    }

    async fn stream(
        &self,
        mut request: ChatRequest,
    ) -> Result<BoxStream<'static, Result<String, Box<dyn Error + Send + Sync>>>, Box<dyn Error + Send + Sync>> {
        request.stream = Some(true);
        let url = format!("{}/chat/completions", self.base_url.trim_end_matches('/'));
        
        let mut builder = self.client.post(&url).json(&request);
        
        if let Some(key) = &self.api_key {
            builder = builder.header("Authorization", format!("Bearer {}", key));
        }

        let resp = builder.send().await?;
        
        if !resp.status().is_success() {
            let error_text = resp.text().await?;
            return Err(format!("API Error: {}", error_text).into());
        }

        let stream = resp.bytes_stream();
        
        let output_stream = stream.flat_map(move |chunk_result| {
            match chunk_result {
                Ok(bytes) => {
                    let text = String::from_utf8_lossy(&bytes).to_string();
                    let mut chunks = Vec::new();
                    
                    for line in text.lines() {
                        let line = line.trim();
                        if line.is_empty() { continue; }
                        if line == "data: [DONE]" { continue; }
                        
                        if line.starts_with("data: ") {
                            let json_str = &line[6..];
                            if let Ok(data) = serde_json::from_str::<OpenAIStreamResponse>(json_str) {
                                if let Some(choice) = data.choices.first() {
                                    if let Some(content) = &choice.delta.content {
                                        chunks.push(Ok(content.clone()));
                                    }
                                }
                            }
                        }
                    }
                    stream::iter(chunks)
                },
                Err(e) => stream::iter(vec![Err(Box::new(e) as Box<dyn Error + Send + Sync>)]),
            }
        });

        Ok(Box::pin(output_stream))
    }

    async fn validate(&self) -> Result<ValidationResult, Box<dyn Error + Send + Sync>> {
        // Try to list models to validate connection and API key
        match self.list_models().await {
            Ok(models) => Ok(ValidationResult {
                valid: true,
                message: Some(format!("Connection successful. Found {} models.", models.len())),
                models_count: Some(models.len()),
            }),
            Err(e) => Ok(ValidationResult {
                valid: false,
                message: Some(format!("Validation failed: {}", e)),
                models_count: None,
            }),
        }
    }

    async fn list_models(&self) -> Result<Vec<ModelInfo>, Box<dyn Error + Send + Sync>> {
        let url = format!("{}/models", self.base_url.trim_end_matches('/'));
        
        let mut builder = self.client.get(&url);
        if let Some(key) = &self.api_key {
            builder = builder.header("Authorization", format!("Bearer {}", key));
        }
        
        let resp = builder.send().await?;
        
        if !resp.status().is_success() {
            let error_text = resp.text().await?;
            return Err(format!("Failed to list models: {}", error_text).into());
        }
        
        #[derive(Deserialize)]
        struct ModelsResponse {
            data: Vec<ModelData>,
        }
        
        #[derive(Deserialize)]
        struct ModelData {
            id: String,
            #[serde(default)]
            owned_by: Option<String>,
        }
        
        let data: ModelsResponse = resp.json().await?;
        
        Ok(data.data.into_iter().map(|m| ModelInfo {
            id: m.id.clone(),
            name: m.id,
            description: m.owned_by,
            context_window: None,
        }).collect())
    }

    fn capabilities(&self) -> ProviderCapabilities {
        ProviderCapabilities {
            supports_images: true,  // OpenAI-compatible APIs support images
            supports_streaming: true,
            supports_function_calling: false,
            max_context_tokens: None,  // Varies by model
            max_images_per_message: Some(10),  // OpenAI limit
        }
    }
}
