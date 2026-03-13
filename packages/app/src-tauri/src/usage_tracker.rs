use crate::app_config::SortMode;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::{AppHandle, Manager};

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct CommandUsageStats {
    pub command_name: String,
    pub usage_count: u32,
    pub last_used: i64, // Unix timestamp in seconds
}

#[derive(Clone)]
pub struct UsageTracker {
    stats: HashMap<String, CommandUsageStats>,
    file_path: PathBuf,
    pending_saves: u32,
    last_save_time: SystemTime,
}

impl UsageTracker {
    pub fn new(app: &AppHandle) -> Self {
        let file_path = Self::get_usage_file_path(app);
        let stats = Self::load_from_file(&file_path);

        UsageTracker {
            stats,
            file_path,
            pending_saves: 0,
            last_save_time: SystemTime::now(),
        }
    }

    fn get_usage_file_path(app: &AppHandle) -> PathBuf {
        let path = app.path().app_data_dir().unwrap();
        if !path.exists() {
            fs::create_dir_all(&path).unwrap();
        }
        path.join("command_usage.json")
    }

    fn load_from_file(path: &PathBuf) -> HashMap<String, CommandUsageStats> {
        if !path.exists() {
            return HashMap::new();
        }

        match fs::read_to_string(path) {
            Ok(json_str) => {
                let result: Result<Vec<CommandUsageStats>, serde_json::Error> =
                    serde_json::from_str(&json_str);
                match result {
                    Ok(stats_vec) => stats_vec
                        .into_iter()
                        .map(|stat| (stat.command_name.clone(), stat))
                        .collect(),
                    Err(e) => {
                        eprintln!("Failed to parse command_usage.json: {}", e);
                        HashMap::new()
                    }
                }
            }
            Err(e) => {
                eprintln!("Failed to read command_usage.json: {}", e);
                HashMap::new()
            }
        }
    }

    fn save_to_file(&self) {
        let stats_vec: Vec<CommandUsageStats> = self.stats.values().cloned().collect();
        let json = serde_json::to_string_pretty(&stats_vec).unwrap();
        if let Err(e) = fs::write(&self.file_path, json) {
            eprintln!("Failed to save command_usage.json: {}", e);
        }
    }

    pub fn record_usage(&mut self, command_name: &str) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        self.stats
            .entry(command_name.to_string())
            .and_modify(|stat| {
                stat.usage_count += 1;
                stat.last_used = now;
            })
            .or_insert(CommandUsageStats {
                command_name: command_name.to_string(),
                usage_count: 1,
                last_used: now,
            });

        self.pending_saves += 1;

        // 批量保存策略：每 5 次使用或距离上次保存超过 30 秒
        let should_save = self.pending_saves >= 5
            || self
                .last_save_time
                .elapsed()
                .map(|d| d.as_secs() >= 30)
                .unwrap_or(true);

        if should_save {
            self.save_to_file();
            self.pending_saves = 0;
            self.last_save_time = SystemTime::now();
        }
    }

    pub fn get_usage_count(&self, command_name: &str) -> u32 {
        self.stats
            .get(command_name)
            .map(|stat| stat.usage_count)
            .unwrap_or(0)
    }

    pub fn get_last_used(&self, command_name: &str) -> Option<i64> {
        self.stats.get(command_name).map(|stat| stat.last_used)
    }

    pub fn calculate_score(&self, command_name: &str, mode: &SortMode) -> f64 {
        match mode {
            SortMode::Smart => {
                let usage_count = self.get_usage_count(command_name) as f64;
                let recency_score = self.calculate_recency_score(command_name);
                usage_count * 0.7 + recency_score * 0.3
            }
            SortMode::Frequency => self.get_usage_count(command_name) as f64,
            SortMode::Recent => self.calculate_recency_score(command_name),
            SortMode::Default => 0.0,
        }
    }

    fn calculate_recency_score(&self, command_name: &str) -> f64 {
        if let Some(last_used) = self.get_last_used(command_name) {
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64;
            let days_ago = (now - last_used) as f64 / 86400.0; // 转换为天数

            // 使用指数衰减：最近使用的分数更高
            // 1 天前: ~90 分, 7 天前: ~50 分, 30 天前: ~10 分
            100.0 * (-days_ago / 10.0).exp()
        } else {
            0.0
        }
    }

    #[allow(dead_code)]
    pub fn force_save(&mut self) {
        if self.pending_saves > 0 {
            self.save_to_file();
            self.pending_saves = 0;
            self.last_save_time = SystemTime::now();
        }
    }

    pub fn clear_all(&mut self) {
        self.stats.clear();
        self.save_to_file();
        self.pending_saves = 0;
    }

    pub fn get_all_stats(&self) -> Vec<CommandUsageStats> {
        self.stats.values().cloned().collect()
    }
}

impl Drop for UsageTracker {
    fn drop(&mut self) {
        // Save any pending changes when the tracker is dropped
        if self.pending_saves > 0 {
            self.save_to_file();
        }
    }
}

// 全局状态管理
pub struct UsageTrackerState(pub Mutex<Option<UsageTracker>>);

// 辅助函数：确保 tracker 已初始化并执行操作
fn with_tracker<F, R>(
    app: &AppHandle,
    state: &tauri::State<'_, UsageTrackerState>,
    operation: F,
) -> Result<R, String>
where
    F: FnOnce(&mut UsageTracker) -> R,
{
    let mut tracker_opt = state.0.lock().map_err(|e| e.to_string())?;

    if tracker_opt.is_none() {
        *tracker_opt = Some(UsageTracker::new(app));
    }

    tracker_opt
        .as_mut()
        .map(operation)
        .ok_or_else(|| "Failed to get tracker".to_string())
}

// 辅助函数：只读操作
fn with_tracker_readonly<F, R>(
    app: &AppHandle,
    state: &tauri::State<'_, UsageTrackerState>,
    operation: F,
) -> Result<R, String>
where
    F: FnOnce(&UsageTracker) -> R,
{
    let mut tracker_opt = state.0.lock().map_err(|e| e.to_string())?;

    if tracker_opt.is_none() {
        *tracker_opt = Some(UsageTracker::new(app));
    }

    tracker_opt
        .as_ref()
        .map(operation)
        .ok_or_else(|| "Failed to get tracker".to_string())
}

// Tauri 命令
#[tauri::command]
pub fn record_command_usage(
    app: AppHandle,
    state: tauri::State<'_, UsageTrackerState>,
    command_name: String,
) -> Result<(), String> {
    // 检查是否启用使用追踪
    let config_state = app.state::<crate::app_config::AppConfigState>();
    let config = config_state.0.lock().map_err(|e| e.to_string())?;

    if !config.enable_usage_tracking {
        return Ok(());
    }
    drop(config); // 释放锁

    with_tracker(&app, &state, |tracker| {
        tracker.record_usage(&command_name);
    })
}

#[tauri::command]
pub fn get_usage_stats(
    app: AppHandle,
    state: tauri::State<'_, UsageTrackerState>,
) -> Result<Vec<CommandUsageStats>, String> {
    with_tracker_readonly(&app, &state, |tracker| tracker.get_all_stats())
}

#[tauri::command]
pub fn clear_usage_stats(
    app: AppHandle,
    state: tauri::State<'_, UsageTrackerState>,
) -> Result<(), String> {
    with_tracker(&app, &state, |tracker| {
        tracker.clear_all();
    })
}
