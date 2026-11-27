use serde::Deserialize;
use std::sync::atomic::{AtomicBool, AtomicIsize, AtomicU64, Ordering};
use std::sync::{LazyLock, Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tauri::{webview::WebviewBuilder, Listener, Manager, WebviewUrl, WindowBuilder};
use tauri::{LogicalPosition, LogicalSize, PhysicalPosition, PhysicalSize, Rect};
use tauri_plugin_clipboard_manager::ClipboardExt;

const TRANSLATOR_TOP_BAR_HEIGHT: f64 = 36.0;
#[cfg(target_os = "macos")]
const TRANSLATOR_TITLEBAR_INSET: f64 = 28.0;
#[cfg(not(target_os = "macos"))]
const TRANSLATOR_TITLEBAR_INSET: f64 = 0.0;

/// 空闲超时时间：10 分钟未被再次唤起时彻底销毁窗口，释放 WebView2 进程与全部内存
const TRANSLATOR_IDLE_TIMEOUT: Duration = Duration::from_secs(600);

const TRANSLATOR_ESC_SCRIPT: &str = r#"
(() => {
  const invoke = (cmd, args = {}) => {
    if (window.__TAURI__ && window.__TAURI__.core) {
      return window.__TAURI__.core.invoke(cmd, args);
    }
    if (window.__TAURI_INTERNALS__) {
      return window.__TAURI_INTERNALS__.invoke(cmd, args);
    }
    return Promise.reject(new Error("Tauri invoke bridge unavailable"));
  };

  window.addEventListener(
    "keydown",
    (event) => {
      if (event.key !== "Escape") return;
      event.preventDefault();
      event.stopPropagation();
      invoke("close_translator_window").catch((error) => {
        console.error("[translator] Failed to close window from Escape:", error);
      });
    },
    true
  );
})();
"#;

static CURRENT_ENGINE: Mutex<String> = Mutex::new(String::new());
static CURRENT_TEXT: Mutex<Option<String>> = Mutex::new(None);

/// 闲置计时代数：每次窗口隐藏/唤起时递增，用于废弃旧的定时销毁任务
static IDLE_GENERATION: AtomicU64 = AtomicU64::new(0);

/// 窗口当前是否处于可见/活跃状态
static IS_WINDOW_VISIBLE: AtomicBool = AtomicBool::new(false);

/// 当前窗口实例代数与已销毁代数（用于判断特定窗口实例是否已完成销毁）
static CURRENT_WINDOW_INSTANCE_ID: AtomicU64 = AtomicU64::new(0);
static DESTROYED_WINDOW_INSTANCE_ID: AtomicU64 = AtomicU64::new(0);

/// 当前窗口原生句柄（Windows 下用于兜底强制销毁残留僵尸窗口）
static CURRENT_WINDOW_HWND: AtomicIsize = AtomicIsize::new(0);

/// 最后一次活跃的墙钟时间戳（毫秒），用于在系统休眠唤醒后淘汰过期窗口
static LAST_ACTIVE_EPOCH_MS: AtomicU64 = AtomicU64::new(0);

/// 串行化窗口打开/重建/销毁操作，避免并发竞态导致状态不同步或僵尸窗口
static WINDOW_LIFECYCLE_LOCK: LazyLock<tokio::sync::Mutex<()>> =
    LazyLock::new(|| tokio::sync::Mutex::new(()));

fn current_epoch_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

fn mark_translator_active() {
    LAST_ACTIVE_EPOCH_MS.store(current_epoch_ms(), Ordering::SeqCst);
}

fn is_window_stale() -> bool {
    let last = LAST_ACTIVE_EPOCH_MS.load(Ordering::SeqCst);
    if last == 0 {
        return false;
    }
    let elapsed_ms = current_epoch_ms().saturating_sub(last);
    elapsed_ms >= TRANSLATOR_IDLE_TIMEOUT.as_millis() as u64
}

#[derive(Deserialize)]
struct TranslatorSwitchPayload {
    engine: Option<String>,
}

fn get_current_engine() -> String {
    let mut guard = CURRENT_ENGINE
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    if guard.is_empty() {
        *guard = "sougou".to_string();
    }
    guard.clone()
}

fn set_current_engine(engine: &str) {
    let mut guard = CURRENT_ENGINE
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    *guard = engine.to_string();
}

fn get_current_text() -> Option<String> {
    let guard = CURRENT_TEXT
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    guard.clone()
}

fn set_current_text(text: Option<String>) {
    let mut guard = CURRENT_TEXT
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    *guard = text;
}

fn resolve_translator_text(
    app: &tauri::AppHandle,
    text: Option<String>,
) -> Result<Option<String>, String> {
    let explicit_text = text
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string);

    if explicit_text.is_some() {
        return Ok(explicit_text);
    }

    match app.clipboard().read_text() {
        Ok(clipboard_text) => {
            let trimmed = clipboard_text.trim();
            if trimmed.is_empty() {
                Ok(None)
            } else {
                Ok(Some(trimmed.to_string()))
            }
        }
        Err(_) => Ok(None),
    }
}

