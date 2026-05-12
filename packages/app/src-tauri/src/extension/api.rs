//! # Extension API 模块
//!
//! 提供 Extension 相关的 Tauri 命令

use crate::extension::types::EmojiGridData;
use crate::extension::{self, ExtensionPreview};
use tauri::{command, AppHandle, Emitter, Manager};

/// 获取输入的实时预览结果
///
/// 前端在用户输入时调用此命令，获取 Extension 的预览结果
#[command]
pub fn get_extension_preview(app: AppHandle, input: String) -> Option<ExtensionPreview> {
    extension::get_preview_for_input(&app, &input)
}

/// 执行 Extension 命令并复制结果
#[command]
pub fn execute_extension(
    app: AppHandle,
    extension_id: String,
    command_code: String,
    input: String,
) -> extension::ExtensionResult {
    extension::execute_extension_command(&app, &extension_id, &command_code, &input)
}

#[command]
pub fn get_extensions(app: AppHandle) -> Vec<extension::types::ExtensionInfo> {
    extension::get_extension_infos(&app)
}

#[command]
pub async fn toggle_extension(
    app: AppHandle,
    extension_id: String,
    enabled: bool,
) -> Result<(), String> {
    if extension::get_extension_by_id(&extension_id).is_none() {
        return Err(format!("Extension not found: {}", extension_id));
    }

    let config_state = app.state::<crate::app_config::AppConfigState>();
    {
        let mut config = config_state.0.lock().map_err(|e| e.to_string())?;
        let is_disabled = config
            .disabled_extension_ids
            .iter()
            .any(|id| id == &extension_id);

        if enabled && is_disabled {
            config
                .disabled_extension_ids
                .retain(|id| id != &extension_id);
        } else if !enabled && !is_disabled {
            config.disabled_extension_ids.push(extension_id.clone());
        }

        crate::app_config::save_config(&app, &config)?;
    }

    crate::command_manager::commands::refresh_commands(app.clone()).await;
    let _ = app.emit("extensions_changed", ());

    Ok(())
}

/// 获取 Emoji 数据
///
/// Emoji 页面专用 API，直接获取 emoji 数据而不经过 preview 机制
#[command]
pub fn get_emoji_data(search_query: String) -> Option<EmojiGridData> {
    use crate::extensions::emoji::data;

    let groups = if search_query.is_empty() {
        data::get_all_groups()
    } else {
        data::search_emojis(&search_query)
    };

    if groups.is_empty() || groups.iter().all(|g| g.emojis.is_empty()) {
        return None;
    }

    Some(EmojiGridData { groups })
}

#[command]
pub fn get_color_conversion(input: String) -> Option<crate::extensions::color::ColorConversion> {
    crate::extensions::color::convert_color_value(&input)
}

/// 启动取色流程：截图主屏幕 → 隐藏主窗口 → 打开全屏 Overlay 窗口
#[command]
pub async fn start_color_picker(app: AppHandle) -> Result<(), String> {
    crate::extensions::color_picker::start_color_picker(app).await
}

/// Overlay WebView 启动后调用，获取截图数据
#[command]
pub fn get_color_picker_capture(
) -> Result<crate::extensions::color_picker::ColorPickerCapture, String> {
    crate::extensions::color_picker::get_color_picker_capture()
}

/// Overlay WebView 启动后调用，获取 RGBA 像素数据
#[command]
pub fn get_color_picker_image() -> Result<tauri::ipc::Response, String> {
    let capture = crate::extensions::color_picker::get_color_picker_capture()?;
    Ok(tauri::ipc::Response::new(capture.rgba_data))
}

/// 用户点击取色或取消后由 Overlay WebView 调用
/// - hex: Some(value) 表示已取色，None 表示已取消
///
/// # 死锁避免
/// 该命令由 Overlay 窗口自身的 IPC 调用触发。若在此同步关闭 Overlay 窗口，
/// 会导致：关窗需要向该窗口消息队列发消息 → 消息队列正在等 IPC 响应 → 死锁。
/// 解决方案：先 emit 结果（IPC 立即返回），再用 spawn 异步延迟关窗。
#[command]
pub async fn finish_color_picker(app: AppHandle, hex: Option<String>) -> Result<(), String> {
    // 清理截图缓存
    crate::extensions::color_picker::clear_capture_cache();

    // 将 hex 转换为完整颜色信息
    let picked_color = hex.and_then(|value| crate::extensions::color::convert_color_value(&value));

    // 先广播结果 —— 此时 IPC 响应可以正常返回给 Overlay JS
    app.emit("color_picker_result", picked_color)
        .map_err(|e| e.to_string())?;

    // 在后台异步恢复主窗口并隐藏 Overlay 窗口。
    // 延迟 80ms 确保 IPC 响应已送达，避免操作当前窗口消息队列时卡住。
    let app_clone = app.clone();
    tauri::async_runtime::spawn(async move {
        tokio::time::sleep(std::time::Duration::from_millis(80)).await;

        if let Some(main) = app_clone.get_webview_window("main") {
            let _ = main.show();
            let _ = main.set_focus();
        }

        tokio::time::sleep(std::time::Duration::from_millis(32)).await;

        if let Some(overlay) = app_clone.get_webview_window("color-picker-overlay") {
            let _ = overlay.hide();
        }
    });

    Ok(())
}
