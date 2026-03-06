use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tauri::{AppHandle, Emitter, Manager, State};
use tokio::sync::Mutex;
use tokio_cron_scheduler::{Job, JobScheduler};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduleTask {
    pub id: String,
    pub plugin_id: String,
    pub cron: String,
    pub enabled: bool,
    #[serde(default)]
    pub execute_at: Option<u64>,
    #[serde(skip)]
    pub job_id: Option<Uuid>,
}

pub struct SchedulerState {
    scheduler: Arc<Mutex<JobScheduler>>,
    tasks: Arc<Mutex<HashMap<String, ScheduleTask>>>,
}

impl SchedulerState {
    pub async fn new() -> Result<Self, String> {
        let scheduler = JobScheduler::new()
            .await
            .map_err(|e| format!("Failed to create scheduler: {}", e))?;

        Ok(Self {
            scheduler: Arc::new(Mutex::new(scheduler)),
            tasks: Arc::new(Mutex::new(HashMap::new())),
        })
    }

    pub async fn start(&self) -> Result<(), String> {
        self.scheduler
            .lock()
            .await
            .start()
            .await
            .map_err(|e| format!("Failed to start scheduler: {}", e))
    }

    pub async fn load_from_store(&self, app_handle: &AppHandle) -> Result<(), String> {
        use tauri_plugin_store::StoreExt;

        let store = app_handle
            .store("scheduler.json")
            .map_err(|e| format!("Failed to open store: {}", e))?;

        // 读取保存的任务
        if let Some(tasks_value) = store.get("tasks") {
            if let Ok(saved_tasks) =
                serde_json::from_value::<Vec<ScheduleTask>>(tasks_value.clone())
            {
                let mut success_count = 0;
                let mut failed_count = 0;
                let mut skipped_count = 0;

                // 重新注册所有任务
                for task in saved_tasks {
                    let task_key = format!("{}:{}", task.plugin_id, task.id);
                    let plugin_id = task.plugin_id.clone();
                    let task_id = task.id.clone();
                    let app_handle_clone = app_handle.clone();

                    // 创建 Job
                    let is_one_shot = task.execute_at.is_some();
                    let job_result = if let Some(execute_at) = task.execute_at {
                        let now = std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_millis() as u64;

                        // 已过期的 one-shot 任务直接丢弃，不触发
                        if execute_at <= now {
                            println!(
                                "[Scheduler] Skipping expired one-shot task {}: was scheduled for {}, now {}",
                                task_key, execute_at, now
                            );
                            // 从内存 map 中清除（store 在循环结束后统一保存）
                            skipped_count += 1;
                            continue;
                        }

                        let delay_ms = execute_at - now;
                        Job::new_one_shot_async(std::time::Duration::from_millis(delay_ms), move |_uuid, _l| {
                            let app_handle = app_handle_clone.clone();
                            let plugin_id = plugin_id.clone();
                            let task_id = task_id.clone();

                            Box::pin(async move {
                                if let Err(e) = app_handle.emit(
                                    "scheduler:execute-task",
                                    serde_json::json!({
                                        "pluginId": plugin_id,
                                        "taskId": task_id,
                                    }),
                                ) {
                                    eprintln!("[Scheduler] Failed to emit task execution event: {}", e);
                                }
                                
                                // Auto cleanup one-shot tasks
                                let state = app_handle.state::<SchedulerState>();
                                let task_key = format!("{}:{}", plugin_id, task_id);
                                state.tasks.lock().await.remove(&task_key);
                                let _ = state.save_to_store(&app_handle).await;
                            })
                        })
                    } else {
                        let six_field_cron = to_six_field_cron(&task.cron);
                        Job::new_async(six_field_cron.as_str(), move |_uuid, _l| {
                            let app_handle = app_handle_clone.clone();
                            let plugin_id = plugin_id.clone();
                            let task_id = task_id.clone();

                            Box::pin(async move {
                                if let Err(e) = app_handle.emit(
                                    "scheduler:execute-task",
                                    serde_json::json!({
                                        "pluginId": plugin_id,
                                        "taskId": task_id,
                                    }),
                                ) {
                                    eprintln!("[Scheduler] Failed to emit task execution event: {}", e);
                                }
                            })
                        })
                    };

                    match job_result {
                        Ok(job) => {
                            let job_id = job.guid();

                            match self.scheduler.lock().await.add(job).await {
                                Ok(_) => {
                                    let mut restored_task = task.clone();
                                    restored_task.job_id = Some(job_id);
                                    self.tasks
                                        .lock()
                                        .await
                                        .insert(task_key.clone(), restored_task);
                                    success_count += 1;
                                }
                                Err(e) => {
                                    eprintln!(
                                        "[Scheduler] Failed to add task {} to scheduler: {}",
                                        task_key, e
                                    );
                                    failed_count += 1;
                                }
                            }
                        }
                        Err(e) => {
                            eprintln!(
                                "[Scheduler] Failed to create job for task {}: {}",
                                task_key, e
                            );
                            failed_count += 1;
                        }
                    }
                }

                println!(
                    "[Scheduler] Loaded {} tasks from store ({} succeeded, {} failed, {} expired/skipped)",
                    success_count + failed_count,
                    success_count,
                    failed_count,
                    skipped_count
                );
            }
        }

        Ok(())
    }

