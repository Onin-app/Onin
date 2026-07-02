use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecordConfig {
    pub fps: u32,
    pub record_audio: bool, // [Placeholder] 是否录制麦克风，目前由前端选项保存，为音频录像流扩展预留
    pub record_system_sound: bool, // [Placeholder] 是否录制系统声卡声音，为音频混音后续扩展预留
    pub exclude_own_window: bool, // 是否排除 Onin 自身窗口，目前由前端主动执行隐藏窗口，保留此参数记录
    pub monitor_index: Option<i32>, // 选中的屏幕索引，None/Some(-1) 为跟随鼠标
    pub save_folder_type: Option<String>, // 保存文件夹类型："video" | "download" | "desktop" | "custom"
    pub custom_save_folder: Option<String>, // 自定义保存路径
}

impl Default for RecordConfig {
    fn default() -> Self {
        Self {
            fps: 30,
            record_audio: true,
            record_system_sound: false,
            exclude_own_window: true,
            monitor_index: Some(0),
            save_folder_type: Some("video".to_string()),
            custom_save_folder: None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RecordState {
    Idle,
    Recording,
    Paused,
}

/// 录像引擎状态快照，用于发给前端
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecordStateSnapshot {
    pub state: RecordState,
    pub duration_secs: u64,
}

/// 跨平台屏幕录制引擎抽象接口
pub trait ScreenRecordEngine: Send + Sync {
    /// 开始录屏，视频输出到指定路径
    fn start(
        &self,
        config: &RecordConfig,
        output_path: &Path,
        app: &tauri::AppHandle,
    ) -> Result<(), String>;

    /// 暂停录制
    fn pause(&self) -> Result<(), String>;

    /// 恢复录制
    fn resume(&self) -> Result<(), String>;

    /// 停止录屏，完成文件封装和收尾工作
    fn stop(&self) -> Result<(), String>;

    /// 获取当前录像引擎的具体状态快照
    fn get_state(&self) -> RecordStateSnapshot;
}
