use super::config::AIConfig;
use tauri::{AppHandle, command, State, Emitter};
use crate::ai_manager::AIManager;
use crate::ai_manager::provider::ChatRequest;
use std::sync::Arc;

#[command]
pub async fn get_ai_config(
    ai_manager: State<'_, Arc<AIManager>>,
) -> Result<AIConfig, String> {
    Ok(ai_manager.get_config().await)
}

#[command]
pub async fn update_ai_config(
    ai_manager: State<'_, Arc<AIManager>>,
    config: AIConfig,
) -> Result<(), String> {
    ai_manager.update_config(config).await.map_err(|e| e.to_string())
}

#[command]
pub async fn plugin_ai_ask(
    ai_manager: State<'_, Arc<AIManager>>,
    request: ChatRequest,
) -> Result<String, String> {
    ai_manager.ask(request).await.map_err(|e| e.to_string())
}

#[command]
pub async fn plugin_ai_stream(
    app: AppHandle,
    ai_manager: State<'_, Arc<AIManager>>,
    request: ChatRequest,
    event_id: String,
) -> Result<(), String> {
    let mut stream = ai_manager.stream(request).await.map_err(|e| e.to_string())?;
    
    use futures::StreamExt;
    
    // Spawn a task to handle the stream so we don't block the command handler
    // We can't return the stream directly from a command easily in Tauri v1/v2 without specialized plugins or valid return types
    // So we emit events.
    tauri::async_runtime::spawn(async move {
        while let Some(chunk_result) = stream.next().await {
            match chunk_result {
                Ok(content) => {
                    let _ = app.emit(&event_id, content);
                }
                Err(e) => {
                    let _ = app.emit(&format!("{}_error", event_id), e.to_string());
                    break;
                }
            }
        }
        let _ = app.emit(&format!("{}_done", event_id), ());
    });

    Ok(())
}

#[command]
pub async fn validate_ai_provider(
    base_url: String,
    api_key: Option<String>,
) -> Result<crate::ai_manager::provider::ValidationResult, String> {
    use crate::ai_manager::provider::AIProvider;
    let provider = crate::ai_manager::providers::openai_compatible::OpenAICompatibleProvider::new(base_url, api_key);
    provider.validate().await.map_err(|e| e.to_string())
}

#[command]
pub async fn list_ai_models(
    ai_manager: State<'_, Arc<AIManager>>,
) -> Result<Vec<crate::ai_manager::provider::ModelInfo>, String> {
    ai_manager.list_models().await.map_err(|e| e.to_string())
}

#[command]
pub async fn get_ai_capabilities(
    ai_manager: State<'_, Arc<AIManager>>,
) -> Result<Option<crate::ai_manager::provider::ProviderCapabilities>, String> {
    Ok(ai_manager.get_capabilities().await)
}
