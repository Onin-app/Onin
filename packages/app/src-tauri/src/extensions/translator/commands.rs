use serde::Deserialize;
use std::time::Duration;
use tauri::{webview::WebviewBuilder, Listener, Manager, WebviewUrl, WindowBuilder};
use tauri::{LogicalPosition, LogicalSize, PhysicalPosition, PhysicalSize, Rect};
use tauri_plugin_clipboard_manager::ClipboardExt;

const TRANSLATOR_TOP_BAR_HEIGHT: f64 = 36.0;
#[cfg(target_os = "macos")]
const TRANSLATOR_TITLEBAR_INSET: f64 = 28.0;
#[cfg(not(target_os = "macos"))]
const TRANSLATOR_TITLEBAR_INSET: f64 = 0.0;
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

/// translator-host 窗口内全部 webview 的 label。
/// 任何一次打开流程都必须确保这些 webview 全部存在且已加载出内容，
/// 否则该窗口视为损坏，需要销毁重建。
const TRANSLATOR_WEBVIEW_LABELS: [&str; 4] = [
    "translator-ui",
    "translator-google",
    "translator-baidu",
    "translator-sougou",
];

#[derive(Deserialize)]
struct TranslatorSwitchPayload {
    engine: Option<String>,
}

struct TranslatorUrls {
    google: String,
    baidu: String,
    sougou: String,
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

fn build_translator_urls(text: Option<&str>) -> TranslatorUrls {
    let encoded = text
        .map(|value| url::form_urlencoded::byte_serialize(value.as_bytes()).collect::<String>())
        .unwrap_or_default();

    let google = if text.is_some() {
        format!(
            "https://translate.google.com/?sl=auto&tl=zh-CN&text={}",
            encoded
        )
    } else {
        "https://translate.google.com/".to_string()
    };

    let baidu = if text.is_some() {
        format!("https://fanyi.baidu.com/#auto/zh/{}", encoded)
    } else {
        "https://fanyi.baidu.com".to_string()
    };

    let sougou = if text.is_some() {
        format!("https://fanyi.sogou.com/text?keyword={}", encoded)
    } else {
        "https://fanyi.sogou.com/".to_string()
    };

    TranslatorUrls {
        google,
        baidu,
        sougou,
    }
}

/// 将全部翻译 webview 导航到指定文本对应的 URL。
///
/// 只要调用方提供了文本就无条件导航（不比较当前 URL），这样即使
/// "A -> B -> A" 这类文本重复场景也能保证内容刷新，避免展示过期结果。
fn refresh_translator_webviews(window: &tauri::Window, text: Option<&str>) -> Result<(), String> {
    let urls = build_translator_urls(text);

    let targets = [
        ("translator-google", &urls.google),
        ("translator-baidu", &urls.baidu),
        ("translator-sougou", &urls.sougou),
    ];

    for (label, target_url) in targets {
        if let Some(webview) = window.get_webview(label) {
            let url = target_url
                .parse::<tauri::Url>()
                .map_err(|e: url::ParseError| format!("解析 URL 失败 '{}': {}", target_url, e))?;
            webview
                .navigate(url)
                .map_err(|e| format!("导航 webview '{}' 失败: {}", label, e))?;
        } else {
            return Err(format!("缺少 webview '{}'", label));
        }
    }

    Ok(())
}

fn layout_translator_webviews(window: &tauri::Window) -> Result<(), String> {
    let inner_size = window.inner_size().map_err(|e| e.to_string())?;
    let scale_factor = window.scale_factor().map_err(|e| e.to_string())?;
    let titlebar_inset = (TRANSLATOR_TITLEBAR_INSET * scale_factor).round() as u32;
    let top_bar_height = (TRANSLATOR_TOP_BAR_HEIGHT * scale_factor).round() as u32;
    let webview_top = titlebar_inset.saturating_add(top_bar_height);
    let content_height = inner_size.height.saturating_sub(webview_top);

    if let Some(ui_webview) = window.get_webview("translator-ui") {
        ui_webview
            .set_bounds(Rect {
                position: PhysicalPosition::new(0, titlebar_inset as i32).into(),
                size: PhysicalSize::new(inner_size.width, top_bar_height).into(),
            })
            .map_err(|e| e.to_string())?;
    }

    for label in ["translator-google", "translator-baidu", "translator-sougou"] {
        if let Some(webview) = window.get_webview(label) {
            webview
                .set_bounds(Rect {
                    position: PhysicalPosition::new(0, webview_top as i32).into(),
                    size: PhysicalSize::new(inner_size.width, content_height).into(),
                })
                .map_err(|e| e.to_string())?;
        }
    }

    Ok(())
}

// ============================================================================
// 窗口健康检查与恢复
// ============================================================================

/// 判断 webview 是否已完成首次导航（URL 不再停留在 `about:blank`）。
///
/// 未加载、加载失败或渲染进程失效时 URL 都会停留在 `about:blank`；
/// 该判据适用于外部翻译站点（Google / 百度 / 搜狗）。
fn webview_navigated(webview: &tauri::webview::Webview) -> bool {
    webview
        .url()
        .map(|url| !url.as_str().is_empty() && url.as_str() != "about:blank")
        .unwrap_or(false)
}

/// 判断本地 SPA webview（translator-ui）是否存活。
///
/// 除了 URL 判据外，再用 `eval("void 0")` 探活渲染进程：
/// 系统休眠唤醒后 WebView2 渲染进程可能已死，此时 ExecuteScript 会失败，
/// 即使 URL 仍停留在旧地址。该探活只用于我们自己的页面（CSP 允许 unsafe-eval），
/// 避免对外部站点产生误判。
fn ui_webview_alive(webview: &tauri::webview::Webview) -> bool {
    webview_navigated(webview) && webview.eval("void 0").is_ok()
}

/// 判断单个 webview 是否健康，`is_ui` 决定是否启用渲染进程探活。
fn webview_healthy(webview: &tauri::webview::Webview, is_ui: bool) -> bool {
    if is_ui {
        ui_webview_alive(webview)
    } else {
        webview_navigated(webview)
    }
}

/// 检查 translator 窗口是否健康可用。
///
/// 任一 webview 缺失（如上次创建中途失败留下的半成品窗口）、未加载出内容或
/// 渲染进程失效，都视为不健康，调用方应销毁窗口并重建。
fn translator_window_healthy(window: &tauri::Window) -> bool {
    let mut healthy = true;

    for label in TRANSLATOR_WEBVIEW_LABELS {
        match window.get_webview(label) {
            Some(webview) => {
                if !webview_healthy(&webview, label == "translator-ui") {
                    eprintln!(
                        "[translator] webview '{}' 内容无效 (url={:?})",
                        label,
                        webview.url()
                    );
                    healthy = false;
                }
            }
            None => {
                eprintln!("[translator] 缺少 webview '{}'，窗口视为损坏", label);
                healthy = false;
            }
        }
    }

    healthy
}

/// 销毁 translator 窗口（包括其中全部 webview）。
///
/// 使用 `destroy()` 直接销毁，不触发关闭事件，确保损坏的窗口不会残留。
fn destroy_translator_window(app: &tauri::AppHandle) {
    if let Some(window) = app.get_window("translator-host") {
        if let Err(e) = window.destroy() {
            eprintln!("[translator] 销毁异常窗口失败: {}", e);
        }
    }
}

/// 打开后白屏看门狗。
///
/// 窗口创建后延迟检查各 webview 是否真正加载出内容；若仍停留在 `about:blank`
/// （WebView2 在长时间运行 / 休眠唤醒后可能无法正常渲染新 webview），
/// 自动 reload 重试，避免用户看到"能弹出但全空白"的窗口。
fn spawn_blank_watchdog(app: tauri::AppHandle) {
    tauri::async_runtime::spawn(async move {
        const CHECK_DELAY: Duration = Duration::from_secs(4);
        const MAX_RETRIES: u32 = 2;

        // 先给页面加载留出时间
        tokio::time::sleep(CHECK_DELAY).await;

        for attempt in 0..=MAX_RETRIES {
            let Some(window) = app.get_window("translator-host") else {
                // 窗口已被关闭/销毁
                return;
            };

            // 找出尚未加载出内容的 webview
            let mut blank_labels: Vec<&str> = Vec::new();
            for label in TRANSLATOR_WEBVIEW_LABELS {
                match window.get_webview(label) {
                    Some(webview) => {
                        if !webview_healthy(&webview, label == "translator-ui") {
                            blank_labels.push(label);
                        }
                    }
                    None => {
                        blank_labels.push(label);
                    }
                }
            }

            if blank_labels.is_empty() {
                return;
            }

            eprintln!(
                "[translator] 检测到 webview 未加载成功 (attempt {}): {:?}",
                attempt + 1,
                blank_labels
            );

            if attempt == MAX_RETRIES {
                eprintln!("[translator] webview 多次加载失败，销毁窗口等待下次重建");
                destroy_translator_window(&app);
                return;
            }

            for label in &blank_labels {
                if let Some(webview) = window.get_webview(label) {
                    if let Err(e) = webview.reload() {
                        eprintln!("[translator] reload '{}' 失败: {}", label, e);
                    }
                }
            }

            tokio::time::sleep(CHECK_DELAY).await;
        }
    });
}

/// Open Translator Window with Multi-Webview Architecture
#[tauri::command]
pub async fn open_translator_window(app: tauri::AppHandle) -> Result<(), String> {
    open_window(&app, None).await
}

#[tauri::command]
pub fn close_translator_window(app: tauri::AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_window("translator-host") {
        window.close().map_err(|e| e.to_string())
    } else {
        Ok(())
    }
}

pub async fn open_window(app: &tauri::AppHandle, text: Option<String>) -> Result<(), String> {
    let resolved_text = resolve_translator_text(app, text)?;

    // ---- 复用路径 ----
    // 仅当窗口结构完整且 webview 已加载出内容时才复用；
    // 否则（缺失 webview / about:blank / 渲染进程已死）销毁重建，
    // 避免把损坏的空白窗口重新弹给用户。
    if let Some(window) = app.get_window("translator-host") {
        if translator_window_healthy(&window) {
            if let Some(text) = resolved_text.as_deref() {
                // 有新的翻译文本：无条件刷新全部引擎
                if let Err(e) = refresh_translator_webviews(&window, Some(text)) {
                    eprintln!("[translator] 刷新翻译内容失败({})，销毁重建窗口", e);
                    destroy_translator_window(app);
                } else {
                    if let Err(e) = window.set_focus() {
                        return Err(e.to_string());
                    }
                    return Ok(());
                }
            } else {
                // 剪贴板为空：保留当前已加载的翻译内容，仅聚焦
                if let Err(e) = window.set_focus() {
                    return Err(e.to_string());
                }
                return Ok(());
            }
        } else {
            eprintln!("[translator] 检测到窗口异常，销毁后重建");
            destroy_translator_window(app);
        }
    }

    // ---- 创建路径 ----
    create_translator_window(app, resolved_text.as_deref()).await
}

/// 创建全新的 translator 窗口及其全部 webview。
///
/// 任何一步失败都会销毁已创建的部分，保证不会留下"半成品"窗口：
/// 半成品窗口会在下一次快捷键触发时被 `get_window` 命中并被当成正常窗口复用，
/// 这正是"弹窗能弹出来、但内容全空白"的常见根因之一。
async fn create_translator_window(
    app: &tauri::AppHandle,
    text: Option<&str>,
) -> Result<(), String> {
    // 防御：清理任何可能残留的旧窗口
    destroy_translator_window(app);

    let window = WindowBuilder::new(app, "translator-host")
        .title("Translator")
        .inner_size(1000.0, 800.0)
        .resizable(true)
        .build()
        .map_err(|e| format!("创建 translator 窗口失败: {}", e))?;

    let window_for_layout = window.clone();
    window.on_window_event(move |event| match event {
        tauri::WindowEvent::Resized(..) => {
            if let Err(e) = layout_translator_webviews(&window_for_layout) {
                eprintln!("[translator] Failed to update webview layout: {}", e);
            }
        }
        _ => {}
    });

    // 2. Create UI Webview (Top 50px)
    // This loads the dedicated translator shell route.
    let titlebar_inset = TRANSLATOR_TITLEBAR_INSET;

    let setup_result = (|| {
        let _ui_webview = window
            .add_child(
                WebviewBuilder::new("translator-ui", WebviewUrl::App("/translator-shell".into()))
                    .initialization_script(TRANSLATOR_ESC_SCRIPT),
                LogicalPosition::new(0.0, titlebar_inset),
                LogicalSize::new(1000.0, TRANSLATOR_TOP_BAR_HEIGHT),
            )
            .map_err(|e| format!("创建 translator-ui webview 失败: {}", e))?;

        let urls = build_translator_urls(text);

        // 3. Create Google Webview (Rest of the area, default visible)
        let google_webview = window
            .add_child(
                WebviewBuilder::new(
                    "translator-google",
                    WebviewUrl::External(
                        urls.google
                            .parse::<tauri::Url>()
                            .map_err(|e: url::ParseError| e.to_string())?,
                    ),
                )
                .initialization_script(TRANSLATOR_ESC_SCRIPT),
                LogicalPosition::new(0.0, titlebar_inset + TRANSLATOR_TOP_BAR_HEIGHT),
                LogicalSize::new(1000.0, 800.0 - titlebar_inset - TRANSLATOR_TOP_BAR_HEIGHT),
            )
            .map_err(|e| format!("创建 translator-google webview 失败: {}", e))?;

        // Baidu
        let baidu_webview = window
            .add_child(
                WebviewBuilder::new(
                    "translator-baidu",
                    WebviewUrl::External(
                        urls.baidu
                            .parse::<tauri::Url>()
                            .map_err(|e: url::ParseError| e.to_string())?,
                    ),
                )
                .initialization_script(TRANSLATOR_ESC_SCRIPT),
                LogicalPosition::new(0.0, titlebar_inset + TRANSLATOR_TOP_BAR_HEIGHT),
                LogicalSize::new(1000.0, 800.0 - titlebar_inset - TRANSLATOR_TOP_BAR_HEIGHT),
            )
            .map_err(|e| format!("创建 translator-baidu webview 失败: {}", e))?;

        // Sougou
        let sougou_webview = window
            .add_child(
                WebviewBuilder::new(
                    "translator-sougou",
                    WebviewUrl::External(
                        urls.sougou
                            .parse::<tauri::Url>()
                            .map_err(|e: url::ParseError| e.to_string())?,
                    ),
                )
                .initialization_script(TRANSLATOR_ESC_SCRIPT),
                LogicalPosition::new(0.0, titlebar_inset + TRANSLATOR_TOP_BAR_HEIGHT),
                LogicalSize::new(1000.0, 800.0 - titlebar_inset - TRANSLATOR_TOP_BAR_HEIGHT),
            )
            .map_err(|e| format!("创建 translator-sougou webview 失败: {}", e))?;

        // Hide others initially
        // Default show sougou (since it's first in the list in frontend)
        google_webview.hide().map_err(|e| e.to_string())?;
        baidu_webview.hide().map_err(|e| e.to_string())?;
        // sougou_webview is shown by default

        // 5. Listen for switch event
        // The UI webview will emit "translator_switch".

        let google_webview_clone = google_webview.clone();
        let baidu_webview_clone = baidu_webview.clone();
        let sougou_webview_clone = sougou_webview.clone();

        window.listen("translator_switch", move |event| {
            let payload = event.payload();
            let engine = serde_json::from_str::<TranslatorSwitchPayload>(payload)
                .ok()
                .and_then(|payload| payload.engine)
                .filter(|engine| matches!(engine.as_str(), "google" | "baidu" | "sougou"))
                .unwrap_or_else(|| "sougou".to_string());

            // Hide all first
            let _ = google_webview_clone.hide();
            let _ = baidu_webview_clone.hide();
            let _ = sougou_webview_clone.hide();

            match engine.as_str() {
                "google" => {
                    let _ = google_webview_clone.show();
                    let _ = google_webview_clone.set_focus();
                }
                "baidu" => {
                    let _ = baidu_webview_clone.show();
                    let _ = baidu_webview_clone.set_focus();
                }
                _ => {
                    // sougou
                    let _ = sougou_webview_clone.show();
                    let _ = sougou_webview_clone.set_focus();
                }
            }
        });

        layout_translator_webviews(&window)?;

        Ok(())
    })();

    if let Err(e) = setup_result {
        // 清理半成品窗口，防止下次打开时复用坏状态
        eprintln!("[translator] 创建窗口失败({})，已清理残留窗口", e);
        destroy_translator_window(app);
        return Err(e);
    }

    // 创建成功后启动白屏看门狗：延迟检查加载情况，必要时自动 reload
    spawn_blank_watchdog(app.clone());

    Ok(())
}
