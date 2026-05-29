use self::config::AIConfig;
use self::provider::{AIProvider, ChatRequest};
use self::providers::openai_compatible::OpenAICompatibleProvider;

use std::error::Error;
use std::path::PathBuf;
use std::sync::Arc;
use tauri::{AppHandle, Manager};
use tokio::sync::RwLock;

pub mod commands;
pub mod config;
pub mod history;
pub mod provider;
pub mod providers;

use futures::stream::BoxStream;

/// Manages AI providers and configuration
pub struct AIManager {
    config: RwLock<AIConfig>,
    active_provider: RwLock<Option<Arc<dyn AIProvider>>>,
    app_handle: AppHandle,
    history_manager: std::sync::Mutex<self::history::HistoryManager>,
    app_data_dir: PathBuf,
}

impl AIManager {
    pub fn new(app_handle: AppHandle) -> Self {
        let data_dir = app_handle
            .path()
            .app_data_dir()
            .unwrap_or_else(|_| std::env::current_dir().unwrap_or_default());
        let history_manager = self::history::HistoryManager::new(data_dir.clone());

        Self {
            config: RwLock::new(AIConfig::default()),
            active_provider: RwLock::new(None),
            app_handle,
            history_manager: std::sync::Mutex::new(history_manager),
            app_data_dir: data_dir,
        }
    }

    /// Get AI config file path
    fn get_config_path(&self) -> PathBuf {
        self.app_data_dir.join("ai_config.json")
    }

    /// Load configuration from file
    pub async fn load_config(&self) -> Result<AIConfig, String> {
        let config_path = self.get_config_path();

        if !config_path.exists() {
            // If config file doesn't exist, return default config
            return Ok(AIConfig::default());
        }

        let content = std::fs::read_to_string(&config_path)
            .map_err(|e| format!("Failed to read AI config file: {}", e))?;

        let config: AIConfig = serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse AI config: {}", e))?;

        Ok(config)
    }

    /// Save configuration to file
    fn save_config(&self, config: &AIConfig) -> Result<(), String> {
        let config_path = self.get_config_path();

        let content = serde_json::to_string_pretty(config)
            .map_err(|e| format!("Failed to serialize AI config: {}", e))?;

        std::fs::write(&config_path, content)
            .map_err(|e| format!("Failed to write AI config file: {}", e))?;

        Ok(())
    }

    /// Update configuration and re-initialize the active provider
    pub async fn update_config(
        &self,
        new_config: AIConfig,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        let mut config = self.config.write().await;
        *config = new_config.clone();

        let mut active_provider = self.active_provider.write().await;

        if let Some(provider_id) = &new_config.active_provider_id {
            if let Some(provider_config) =
                new_config.providers.iter().find(|p| &p.id == provider_id)
            {
                // Initialize the provider based on configuration
                // Currently only supports OpenAI Compatible generic provider type
                // In the future, we can add more types like "ollama_native" if needed
                let provider = OpenAICompatibleProvider::new(
                    provider_config.base_url.clone(),
                    provider_config.api_key.clone(),
                );
                *active_provider = Some(Arc::new(provider));
            } else {
                return Err(format!(
                    "Provider with ID {} not found in configuration",
                    provider_id
                )
                .into());
            }
        } else {
            *active_provider = None;
        }

        // Save to file
        self.save_config(&new_config).map_err(|e| e.to_string())?;

        Ok(())
    }

    pub async fn get_config(&self) -> AIConfig {
        self.config.read().await.clone()
    }

    pub async fn ask(
        &self,
        mut request: ChatRequest,
    ) -> Result<String, Box<dyn Error + Send + Sync>> {
        let config = self.config.read().await;
        let provider_lock = self.active_provider.read().await;

        if let Some(provider) = provider_lock.as_ref() {
            // If no model specified, use the default model from config
            if request.model.is_none() {
                if let Some(provider_id) = &config.active_provider_id {
                    if let Some(provider_config) =
                        config.providers.iter().find(|p| &p.id == provider_id)
                    {
                        request.model = provider_config.default_model.clone();
                    }
                }
            }

            // If still no model, return error
            if request.model.is_none() {
                return Err("No model specified and no default model configured".into());
            }

            provider.ask(request).await
        } else {
            Err("No active AI provider configured".into())
        }
    }

    pub async fn stream(
        &self,
        mut request: ChatRequest,
    ) -> Result<
        BoxStream<'static, Result<String, Box<dyn Error + Send + Sync>>>,
        Box<dyn Error + Send + Sync>,
    > {
        let config = self.config.read().await;
        let provider_lock = self.active_provider.read().await;

        if let Some(provider) = provider_lock.as_ref() {
            // If no model specified, use the default model from config
            if request.model.is_none() {
                if let Some(provider_id) = &config.active_provider_id {
                    if let Some(provider_config) =
                        config.providers.iter().find(|p| &p.id == provider_id)
                    {
                        request.model = provider_config.default_model.clone();
                    }
                }
            }

            // If still no model, return error
            if request.model.is_none() {
                return Err("No model specified and no default model configured".into());
            }

            provider.stream(request).await
        } else {
            Err("No active AI provider configured".into())
        }
    }

    pub async fn list_models(
        &self,
    ) -> Result<Vec<self::provider::ModelInfo>, Box<dyn Error + Send + Sync>> {
        let provider_lock = self.active_provider.read().await;
        if let Some(provider) = provider_lock.as_ref() {
            provider.list_models().await
        } else {
            Err("No active AI provider configured".into())
        }
    }

    pub async fn get_capabilities(&self) -> Option<self::provider::ProviderCapabilities> {
        let provider_lock = self.active_provider.read().await;
        provider_lock.as_ref().map(|p| p.capabilities())
    }

    pub fn load_index(&self) -> Result<Vec<self::history::ChatSessionMeta>, String> {
        let history = self.history_manager.lock().map_err(|e| e.to_string())?;
        history.load_index()
    }

    pub fn get_session(&self, id: &str) -> Result<self::history::ChatSession, String> {
        let history = self.history_manager.lock().map_err(|e| e.to_string())?;
        history.get_session(id)
    }

    pub fn save_session(&self, session: self::history::ChatSession) -> Result<(), String> {
        let history = self.history_manager.lock().map_err(|e| e.to_string())?;
        history.save_session(session)
    }

    pub fn delete_session(&self, id: &str) -> Result<(), String> {
        let history = self.history_manager.lock().map_err(|e| e.to_string())?;
        history.delete_session(id)
    }

    pub fn clear_all_sessions(&self) -> Result<(), String> {
        let history = self.history_manager.lock().map_err(|e| e.to_string())?;
        history.clear_all_sessions()
    }
}
