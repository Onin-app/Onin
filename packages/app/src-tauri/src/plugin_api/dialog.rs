use serde::{Deserialize, Serialize};
use tauri::AppHandle;
use tauri_plugin_dialog::{DialogExt, MessageDialogButtons, MessageDialogKind};
use tokio::sync::oneshot;

#[derive(Debug, Serialize, Deserialize)]
pub struct DialogError {
    pub name: String,
    pub message: String,
    pub code: Option<String>,
}

impl From<String> for DialogError {
    fn from(message: String) -> Self {
        DialogError {
            name: "DialogError".to_string(),
            message,
            code: None,
        }
    }
}

impl From<&str> for DialogError {
    fn from(message: &str) -> Self {
        DialogError {
            name: "DialogError".to_string(),
            message: message.to_string(),
            code: None,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MessageDialogOptions {
    pub title: Option<String>,
    pub message: String,
    pub kind: Option<String>,
    #[serde(rename = "okLabel")]
    pub ok_label: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ConfirmDialogOptions {
    pub title: Option<String>,
    pub message: String,
    pub kind: Option<String>,
    #[serde(rename = "okLabel")]
    pub ok_label: Option<String>,
    #[serde(rename = "cancelLabel")]
    pub cancel_label: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DialogFilter {
    pub name: String,
    pub extensions: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OpenDialogOptions {
    pub title: Option<String>,
    #[serde(rename = "defaultPath")]
    pub default_path: Option<String>,
    pub filters: Option<Vec<DialogFilter>>,
    pub multiple: Option<bool>,
    pub directory: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SaveDialogOptions {
    pub title: Option<String>,
    #[serde(rename = "defaultPath")]
    pub default_path: Option<String>,
    pub filters: Option<Vec<DialogFilter>>,
}

// 转换 kind 字符串到 MessageDialogKind
fn parse_message_kind(kind: Option<String>) -> MessageDialogKind {
    match kind.as_deref() {
        Some("info") => MessageDialogKind::Info,
        Some("warning") => MessageDialogKind::Warning,
        Some("error") => MessageDialogKind::Error,
        _ => MessageDialogKind::Info,
    }
}

// 转换 DialogFilter 到文件扩展名数组
fn convert_filters(filters: Option<Vec<DialogFilter>>) -> Vec<(String, Vec<String>)> {
    filters
        .unwrap_or_default()
        .into_iter()
        .map(|f| (f.name, f.extensions))
        .collect()
}

#[tauri::command]
pub async fn plugin_dialog_message(
    app: AppHandle,
    options: MessageDialogOptions,
) -> Result<(), DialogError> {
    let kind = parse_message_kind(options.kind);
    let title = options.title.unwrap_or_else(|| "Message".to_string());

    app.dialog()
        .message(&options.message)
        .title(&title)
        .kind(kind)
        .blocking_show();
    Ok(())
}

#[tauri::command]
pub async fn plugin_dialog_confirm(
    app: AppHandle,
    options: ConfirmDialogOptions,
) -> Result<bool, DialogError> {
    let kind = parse_message_kind(options.kind);
    let title = options.title.unwrap_or_else(|| "Confirm".to_string());

    // 使用异步 API 来创建确认对话框，明确设置为确认/取消按钮
    let (tx, rx) = oneshot::channel();

    app.dialog()
        .message(&options.message)
        .title(&title)
        .kind(kind)
        .buttons(MessageDialogButtons::OkCancel) // 明确设置为确认/取消按钮
        .show(move |confirmed| {
            let _ = tx.send(confirmed);
        });

    // 等待用户选择
    let confirmed = rx.await.unwrap_or(false);
    Ok(confirmed)
}

#[tauri::command]
pub async fn plugin_dialog_open(
    app: AppHandle,
    options: OpenDialogOptions,
) -> Result<Option<serde_json::Value>, DialogError> {
    let mut dialog = app.dialog().file();

    if let Some(title) = options.title {
        dialog = dialog.set_title(&title);
    }

    if let Some(default_path) = options.default_path {
        dialog = dialog.set_directory(&default_path);
    }

    let filters = convert_filters(options.filters);
    for (name, extensions) in filters {
        let ext_refs: Vec<&str> = extensions.iter().map(|s| s.as_str()).collect();
        dialog = dialog.add_filter(&name, &ext_refs);
    }

    // 使用 oneshot channel 来正确处理异步回调
    let (tx, rx) = oneshot::channel();

    if options.directory.unwrap_or(false) {
        // 选择文件夹
        dialog.pick_folder(move |path| {
            let result = path.map(|p| serde_json::Value::String(p.to_string()));
            let _ = tx.send(result);
        });
    } else if options.multiple.unwrap_or(false) {
        // 选择多个文件
        dialog.pick_files(move |paths| {
            let result = paths.map(|paths| {
                let path_strings: Vec<String> = paths.into_iter().map(|p| p.to_string()).collect();
                serde_json::json!(path_strings)
            });
            let _ = tx.send(result);
        });
    } else {
        // 选择单个文件
        dialog.pick_file(move |path| {
            let result = path.map(|p| serde_json::Value::String(p.to_string()));
            let _ = tx.send(result);
        });
    }

    // 等待对话框完成
    let final_result = rx.await.unwrap_or(None);
    Ok(final_result)
}

#[tauri::command]
pub async fn plugin_dialog_save(
    app: AppHandle,
    options: SaveDialogOptions,
) -> Result<Option<String>, DialogError> {
    let mut dialog = app.dialog().file();

    if let Some(title) = options.title {
        dialog = dialog.set_title(&title);
    }

    if let Some(default_path) = options.default_path {
        dialog = dialog.set_file_name(&default_path);
    }

    let filters = convert_filters(options.filters);
    for (name, extensions) in filters {
        let ext_refs: Vec<&str> = extensions.iter().map(|s| s.as_str()).collect();
        dialog = dialog.add_filter(&name, &ext_refs);
    }

    // 使用 oneshot channel 来正确处理异步回调
    let (tx, rx) = oneshot::channel();

    dialog.save_file(move |path| {
        let result = path.map(|p| p.to_string());
        let _ = tx.send(result);
    });

    // 等待对话框完成
    let path_string = rx.await.unwrap_or(None);
    Ok(path_string)
}