    pub async fn save_to_store(&self, app_handle: &AppHandle) -> Result<(), String> {
        use tauri_plugin_store::StoreExt;

        let store = app_handle
            .store("scheduler.json")
            .map_err(|e| format!("Failed to open store: {}", e))?;

        let tasks = self.tasks.lock().await;
        let tasks_vec: Vec<ScheduleTask> = tasks.values().cloned().collect();

        store.set("tasks", serde_json::to_value(&tasks_vec).unwrap());

        store
            .save()
            .map_err(|e| format!("Failed to save store: {}", e))?;

        Ok(())
    }
}

#[derive(Deserialize)]
pub struct ScheduleOptions {
    pub id: String,
    pub cron: String,
}

#[derive(Deserialize)]
pub struct ScheduleOnceOptions {
    pub id: String,
    pub execute_at: u64,
}

/// 验证单个 cron 字段的范围
fn validate_cron_field(value: &str, min: u32, max: u32, field_name: &str) -> Result<(), String> {
    // 允许通配符
    if value == "*" {
        return Ok(());
    }

    // 处理范围 (例如: 1-5)
    if value.contains('-') {
        let parts: Vec<&str> = value.split('-').collect();
        if parts.len() != 2 {
            return Err(format!("Invalid range format in {}: {}", field_name, value));
        }
        let start = parts[0]
            .parse::<u32>()
            .map_err(|_| format!("Invalid number in {} range: {}", field_name, parts[0]))?;
        let end = parts[1]
            .parse::<u32>()
            .map_err(|_| format!("Invalid number in {} range: {}", field_name, parts[1]))?;

        if start < min || start > max || end < min || end > max {
            return Err(format!(
                "{} range {}-{} out of bounds ({}-{})",
                field_name, start, end, min, max
            ));
        }
        return Ok(());
    }

    // 处理列表 (例如: 1,3,5)
    if value.contains(',') {
        for part in value.split(',') {
            validate_cron_field(part.trim(), min, max, field_name)?;
        }
        return Ok(());
    }

    // 处理步长 (例如: */5 或 0-23/2)
    if value.contains('/') {
        let parts: Vec<&str> = value.split('/').collect();
        if parts.len() != 2 {
            return Err(format!("Invalid step format in {}: {}", field_name, value));
        }

        // 验证基础部分
        if parts[0] != "*" {
            validate_cron_field(parts[0], min, max, field_name)?;
        }

        // 验证步长值
        parts[1]
            .parse::<u32>()
            .map_err(|_| format!("Invalid step value in {}: {}", field_name, parts[1]))?;

        return Ok(());
    }

    // 单个数值
    let num = value
        .parse::<u32>()
        .map_err(|_| format!("Invalid number in {}: {}", field_name, value))?;

    if num < min || num > max {
        return Err(format!(
            "{} value {} out of bounds ({}-{})",
            field_name, num, min, max
        ));
    }

    Ok(())
}

/// 验证 cron 表达式格式
fn validate_cron(cron: &str) -> Result<(), String> {
    let parts: Vec<&str> = cron.split_whitespace().collect();
    if parts.len() != 5 {
        return Err("Invalid cron format. Expected: 'minute hour day month weekday'".to_string());
    }

    // 验证每个字段的范围
    validate_cron_field(parts[0], 0, 59, "minute")?;
    validate_cron_field(parts[1], 0, 23, "hour")?;
    validate_cron_field(parts[2], 1, 31, "day")?;
    validate_cron_field(parts[3], 1, 12, "month")?;
    validate_cron_field(parts[4], 0, 6, "weekday")?;

    Ok(())
}

/// 将5字段 cron（分 时 日 月 周）转换为 tokio_cron_scheduler 所需的6字段格式（秒 分 时 日 月 周）
fn to_six_field_cron(cron: &str) -> String {
    let parts: Vec<&str> = cron.split_whitespace().collect();
    if parts.len() == 5 {
        // 5字段：补充秒字段 0 在最前面
        format!("0 {}", cron)
    } else {
        // 已经是6字段或其他格式，直接返回
        cron.to_string()
    }
}