fn build_engine_url(engine: &str, text: Option<&str>) -> String {
    let encoded = text
        .map(|value| url::form_urlencoded::byte_serialize(value.as_bytes()).collect::<String>())
        .unwrap_or_default();

    match engine {
        "google" => {
            if text.is_some() {
                format!(
                    "https://translate.google.com/?sl=auto&tl=zh-CN&text={}",
                    encoded
                )
            } else {
                "https://translate.google.com/".to_string()
            }
        }
        "baidu" => {
            if text.is_some() {
                format!("https://fanyi.baidu.com/#auto/zh/{}", encoded)
            } else {
                "https://fanyi.baidu.com".to_string()
            }
        }
        _ => {
            // 默认搜狗
            if text.is_some() {
                format!("https://fanyi.sogou.com/text?keyword={}", encoded)
            } else {
                "https://fanyi.sogou.com/".to_string()
            }
        }
    }
}

fn layout_translator_webviews(window: &tauri::Window) -> Result<(), String> {
    let inner_size = window.inner_size().map_err(|e| e.to_string())?;
    let scale_factor = window.scale_factor().map_err(|e| e.to_string())?;
    let titlebar_inset = (TRANSLATOR_TITLEBAR_INSET * scale_factor).round() as u32;
    let top_bar_height = (TRANSLATOR_TOP_BAR_HEIGHT * scale_factor).round() as u32;
    let webview_top = titlebar_inset.saturating_add(top_bar_height);

    // 防御保护：若窗口尚未完成渲染映射获取到 0 尺寸，采用默认 1000x800 的物理尺寸兜底，杜绝 0 尺寸白屏
    let width = if inner_size.width > 0 {
        inner_size.width
    } else {
        (1000.0 * scale_factor).round() as u32
    };
    let height = if inner_size.height > 0 {
        inner_size.height
    } else {
        (800.0 * scale_factor).round() as u32
    };

    let content_height = height.saturating_sub(webview_top);

    if let Some(ui_webview) = window.get_webview("translator-ui") {
        let _ = ui_webview.set_bounds(Rect {
            position: PhysicalPosition::new(0, titlebar_inset as i32).into(),
            size: PhysicalSize::new(width, top_bar_height).into(),
        });
    }

    if let Some(content_webview) = window.get_webview("translator-content") {
        let _ = content_webview.set_bounds(Rect {
            position: PhysicalPosition::new(0, webview_top as i32).into(),
            size: PhysicalSize::new(width, content_height).into(),
        });
    }

    Ok(())
}

fn is_instance_destroyed(instance_id: u64) -> bool {
    DESTROYED_WINDOW_INSTANCE_ID.load(Ordering::SeqCst) >= instance_id
}

