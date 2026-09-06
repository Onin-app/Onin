use super::config::AIConfig;
use crate::ai_manager::history::{ChatSession, ChatSessionMeta};
use crate::ai_manager::provider::ChatRequest;
use crate::ai_manager::AIManager;
use std::sync::Arc;
use tauri::{command, AppHandle, Emitter, Manager, State};

#[command]
pub async fn get_ai_config(ai_manager: State<'_, Arc<AIManager>>) -> Result<AIConfig, String> {
    Ok(ai_manager.get_config().await)
}

#[command]
pub async fn update_ai_config(
    ai_manager: State<'_, Arc<AIManager>>,
    config: AIConfig,
) -> Result<(), String> {
    ai_manager
        .update_config(config)
        .await
        .map_err(|e| e.to_string())
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
    let mut stream = ai_manager
        .stream(request)
        .await
        .map_err(|e| e.to_string())?;

    use futures::StreamExt;

    let ai_manager_clone = ai_manager.inner().clone();
    let event_id_clone = event_id.clone();

    // Spawn a task to handle the stream so we don't block the command handler
    // We can't return the stream directly from a command easily in Tauri v1/v2 without specialized plugins or valid return types
    // So we emit events.
    let handle = tauri::async_runtime::spawn(async move {
        while let Some(chunk_result) = stream.next().await {
            match chunk_result {
                Ok(content) => {
                    let _ = app.emit(&event_id_clone, content);
                }
                Err(e) => {
                    let _ = app.emit(&format!("{}_error", event_id_clone), e.to_string());
                    break;
                }
            }
        }
        let _ = app.emit(&format!("{}_done", event_id_clone), ());

        // Clean up from active_streams
        let mut streams = ai_manager_clone.active_streams.lock().unwrap();
        streams.remove(&event_id_clone);
    });

    // Save JoinHandle to active_streams
    let mut streams = ai_manager.active_streams.lock().unwrap();
    streams.insert(event_id, handle);

    Ok(())
}

#[command]
pub async fn abort_ai_stream(
    ai_manager: State<'_, Arc<AIManager>>,
    event_id: String,
) -> Result<bool, String> {
    Ok(ai_manager.abort_stream(&event_id))
}

