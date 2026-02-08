use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct AIConfig {
    pub active_provider_id: Option<String>,
    pub providers: Vec<ProviderConfig>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProviderConfig {
    pub id: String,              // Unique instance ID (e.g., "openai_1234567_abc123")
    pub provider_type: String,   // Template type (e.g., "openai", "deepseek")
    pub name: String,            // Display name
    pub base_url: String,        // API endpoint
    pub api_key: Option<String>, // API Key (optional for local models)
    pub default_model: Option<String>, // Default model for this provider
}