/// 注册定时任务
#[tauri::command]
pub async fn schedule_task(
    plugin_id: String,
    options: ScheduleOptions,
    app_handle: AppHandle,
    state: State<'_, SchedulerState>,
) -> Result<(), String> {
    use crate::plugin::PluginStore;
    use crate::plugin::types::find_plugin_by_id;

    // 验证 cron 表达式
    validate_cron(&options.cron)?;

    // 从插件 manifest 读取 maxTasks 配置
    let max_tasks = {
        let plugin_store = app_handle.state::<PluginStore>();
        let store_lock = plugin_store.0.lock().unwrap();

        if let Some(plugin) = find_plugin_by_id(&store_lock, &plugin_id) {
            // 检查插件是否启用
            if !plugin.enabled {
                return Err(format!("Plugin {} is disabled", plugin_id));
            }

            // 检查 scheduler 权限
            if let Some(permissions) = &plugin.manifest.permissions {
                if let Some(scheduler_perm) = &permissions.scheduler {
                    if !scheduler_perm.enable {
                        return Err(format!(
                            "Plugin {} does not have scheduler permission",
                            plugin_id
                        ));
                    }
                    scheduler_perm.max_tasks.unwrap_or(10)
                } else {
                    return Err(format!(
                        "Plugin {} does not have scheduler permission configured",
                        plugin_id
                    ));
                }
            } else {
                return Err(format!(
                    "Plugin {} does not have permissions configured",
                    plugin_id
                ));
            }
        } else {
            return Err(format!("Plugin {} not found", plugin_id));
        }
    };

    // 检查插件的任务数量限制
    let current_task_count = {
        let tasks = state.tasks.lock().await;
        tasks.values().filter(|t| t.plugin_id == plugin_id).count()
    };

    if current_task_count >= max_tasks {
        return Err(format!(
            "Task limit reached: plugin {} already has {} tasks (max: {})",
            plugin_id, current_task_count, max_tasks
        ));
    }

    let task_key = format!("{}:{}", plugin_id, options.id);
    let task_id = options.id.clone();
    let plugin_id_clone = plugin_id.clone();

    // 在闭包之前克隆 app_handle
    let app_handle_for_job = app_handle.clone();

    // 创建 Job（将5字段 cron 转换为 tokio_cron_scheduler 所需的6字段格式）
    let six_field_cron = to_six_field_cron(&options.cron);
    let job = Job::new_async(six_field_cron.as_str(), move |_uuid, _l| {
        let app_handle = app_handle_for_job.clone();
        let plugin_id = plugin_id_clone.clone();
        let task_id = task_id.clone();

        Box::pin(async move {
            // 通过事件通知前端执行任务
            if let Err(e) = app_handle.emit(
                "scheduler:execute-task",
                serde_json::json!({
                    "pluginId": plugin_id,
                    "taskId": task_id,
                }),
            ) {
                eprintln!("[Scheduler] Failed to emit task execution event: {}", e);
            }
        })
    })
    .map_err(|e| format!("Failed to create job: {}", e))?;

    let job_id = job.guid();

    // 添加到调度器
    state
        .scheduler
        .lock()
        .await
        .add(job)
        .await
        .map_err(|e| format!("Failed to add job to scheduler: {}", e))?;

    // 保存任务信息
    let task = ScheduleTask {
        id: options.id.clone(),
        plugin_id: plugin_id.clone(),
        cron: options.cron,
        enabled: true,
        execute_at: None,
        job_id: Some(job_id),
    };

    state.tasks.lock().await.insert(task_key, task);

    // 持久化（现在可以使用原始的 app_handle）
    state.save_to_store(&app_handle).await?;

    Ok(())
}