#[command]
pub async fn validate_ai_provider(
    base_url: String,
    api_key: Option<String>,
) -> Result<crate::ai_manager::provider::ValidationResult, String> {
    use crate::ai_manager::provider::AIProvider;
    let provider = crate::ai_manager::providers::openai_compatible::OpenAICompatibleProvider::new(
        base_url, api_key,
    );
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

#[command]
pub async fn get_ai_sessions_index(
    ai_manager: State<'_, Arc<AIManager>>,
) -> Result<Vec<ChatSessionMeta>, String> {
    ai_manager.load_index()
}

#[command]
pub async fn get_ai_session(
    ai_manager: State<'_, Arc<AIManager>>,
    id: String,
) -> Result<ChatSession, String> {
    ai_manager.get_session(&id)
}

#[command]
pub async fn save_ai_session(
    ai_manager: State<'_, Arc<AIManager>>,
    session: ChatSession,
) -> Result<(), String> {
    ai_manager.save_session(session)
}

#[command]
pub async fn delete_ai_session(
    ai_manager: State<'_, Arc<AIManager>>,
    id: String,
) -> Result<(), String> {
    ai_manager.delete_session(&id)
}

#[command]
pub async fn clear_all_ai_sessions(ai_manager: State<'_, Arc<AIManager>>) -> Result<(), String> {
    ai_manager.clear_all_sessions()
}

#[command]
pub async fn fetch_ai_models_direct(
    base_url: String,
    api_key: Option<String>,
) -> Result<Vec<crate::ai_manager::provider::ModelInfo>, String> {
    println!(
        "fetch_ai_models_direct: base_url = {:?}, api_key_is_some = {:?}, api_key_len = {:?}",
        base_url,
        api_key.is_some(),
        api_key.as_ref().map(|k| k.len())
    );
    use crate::ai_manager::provider::AIProvider;
    let provider = crate::ai_manager::providers::openai_compatible::OpenAICompatibleProvider::new(
        base_url, api_key,
    );
    provider.list_models().await.map_err(|e| e.to_string())
}

const MODELS_DEV_REGISTRY_URL: &str = "https://models.dev/api.json";

fn merge_registry_metadata(mut registry: serde_json::Value) -> serde_json::Value {
    if let Some(obj) = registry.as_object_mut() {
        let api_key_urls = [
            ("openai", "https://platform.openai.com/api_keys"),
            ("anthropic", "https://console.anthropic.com/settings/keys"),
            ("google", "https://aistudio.google.com/app/apikey"),
            ("xai", "https://console.x.ai"),
            ("x-ai", "https://console.x.ai"),
            ("deepseek", "https://platform.deepseek.com/api_keys"),
            ("zhipuai", "https://open.bigmodel.cn/usercenter/apikeys"),
            ("zhipu", "https://open.bigmodel.cn/usercenter/apikeys"),
            (
                "moonshotai",
                "https://platform.moonshot.cn/console/api-keys",
            ),
            ("moonshot", "https://platform.moonshot.cn/console/api-keys"),
            ("minimax", "https://platform.minimax.io/"),
            ("minimax-cn", "https://platform.minimaxi.com/"),
            ("alibaba", "https://dashscope-intl.console.aliyun.com/"),
            ("alibaba-cn", "https://dashscope.console.aliyun.com/"),
            ("qwen", "https://dashscope.console.aliyun.com/"),
            ("siliconflow", "https://cloud.siliconflow.cn/account/ak"),
            ("siliconflow-cn", "https://cloud.siliconflow.cn/account/ak"),
            ("groq", "https://console.groq.com/keys"),
            ("together", "https://api.together.xyz/settings/api-keys"),
            ("openrouter", "https://openrouter.ai/settings/keys"),
            ("volcengine", "https://console.volcengine.com/ark/"),
            ("baidu", "https://console.bce.baidu.com/qianfan/"),
            ("tencent", "https://console.cloud.tencent.com/hunyuan"),
            ("baichuan", "https://platform.baichuan-ai.com/"),
            ("stepfun", "https://platform.stepfun.com/"),
            ("stepfun-ai", "https://platform.stepfun.ai/"),
            ("perplexity", "https://www.perplexity.ai/settings/api"),
            ("mistral", "https://console.mistral.ai/"),
            ("01-ai", "https://platform.lingyiwanwu.com/"),
        ];

        let default_base_urls = [
            ("openai", "https://api.openai.com/v1"),
            ("anthropic", "https://api.anthropic.com/v1"),
            (
                "google",
                "https://generativelanguage.googleapis.com/v1beta/openai",
            ),
            ("xai", "https://api.x.ai/v1"),
            ("x-ai", "https://api.x.ai/v1"),
            ("deepseek", "https://api.deepseek.com"),
            ("zhipuai", "https://open.bigmodel.cn/api/paas/v4"),
            ("zhipu", "https://open.bigmodel.cn/api/paas/v4"),
            ("moonshotai", "https://api.moonshot.ai/v1"),
            ("moonshot", "https://api.moonshot.ai/v1"),
            ("minimax", "https://api.minimax.io/v1"),
            ("minimax-cn", "https://api.minimaxi.com/v1"),
            (
                "alibaba-cn",
                "https://dashscope.aliyuncs.com/compatible-mode/v1",
            ),
            (
                "alibaba",
                "https://dashscope-intl.aliyuncs.com/compatible-mode/v1",
            ),
            ("qwen", "https://dashscope.aliyuncs.com/compatible-mode/v1"),
            ("groq", "https://api.groq.com/openai/v1"),
            ("mistral", "https://api.mistral.ai/v1"),
            ("perplexity", "https://api.perplexity.ai"),
            ("cohere", "https://api.cohere.ai/v1"),
            ("togetherai", "https://api.together.xyz/v1"),
            ("deepinfra", "https://api.deepinfra.com/v1"),
            ("cerebras", "https://api.cerebras.ai/v1"),
            ("venice", "https://api.venice.ai/v1"),
            (
                "cloudflare-ai-gateway",
                "https://gateway.ai.cloudflare.com/v1",
            ),
            ("fireworks-ai", "https://api.fireworks.ai/inference/v1"),
        ];

        for (id, provider_val) in obj.iter_mut() {
            if let Some(p) = provider_val.as_object_mut() {
                // 合并 base_url
                if !p.contains_key("api")
                    || p.get("api")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .is_empty()
                {
                    if let Some((_, fallback_url)) = default_base_urls.iter().find(|(k, _)| k == id)
                    {
                        p.insert(
                            "api".to_string(),
                            serde_json::Value::String(fallback_url.to_string()),
                        );
                    }
                }
                // 合并 api_key_url (内置优先，指向密钥创建页)
                if let Some((_, fallback_key_url)) = api_key_urls.iter().find(|(k, _)| k == id) {
                    p.insert(
                        "doc".to_string(),
                        serde_json::Value::String(fallback_key_url.to_string()),
                    );
                }
            }
        }
    }
    registry
}

#[command]
pub async fn get_providers_registry(app: AppHandle) -> Result<Option<serde_json::Value>, String> {
    let data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to get app data dir: {}", e))?;
    let path = data_dir.join("providers_registry.json");

    if !path.exists() {
        return Ok(None);
    }

    let content = std::fs::read_to_string(&path)
        .map_err(|e| format!("Failed to read providers registry file: {}", e))?;

    let json: serde_json::Value = serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse providers registry JSON: {}", e))?;

    Ok(Some(merge_registry_metadata(json)))
}

#[command]
pub async fn sync_providers_registry(
    app: AppHandle,
    ai_manager: State<'_, Arc<AIManager>>,
) -> Result<serde_json::Value, String> {
    let resp = ai_manager
        .client
        .get(MODELS_DEV_REGISTRY_URL)
        .timeout(std::time::Duration::from_secs(15))
        .send()
        .await
        .map_err(|e| format!("Failed to send request to models.dev: {}", e))?;

    if !resp.status().is_success() {
        return Err(format!(
            "HTTP error status from models.dev: {}",
            resp.status()
        ));
    }

    let text = resp
        .text()
        .await
        .map_err(|e| format!("Failed to read response body from models.dev: {}", e))?;

    // 校验 JSON 格式是否正确
    let json: serde_json::Value = serde_json::from_str(&text)
        .map_err(|e| format!("Fetched registry is not valid JSON: {}", e))?;

    // 写入本地
    let data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to get app data dir: {}", e))?;

    if !data_dir.exists() {
        std::fs::create_dir_all(&data_dir)
            .map_err(|e| format!("Failed to create app data dir: {}", e))?;
    }

    let path = data_dir.join("providers_registry.json");
    std::fs::write(&path, &text)
        .map_err(|e| format!("Failed to write providers registry to file: {}", e))?;

    Ok(merge_registry_metadata(json))
}