/// 强制销毁指定窗口实例（在同一主线程任务内顺序执行 Tauri destroy 与 Windows 原生 HWND 兜底）
fn force_destroy_window_instance(
    app: &tauri::AppHandle,
    instance_id: u64,
    hwnd_raw: isize,
    reason: &str,
) {
    if instance_id > 0 && is_instance_destroyed(instance_id) {
        return;
    }

    eprintln!("[translator] 触发强制销毁窗口兜底: {}", reason);
    sentry::capture_message(
        &format!("[translator] 触发强制销毁窗口兜底: {}", reason),
        sentry::Level::Warning,
    );

    let app_for_destroy = app.clone();
    let _ = app.run_on_main_thread(move || {
        let is_same_instance =
            instance_id == 0 || CURRENT_WINDOW_INSTANCE_ID.load(Ordering::SeqCst) == instance_id;
        if is_same_instance {
            if let Some(window) = app_for_destroy.get_window("translator-host") {
                if let Err(e) = window.destroy() {
                    eprintln!("[translator] 兜底 window.destroy() 失败: {}", e);
                }
            }
        }

        #[cfg(target_os = "windows")]
        if hwnd_raw != 0 && (instance_id == 0 || !is_instance_destroyed(instance_id)) {
            unsafe {
                use windows::Win32::Foundation::HWND;
                use windows::Win32::UI::WindowsAndMessaging::{DestroyWindow, IsWindow};

                let hwnd = HWND(hwnd_raw);
                if IsWindow(hwnd).as_bool() {
                    let _ = DestroyWindow(hwnd);
                }
            }
        }
        #[cfg(not(target_os = "windows"))]
        let _ = hwnd_raw;
    });
}