/// 注册单次定时任务 (One-shot task)
#[tauri::command]
pub async fn schedule_once(
    plugin_id: String,
    options: ScheduleOnceOptions,
    app_handle: AppHandle,
    state: State<'_, SchedulerState>,
) -> Result<(), String> {
    use crate::plugin::PluginStore;
    use crate::plugin::types::find_plugin_by_id;

    // 从插件 manifest 读取 maxTasks 配置
    let max_tasks = {
        let plugin_store = app_handle.state::<PluginStore>();
        let store_lock = plugin_store.0.lock().unwrap();

        if let Some(plugin) = find_plugin_by_id(&store_lock, &plugin_id) {
            if !plugin.enabled {
                return Err(format!("Plugin {} is disabled", plugin_id));
            }

            if let Some(permissions) = &plugin.manifest.permissions {
                if let Some(scheduler_perm) = &permissions.scheduler {
                    if !scheduler_perm.enable {
                        return Err(format!("Plugin {} does not have scheduler permission", plugin_id));
                    }
                    scheduler_perm.max_tasks.unwrap_or(10)
                } else {
                    return Err(format!("Plugin {} does not have scheduler permission configured", plugin_id));
                }
            } else {
                return Err(format!("Plugin {} does not have permissions configured", plugin_id));
            }
        } else {
            return Err(format!("Plugin {} not found", plugin_id));
        }
    };

    let current_task_count = {
        let tasks = state.tasks.lock().await;
        tasks.values().filter(|t| t.plugin_id == plugin_id).count()
    };

    if current_task_count >= max_tasks {
        return Err(format!(
            "Task limit reached: plugin {} already has {} tasks (max: {})",
            plugin_id, current_task_count, max_tasks
        ));
    }

    let task_key = format!("{}:{}", plugin_id, options.id);
    let task_id = options.id.clone();
    let plugin_id_clone = plugin_id.clone();
    let app_handle_for_job = app_handle.clone();

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64;
    let delay_ms = if options.execute_at > now { options.execute_at - now } else { 0 };

    let job = Job::new_one_shot_async(std::time::Duration::from_millis(delay_ms), move |_uuid, _l| {
        let app_handle = app_handle_for_job.clone();
        let plugin_id = plugin_id_clone.clone();
        let task_id = task_id.clone();

        Box::pin(async move {
            if let Err(e) = app_handle.emit(
                "scheduler:execute-task",
                serde_json::json!({
                    "pluginId": plugin_id,
                    "taskId": task_id,
                }),
            ) {
                eprintln!("[Scheduler] Failed to emit task execution event: {}", e);
            }

            // Auto cleanup
            let state = app_handle.state::<SchedulerState>();
            let task_key_inner = format!("{}:{}", plugin_id, task_id);
            state.tasks.lock().await.remove(&task_key_inner);
            let _ = state.save_to_store(&app_handle).await;
        })
    })
    .map_err(|e| format!("Failed to create job: {}", e))?;

    let job_id = job.guid();

    state.scheduler.lock().await.add(job).await.map_err(|e| format!("Failed to add job to scheduler: {}", e))?;

    let task = ScheduleTask {
        id: options.id.clone(),
        plugin_id: plugin_id.clone(),
        cron: "-".to_string(),
        enabled: true,
        execute_at: Some(options.execute_at),
        job_id: Some(job_id),
    };

    state.tasks.lock().await.insert(task_key, task);
    state.save_to_store(&app_handle).await?;

    Ok(())
}

/// 取消定时任务
#[tauri::command]
pub async fn cancel_task(
    plugin_id: String,
    task_id: String,
    app_handle: AppHandle,
    state: State<'_, SchedulerState>,
) -> Result<(), String> {
    let task_key = format!("{}:{}", plugin_id, task_id);

    // 获取任务信息
    let task = {
        let tasks = state.tasks.lock().await;
        tasks.get(&task_key).cloned()
    };

    if let Some(task) = task {
        // 从调度器移除 Job
        if let Some(job_id) = task.job_id {
            if let Err(e) = state.scheduler.lock().await.remove(&job_id).await {
                eprintln!(
                    "[Scheduler] Warning: Failed to remove job {} from scheduler: {}",
                    job_id, e
                );
                // 继续执行，确保从任务列表中移除
            }
        }

        // 从任务列表移除
        state.tasks.lock().await.remove(&task_key);

        // 持久化
        state.save_to_store(&app_handle).await?;
    }

    Ok(())
}

/// 获取所有任务
#[tauri::command]
pub async fn list_tasks(
    plugin_id: String,
    state: State<'_, SchedulerState>,
) -> Result<Vec<ScheduleTask>, String> {
    let tasks = state.tasks.lock().await;

    let plugin_tasks: Vec<ScheduleTask> = tasks
        .values()
        .filter(|task| task.plugin_id == plugin_id)
        .cloned()
        .collect();

    Ok(plugin_tasks)
}

/// 初始化调度器（在应用启动时调用）
pub async fn init_scheduler(app_handle: &AppHandle) -> Result<(), String> {
    let state = app_handle.state::<SchedulerState>();

    // 从存储加载任务
    state.load_from_store(app_handle).await?;

    // 启动调度器
    state.start().await?;

    println!("[Scheduler] Initialized successfully");
    Ok(())
}
