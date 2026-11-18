use serde::Deserialize;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Mutex;
use std::time::Duration;
use tauri::{webview::WebviewBuilder, Listener, Manager, WebviewUrl, WindowBuilder};
use tauri::{LogicalPosition, LogicalSize, PhysicalPosition, PhysicalSize, Rect};
use tauri_plugin_clipboard_manager::ClipboardExt;

const TRANSLATOR_TOP_BAR_HEIGHT: f64 = 36.0;
#[cfg(target_os = "macos")]
const TRANSLATOR_TITLEBAR_INSET: f64 = 28.0;
#[cfg(not(target_os = "macos"))]
const TRANSLATOR_TITLEBAR_INSET: f64 = 0.0;

/// 空闲超时时间：1 小时未被再次唤起时彻底销毁窗口，释放 WebView2 进程与全部内存
const TRANSLATOR_IDLE_TIMEOUT: Duration = Duration::from_secs(3600);

/// 复用探活超时时间：复用旧窗口时若底层 WebView2 挂起超过 600ms，触发安全熔断并重建
const TRANSLATOR_REUSE_TIMEOUT: Duration = Duration::from_millis(600);

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
    let content_height = inner_size.height.saturating_sub(webview_top);

    if let Some(ui_webview) = window.get_webview("translator-ui") {
        let _ = ui_webview.set_bounds(Rect {
            position: PhysicalPosition::new(0, titlebar_inset as i32).into(),
            size: PhysicalSize::new(inner_size.width, top_bar_height).into(),
        });
    }

    if let Some(content_webview) = window.get_webview("translator-content") {
        let _ = content_webview.set_bounds(Rect {
            position: PhysicalPosition::new(0, webview_top as i32).into(),
            size: PhysicalSize::new(inner_size.width, content_height).into(),
        });
    }

    Ok(())
}

/// 彻底销毁 translator 窗口（包括其中全部 webview 与子进程）
fn destroy_translator_window(app: &tauri::AppHandle) {
    IS_WINDOW_VISIBLE.store(false, Ordering::SeqCst);
    if let Some(window) = app.get_window("translator-host") {
        if let Err(e) = window.destroy() {
            eprintln!("[translator] 销毁窗口失败: {}", e);
        }
    }
}

/// 调度空闲销毁任务：隐藏 1 小时后若未被再次使用，自动销毁释放资源
fn schedule_idle_cleanup(app: &tauri::AppHandle) {
    let generation = IDLE_GENERATION.fetch_add(1, Ordering::SeqCst) + 1;
    let app_handle = app.clone();

    tauri::async_runtime::spawn(async move {
        tokio::time::sleep(TRANSLATOR_IDLE_TIMEOUT).await;

        // 若 1 小时内无新动作且窗口仍处于隐藏状态，彻底销毁
        if IDLE_GENERATION.load(Ordering::SeqCst) == generation
            && !IS_WINDOW_VISIBLE.load(Ordering::SeqCst)
        {
            sentry::add_breadcrumb(sentry::Breadcrumb {
                category: Some("translator".into()),
                message: Some("空闲达到 1 小时，后台自动销毁 translator 窗口以彻底释放资源".into()),
                level: sentry::Level::Info,
                ..Default::default()
            });
            destroy_translator_window(&app_handle);
        }
    });
}