/// 异步彻底销毁 translator 窗口并等待原生销毁确认（调用方需持有 WINDOW_LIFECYCLE_LOCK）
async fn destroy_translator_window_async(app: &tauri::AppHandle) {
    IS_WINDOW_VISIBLE.store(false, Ordering::SeqCst);
    let instance_id = CURRENT_WINDOW_INSTANCE_ID.load(Ordering::SeqCst);

    let app_for_destroy = app.clone();
    let _ = app.run_on_main_thread(move || {
        if instance_id == 0 || CURRENT_WINDOW_INSTANCE_ID.load(Ordering::SeqCst) == instance_id {
            if let Some(window) = app_for_destroy.get_window("translator-host") {
                if let Err(e) = window.destroy() {
                    eprintln!("[translator] 异步销毁窗口失败: {}", e);
                }
            }
        }
    });

    for _ in 0..10 {
        tokio::time::sleep(Duration::from_millis(25)).await;
        let gone_from_tauri = app.get_window("translator-host").is_none();
        let instance_done = instance_id == 0 || is_instance_destroyed(instance_id);
        if gone_from_tauri && instance_done {
            return;
        }
    }

    // 仅在 Tauri 正常销毁超时未完成时，才触发原生 HWND 强制销毁兜底
    if instance_id > 0 && !is_instance_destroyed(instance_id) {
        let hwnd_raw = CURRENT_WINDOW_HWND.load(Ordering::SeqCst);
        force_destroy_window_instance(app, instance_id, hwnd_raw, "异步销毁超时未收到 Destroyed");
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
}

/// 调度空闲销毁任务：隐藏 10 分钟后若未被再次使用，自动销毁释放资源
fn schedule_idle_cleanup(app: &tauri::AppHandle) {
    let generation = IDLE_GENERATION.fetch_add(1, Ordering::SeqCst) + 1;
    let app_handle = app.clone();

    tauri::async_runtime::spawn(async move {
        tokio::time::sleep(TRANSLATOR_IDLE_TIMEOUT).await;

        // 若 10 分钟内无新动作且窗口仍处于隐藏状态，彻底销毁
        if IDLE_GENERATION.load(Ordering::SeqCst) == generation
            && !IS_WINDOW_VISIBLE.load(Ordering::SeqCst)
        {
            let _lifecycle_guard = WINDOW_LIFECYCLE_LOCK.lock().await;
            if IDLE_GENERATION.load(Ordering::SeqCst) == generation
                && !IS_WINDOW_VISIBLE.load(Ordering::SeqCst)
            {
                sentry::add_breadcrumb(sentry::Breadcrumb {
                    category: Some("translator".into()),
                    message: Some(
                        "空闲达到 10 分钟，后台自动销毁 translator 窗口以彻底释放资源".into(),
                    ),
                    level: sentry::Level::Info,
                    ..Default::default()
                });
                destroy_translator_window_async(&app_handle).await;
            }
        }
    });
}

/// 隐藏窗口并进入 10 分钟空闲待机倒计时
fn hide_translator_window(app: &tauri::AppHandle) {
    if let Some(window) = app.get_window("translator-host") {
        let _ = window.hide();
        IS_WINDOW_VISIBLE.store(false, Ordering::SeqCst);
        mark_translator_active();
        schedule_idle_cleanup(app);
    }
}

/// 导出供前端或全局调用的关闭命令：隐藏并启动空闲回收
#[tauri::command]
pub fn close_translator_window(app: tauri::AppHandle) -> Result<(), String> {
    hide_translator_window(&app);
    Ok(())
}

/// 导出供前端获取当前引擎的命令
#[tauri::command]
pub fn get_translator_engine() -> String {
    get_current_engine()
}

/// 导出供前端或插件调用的打开窗口命令
#[tauri::command]
pub async fn open_translator_window(app: tauri::AppHandle) -> Result<(), String> {
    open_window(&app, None).await
}

/// 安全复用已有窗口：激活显示并立即刷新布局
async fn try_reuse_window(
    window: &tauri::Window,
    resolved_text: &Option<String>,
) -> Result<(), String> {
    let window_clone = window.clone();
    let text_clone = resolved_text.clone();

    // 1. 确保两个 webview 均存在
    let _ui_webview = window_clone
        .get_webview("translator-ui")
        .ok_or_else(|| "缺少 translator-ui webview".to_string())?;
    let content_webview = window_clone
        .get_webview("translator-content")
        .ok_or_else(|| "缺少 translator-content webview".to_string())?;

    // 2. 显示并聚焦宿主窗口
    let _ = window_clone.unminimize();
    let _ = window_clone.show();
    let _ = window_clone.set_focus();

    // 3. 关键：复用唤醒后必须立即重新计算并下发 bounds！
    // 隐藏状态唤醒时，WebView2 必须重新获得 bounds 才能恢复渲染管线，避免保持白屏
    layout_translator_webviews(&window_clone)?;

    // 4. 若有新文本，更新并导航
    if let Some(ref text) = text_clone {
        set_current_text(Some(text.clone()));
        let current_engine = get_current_engine();
        let url_str = build_engine_url(&current_engine, Some(text));

        if let Ok(url) = url_str.parse::<tauri::Url>() {
            content_webview
                .navigate(url)
                .map_err(|e| format!("导航失败: {}", e))?;
        }
    }

    // 5. 异步 50ms 再次进行布局确认，保障动画完成后的最终尺寸一致性
    let window_for_async_layout = window_clone.clone();
    tauri::async_runtime::spawn(async move {
        tokio::time::sleep(Duration::from_millis(50)).await;
        let _ = layout_translator_webviews(&window_for_async_layout);
    });

    Ok(())
}

/// 打开/复用翻译器窗口
pub async fn open_window(app: &tauri::AppHandle, text: Option<String>) -> Result<(), String> {
    let _lifecycle_guard = WINDOW_LIFECYCLE_LOCK.lock().await;
    let resolved_text = resolve_translator_text(app, text)?;

    // 唤起时标记可见并更新代数，取消之前的空闲销毁
    IS_WINDOW_VISIBLE.store(true, Ordering::SeqCst);
    IDLE_GENERATION.fetch_add(1, Ordering::SeqCst);

    // ---- 复用路径 ----
    // 检查 translator-host 及其两个核心 webview 是否健全，且未因系统休眠/过夜而过期
    if let Some(window) = app.get_window("translator-host") {
        if is_window_stale() {
            eprintln!("[translator] 窗口闲置/过夜超时，销毁后重建以避免 WebView2 黑屏");
            sentry::add_breadcrumb(sentry::Breadcrumb {
                category: Some("translator".into()),
                message: Some("窗口闲置/过夜超时，主动销毁并重建".into()),
                level: sentry::Level::Info,
                ..Default::default()
            });
            destroy_translator_window_async(app).await;
        } else {
            let has_ui = window.get_webview("translator-ui").is_some();
            let has_content = window.get_webview("translator-content").is_some();

            if has_ui && has_content {
                match try_reuse_window(&window, &resolved_text).await {
                    Ok(_) => {
                        mark_translator_active();
                        sentry::add_breadcrumb(sentry::Breadcrumb {
                            category: Some("translator".into()),
                            message: Some("成功复用已有 translator 窗口".into()),
                            level: sentry::Level::Info,
                            ..Default::default()
                        });
                        return Ok(());
                    }
                    Err(e) => {
                        eprintln!("[translator] 窗口复用失败: {}，销毁后重建", e);
                        sentry::capture_message(
                            &format!("[translator] 窗口复用失败，安全重建: {}", e),
                            sentry::Level::Warning,
                        );
                        destroy_translator_window_async(app).await;
                    }
                }
            } else {
                eprintln!("[translator] 窗口 webview 缺失，销毁后重建");
                destroy_translator_window_async(app).await;
            }
        }
    }

    // ---- 创建路径 ----
    create_translator_window(app, resolved_text.as_deref()).await
}

/// 创建全新的 translator 窗口（1 个本地 UI 栏 + 1 个动态内容区）
async fn create_translator_window(
    app: &tauri::AppHandle,
    text: Option<&str>,
) -> Result<(), String> {
    // 防御：清理任何可能残留的旧窗口
    if app.get_window("translator-host").is_some()
        || CURRENT_WINDOW_HWND.load(Ordering::SeqCst) != 0
    {
        destroy_translator_window_async(app).await;
    }

    set_current_text(text.map(|s| s.to_string()));
    let engine = get_current_engine();
    let initial_url = build_engine_url(&engine, text);
    let ui_url = format!("/translator-shell?engine={}", engine);

    // 关键：初始 visible 设为 false，待子 webview 挂载布局就绪后再显示，杜绝白屏外框悬挂
    let window = WindowBuilder::new(app, "translator-host")
        .title("Translator")
        .inner_size(1000.0, 800.0)
        .resizable(true)
        .visible(false)
        .build()
        .map_err(|e| format!("创建 translator 窗口失败: {}", e))?;

    let instance_id = CURRENT_WINDOW_INSTANCE_ID.fetch_add(1, Ordering::SeqCst) + 1;
    #[cfg(target_os = "windows")]
    let hwnd_raw = window.hwnd().map(|h| h.0 as isize).unwrap_or(0);
    #[cfg(not(target_os = "windows"))]
    let hwnd_raw: isize = 0;
    CURRENT_WINDOW_HWND.store(hwnd_raw, Ordering::SeqCst);

    let window_for_layout = window.clone();
    let app_for_close = app.clone();
    window.on_window_event(move |event| match event {
        tauri::WindowEvent::Resized(..) | tauri::WindowEvent::ScaleFactorChanged { .. } => {
            if let Err(e) = layout_translator_webviews(&window_for_layout) {
                eprintln!("[translator] Failed to update webview layout: {}", e);
            }
        }
        // 用户点击右上角关闭按钮：允许系统原生销毁，绝不调用 prevent_close，并调度 150ms 兜底强拆确保 100% 关得掉
        tauri::WindowEvent::CloseRequested { .. } => {
            IS_WINDOW_VISIBLE.store(false, Ordering::SeqCst);
            IDLE_GENERATION.fetch_add(1, Ordering::SeqCst);
            sentry::add_breadcrumb(sentry::Breadcrumb {
                category: Some("translator".into()),
                message: Some("用户点击原生关闭按钮，执行原生窗口销毁".into()),
                level: sentry::Level::Info,
                ..Default::default()
            });

            let app_fallback = app_for_close.clone();
            tauri::async_runtime::spawn(async move {
                tokio::time::sleep(Duration::from_millis(150)).await;
                let _lifecycle_guard = WINDOW_LIFECYCLE_LOCK.lock().await;
                if !is_instance_destroyed(instance_id)
                    && CURRENT_WINDOW_INSTANCE_ID.load(Ordering::SeqCst) == instance_id
                {
                    force_destroy_window_instance(
                        &app_fallback,
                        instance_id,
                        hwnd_raw,
                        "CloseRequested 后 150ms 仍未触发 Destroyed",
                    );
                }
            });
        }
        tauri::WindowEvent::Destroyed => {
            IS_WINDOW_VISIBLE.store(false, Ordering::SeqCst);
            IDLE_GENERATION.fetch_add(1, Ordering::SeqCst);
            DESTROYED_WINDOW_INSTANCE_ID.fetch_max(instance_id, Ordering::SeqCst);
            if CURRENT_WINDOW_HWND.load(Ordering::SeqCst) == hwnd_raw {
                CURRENT_WINDOW_HWND.store(0, Ordering::SeqCst);
            }
        }
        _ => {}
    });

    let titlebar_inset = TRANSLATOR_TITLEBAR_INSET;

    let setup_result = (|| {
        // 1. 创建顶部 UI 栏 (高度 36px)
        let _ui_webview = window
            .add_child(
                WebviewBuilder::new("translator-ui", WebviewUrl::App(ui_url.into()))
                    .initialization_script(TRANSLATOR_ESC_SCRIPT),
                LogicalPosition::new(0.0, titlebar_inset),
                LogicalSize::new(1000.0, TRANSLATOR_TOP_BAR_HEIGHT),
            )
            .map_err(|e| format!("创建 translator-ui webview 失败: {}", e))?;

        // 2. 创建内容区域 Webview (单个动态导航实例)
        let content_url = initial_url
            .parse::<tauri::Url>()
            .map_err(|e: url::ParseError| e.to_string())?;

        let _content_webview = window
            .add_child(
                WebviewBuilder::new("translator-content", WebviewUrl::External(content_url))
                    .initialization_script(TRANSLATOR_ESC_SCRIPT),
                LogicalPosition::new(0.0, titlebar_inset + TRANSLATOR_TOP_BAR_HEIGHT),
                LogicalSize::new(1000.0, 800.0 - titlebar_inset - TRANSLATOR_TOP_BAR_HEIGHT),
            )
            .map_err(|e| format!("创建 translator-content webview 失败: {}", e))?;

        // 3. 监听前端 Tab 切换事件：直接让 translator-content 导航到新引擎 URL
        let window_for_switch = window.clone();
        window.listen("translator_switch", move |event| {
            let payload = event.payload();
            let engine = serde_json::from_str::<TranslatorSwitchPayload>(payload)
                .ok()
                .and_then(|payload| payload.engine)
                .filter(|engine| matches!(engine.as_str(), "google" | "baidu" | "sougou"))
                .unwrap_or_else(|| "sougou".to_string());

            set_current_engine(&engine);
            mark_translator_active();
            let current_text = get_current_text();
            let target_url = build_engine_url(&engine, current_text.as_deref());

            if let Some(content_webview) = window_for_switch.get_webview("translator-content") {
                if let Ok(url) = target_url.parse::<tauri::Url>() {
                    let _ = content_webview.navigate(url);
                    let _ = content_webview.set_focus();
                }
            }
        });

        layout_translator_webviews(&window)?;

        Ok(())
    })();

    if let Err(e) = setup_result {
        eprintln!("[translator] 创建窗口失败({})，已清理残留窗口", e);
        sentry::capture_message(
            &format!("[translator] 创建窗口失败: {}", e),
            sentry::Level::Error,
        );
        destroy_translator_window_async(app).await;
        return Err(e);
    }

    // 挂载与布局就绪后，统一显示并获得焦点
    let _ = window.show();
    let _ = window.set_focus();
    IS_WINDOW_VISIBLE.store(true, Ordering::SeqCst);
    mark_translator_active();

    // 关键修复：窗口显示后立即再次刷新 bounds！
    // 隐藏状态转为可见时，必须重新下发 bounds 才能激活 WebView2 渲染管线，彻底杜绝白屏
    let _ = layout_translator_webviews(&window);

    // 异步 50ms 再次进行布局确认，避免 DWM 动画与 WM_SHOWWINDOW 时序差
    let window_for_async_layout = window.clone();
    tauri::async_runtime::spawn(async move {
        tokio::time::sleep(Duration::from_millis(50)).await;
        let _ = layout_translator_webviews(&window_for_async_layout);
    });

    sentry::add_breadcrumb(sentry::Breadcrumb {
        category: Some("translator".into()),
        message: Some(format!("全新创建 translator 窗口成功 (engine: {})", engine)),
        level: sentry::Level::Info,
        ..Default::default()
    });

    Ok(())
}
