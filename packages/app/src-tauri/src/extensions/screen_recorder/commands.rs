use super::engine::{RecordConfig, RecordStateSnapshot, ScreenRecordEngine};
use std::sync::Arc;
use tauri::{command, AppHandle, Manager, State};

pub struct RecorderAppState {
    pub engine: Arc<dyn ScreenRecordEngine>,
}

fn get_recordings_dir(app: &AppHandle) -> Result<std::path::PathBuf, String> {
    app.path()
        .video_dir()
        .or_else(|_| app.path().download_dir())
        .or_else(|_| app.path().app_data_dir())
        .map_err(|e| format!("Failed to resolve system directories: {}", e))
        .map(|path| path.join("Onin_Recordings"))
}

#[command]
pub async fn start_screen_record(
    app: AppHandle,
    state: State<'_, RecorderAppState>,
    config: RecordConfig,
) -> Result<String, String> {
    let video_dir = get_recordings_dir(&app)?;

    // 确保录屏文件夹存在
    std::fs::create_dir_all(&video_dir)
        .map_err(|e| format!("Failed to create recording directory: {}", e))?;

    let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S").to_string();
    let file_name = format!("Onin_{}.mp4", timestamp);
    let output_path = video_dir.join(file_name);

    state.engine.start(&config, &output_path, &app)?;

    Ok(output_path.to_string_lossy().to_string())
}

#[command]
pub async fn pause_screen_record(state: State<'_, RecorderAppState>) -> Result<(), String> {
    state.engine.pause()
}

#[command]
pub async fn resume_screen_record(state: State<'_, RecorderAppState>) -> Result<(), String> {
    state.engine.resume()
}

#[command]
pub async fn stop_screen_record(state: State<'_, RecorderAppState>) -> Result<(), String> {
    state.engine.stop()
}

#[command]
pub fn get_screen_record_state(state: State<'_, RecorderAppState>) -> RecordStateSnapshot {
    state.engine.get_state()
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RecordedVideo {
    pub name: String,
    pub path: String,
    pub size_bytes: u64,
    pub created_time: u64,
}

#[command]
pub fn get_recorded_videos(app: AppHandle) -> Result<Vec<RecordedVideo>, String> {
    let video_dir = get_recordings_dir(&app)?;

    if !video_dir.exists() {
        return Ok(Vec::new());
    }

    let mut videos = Vec::new();
    if let Ok(entries) = std::fs::read_dir(video_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() && path.extension().is_some_and(|ext| ext == "mp4") {
                if let Ok(metadata) = entry.metadata() {
                    let name = entry.file_name().to_string_lossy().to_string();
                    let size_bytes = metadata.len();
                    let created_time = metadata
                        .created()
                        .or_else(|_| metadata.modified())
                        .map(|t| {
                            t.duration_since(std::time::SystemTime::UNIX_EPOCH)
                                .unwrap_or_default()
                                .as_millis() as u64
                        })
                        .unwrap_or(0);
                    videos.push(RecordedVideo {
                        name,
                        path: path.to_string_lossy().to_string(),
                        size_bytes,
                        created_time,
                    });
                }
            }
        }
    }

    // 按时间倒序
    videos.sort_by(|a, b| b.created_time.cmp(&a.created_time));
    Ok(videos)
}

#[command]
pub async fn open_video_file(app: AppHandle, path: String) -> Result<(), String> {
    use tauri_plugin_opener::OpenerExt;
    app.opener()
        .open_path(&path, None::<String>)
        .map_err(|e| e.to_string())
}

#[command]
pub fn delete_video_file(path: String) -> Result<(), String> {
    std::fs::remove_file(path).map_err(|e| e.to_string())
}

#[command]
pub fn show_recorder_bar_window(app: AppHandle) {
    if let Some(win) = app.get_webview_window("screen-recorder-bar") {
        let _ = win.show();
        let _ = win.set_focus();
    }
}