/// 隐藏窗口并进入 1 小时空闲待机倒计时
fn hide_translator_window(app: &tauri::AppHandle) {
    if let Some(window) = app.get_window("translator-host") {
        let _ = window.hide();
        IS_WINDOW_VISIBLE.store(false, Ordering::SeqCst);
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

/// 尝试安全复用已有窗口（带 600ms 超时熔断保护，防止渲染进程僵死导致卡死）
async fn try_reuse_window(
    window: &tauri::Window,
    resolved_text: &Option<String>,
) -> Result<(), String> {
    let window_clone = window.clone();
    let text_clone = resolved_text.clone();

    tokio::time::timeout(TRANSLATOR_REUSE_TIMEOUT, async move {
        // 1. 探活 UI Webview：若渲染进程已崩溃或死锁，eval 会失败或卡死，由外层 timeout 熔断
        let ui_webview = window_clone
            .get_webview("translator-ui")
            .ok_or_else(|| "缺少 translator-ui webview".to_string())?;
        ui_webview
            .eval("void 0")
            .map_err(|e| format!("translator-ui 探活失败: {}", e))?;

        // 2. 检查内容 Webview
        let content_webview = window_clone
            .get_webview("translator-content")
            .ok_or_else(|| "缺少 translator-content webview".to_string())?;

        // 3. 若有新文本，更新并导航
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

        // 4. 显示并聚焦
        let _ = window_clone.unminimize();
        let _ = window_clone.show();
        let _ = window_clone.set_focus();

        Ok(())
    })
    .await
    .map_err(|_| "复用 translator 窗口超时 (WebView 响应超时或已僵死)".to_string())?
}

/// 打开/复用翻译器窗口
pub async fn open_window(app: &tauri::AppHandle, text: Option<String>) -> Result<(), String> {
    let resolved_text = resolve_translator_text(app, text)?;

    // 唤起时标记可见并更新代数，取消之前的空闲销毁
    IS_WINDOW_VISIBLE.store(true, Ordering::SeqCst);
    IDLE_GENERATION.fetch_add(1, Ordering::SeqCst);

    // ---- 复用路径 ----
    // 检查 translator-host 及其两个核心 webview 是否健全
    if let Some(window) = app.get_window("translator-host") {
        let has_ui = window.get_webview("translator-ui").is_some();
        let has_content = window.get_webview("translator-content").is_some();

        if has_ui && has_content {
            match try_reuse_window(&window, &resolved_text).await {
                Ok(_) => {
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
                        &format!("[translator] 窗口复用失败/超时，安全熔断重建: {}", e),
                        sentry::Level::Warning,
                    );
                    destroy_translator_window(app);
                }
            }
        } else {
            eprintln!("[translator] 窗口 webview 缺失，销毁后重建");
            destroy_translator_window(app);
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
    destroy_translator_window(app);

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

    let window_for_layout = window.clone();
    window.on_window_event(move |event| match event {
        tauri::WindowEvent::Resized(..) => {
            if let Err(e) = layout_translator_webviews(&window_for_layout) {
                eprintln!("[translator] Failed to update webview layout: {}", e);
            }
        }
        // 用户点击右上角关闭按钮：允许系统原生销毁，绝不调用 prevent_close，确保 100% 关得掉
        tauri::WindowEvent::CloseRequested { .. } => {
            IS_WINDOW_VISIBLE.store(false, Ordering::SeqCst);
            IDLE_GENERATION.fetch_add(1, Ordering::SeqCst);
            sentry::add_breadcrumb(sentry::Breadcrumb {
                category: Some("translator".into()),
                message: Some("用户点击原生关闭按钮，执行原生窗口销毁".into()),
                level: sentry::Level::Info,
                ..Default::default()
            });
        }
        tauri::WindowEvent::Destroyed => {
            IS_WINDOW_VISIBLE.store(false, Ordering::SeqCst);
            IDLE_GENERATION.fetch_add(1, Ordering::SeqCst);
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
        destroy_translator_window(app);
        return Err(e);
    }

    // 挂载与布局就绪后，统一显示并获得焦点
    let _ = window.show();
    let _ = window.set_focus();
    IS_WINDOW_VISIBLE.store(true, Ordering::SeqCst);

    sentry::add_breadcrumb(sentry::Breadcrumb {
        category: Some("translator".into()),
        message: Some(format!("全新创建 translator 窗口成功 (engine: {})", engine)),
        level: sentry::Level::Info,
        ..Default::default()
    });

    Ok(())
}
