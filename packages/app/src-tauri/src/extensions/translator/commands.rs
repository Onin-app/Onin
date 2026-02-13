use tauri::{
    webview::WebviewBuilder, Listener, Manager, WebviewUrl,
    WindowBuilder,
};
use tauri::{LogicalPosition, LogicalSize};

/// Open Translator Window with Multi-Webview Architecture
#[tauri::command]
pub async fn open_translator_window(app: tauri::AppHandle) -> Result<(), String> {
    open_window(&app, None).await
}

pub async fn open_window(app: &tauri::AppHandle, text: Option<String>) -> Result<(), String> {
    if let Some(window) = app.get_window("translator-host") {
        if let Err(e) = window.set_focus() {
             return Err(e.to_string());
        }
        // If window exists and text is provided, we might want to update the tabs.
        // But for now, let's just focus.
        // TODO: Handle updating text in existing window via eval or reload.
        return Ok(());
    }

    // 1. Create the Host Window (Container)
    let window = WindowBuilder::new(app, "translator-host")
        .title("Translator")
        .inner_size(1000.0, 800.0)
        .resizable(true)
        .build()
        .map_err(|e| e.to_string())?;

    // 2. Create UI Webview (Top 50px)
    // This loads the local Svelte route: /extensions/translator
    let _ui_webview = window
        .add_child(
            WebviewBuilder::new("translator-ui", WebviewUrl::App("extensions/translator".into()))
                .auto_resize(),
            LogicalPosition::new(0.0, 0.0),
            LogicalSize::new(1000.0, 50.0),
        )
        .map_err(|e| e.to_string())?;

    let url_encoded_text = text.as_deref().unwrap_or("").to_string(); // Simple for now, need proper encoding
    let encoded = url::form_urlencoded::byte_serialize(url_encoded_text.as_bytes()).collect::<String>();
    
    // Helper to append query (simplified)
    // Google: ?text=...
    let google_url = if let Some(t) = &text {
        format!("https://translate.google.com/?sl=auto&tl=zh-CN&text={}", encoded)
    } else {
        "https://translate.google.com/".to_string()
    };

    // 3. Create Google Webview (Rest of the area, default visible)
    let google_webview = window
        .add_child(
            WebviewBuilder::new(
                "translator-google",
                WebviewUrl::External(google_url.parse().unwrap()),
            )
            .auto_resize(),
            LogicalPosition::new(0.0, 50.0),
            LogicalSize::new(1000.0, 750.0),
        )
        .map_err(|e| e.to_string())?;

    // DeepL: #en/zh/text
    // Auto-detect source is tricky with hash, usually #auto/zh/...
    let deepl_url = if let Some(t) = &text {
        format!("https://www.deepl.com/translator#auto/zh/{}", encoded)
    } else {
        "https://www.deepl.com/translator".to_string()
    };

    // 4. Create DeepL Webview (Rest of the area, default hidden)
    let deepl_webview = window
        .add_child(
            WebviewBuilder::new(
                "translator-deepl",
                WebviewUrl::External(deepl_url.parse().unwrap()),
            )
            .auto_resize(),
            LogicalPosition::new(0.0, 50.0),
            LogicalSize::new(1000.0, 750.0),
        )
        .map_err(|e| e.to_string())?;

    // Bing: ?text=...
    let bing_url = if let Some(t) = &text {
        format!("https://cn.bing.com/translator?text={}", encoded)
    } else {
        "https://cn.bing.com/translator".to_string()
    };

    // Bing
    let bing_webview = window
        .add_child(
            WebviewBuilder::new(
                "translator-bing",
                WebviewUrl::External(bing_url.parse().unwrap()),
            )
            .auto_resize(),
            LogicalPosition::new(0.0, 50.0),
            LogicalSize::new(1000.0, 750.0),
        )
        .map_err(|e| e.to_string())?;

    // Baidu: #zh/en/text (or auto/zh)
    // https://fanyi.baidu.com/#auto/zh/text
    let baidu_url = if let Some(t) = &text {
        format!("https://fanyi.baidu.com/#auto/zh/{}", encoded)
    } else {
        "https://fanyi.baidu.com".to_string()
    };

    // Baidu
    let baidu_webview = window
        .add_child(
            WebviewBuilder::new(
                "translator-baidu",
                WebviewUrl::External(baidu_url.parse().unwrap()),
            )
            .auto_resize(),
            LogicalPosition::new(0.0, 50.0),
            LogicalSize::new(1000.0, 750.0),
        )
        .map_err(|e| e.to_string())?;

    // Sougou: ?text=... (need to verify)
    // https://fanyi.sogou.com/text?keyword=...
    let sougou_url = if let Some(t) = &text {
        format!("https://fanyi.sogou.com/text?keyword={}", encoded)
    } else {
        "https://fanyi.sogou.com/".to_string()
    };

    // Sougou
    let sougou_webview = window
        .add_child(
            WebviewBuilder::new(
                "translator-sougou",
                WebviewUrl::External(sougou_url.parse().unwrap()),
            )
            .auto_resize(),
            LogicalPosition::new(0.0, 50.0),
            LogicalSize::new(1000.0, 750.0),
        )
        .map_err(|e| e.to_string())?;

    // Tencent: ?text=... (need to verify)
    // https://fanyi.qq.com/ (usually POST/client side, URL might support ?text=)
    // https://fanyi.qq.com/?text=...
    let tencent_url = if let Some(t) = &text {
        format!("https://fanyi.qq.com/?text={}", encoded)
    } else {
        "https://fanyi.qq.com".to_string()
    };

    // Tencent
    let tencent_webview = window
        .add_child(
            WebviewBuilder::new(
                "translator-tencent",
                WebviewUrl::External(tencent_url.parse().unwrap()),
            )
            .auto_resize(),
            LogicalPosition::new(0.0, 50.0),
            LogicalSize::new(1000.0, 750.0),
        )
        .map_err(|e| e.to_string())?;

    // Caiyun
    // https://fanyi.caiyunapp.com/
    let caiyun_url = "https://fanyi.caiyunapp.com".to_string(); // Not sure about URL param

    // Caiyun
    let caiyun_webview = window
        .add_child(
            WebviewBuilder::new(
                "translator-caiyun",
                WebviewUrl::External(caiyun_url.parse().unwrap()),
            )
            .auto_resize(),
            LogicalPosition::new(0.0, 50.0),
            LogicalSize::new(1000.0, 750.0),
        )
        .map_err(|e| e.to_string())?;

    // Youdao: http://fanyi.youdao.com/
    let youdao_url = "https://fanyi.youdao.com".to_string(); 

    // Youdao
    let youdao_webview = window
        .add_child(
            WebviewBuilder::new(
                "translator-youdao",
                WebviewUrl::External(youdao_url.parse().unwrap()),
            )
            .auto_resize(),
            LogicalPosition::new(0.0, 50.0),
            LogicalSize::new(1000.0, 750.0),
        )
        .map_err(|e| e.to_string())?;

    // Papago: https://papago.naver.com/?sk=auto&tk=zh-CN&st=...
    let papago_url = if let Some(t) = &text {
        format!("https://papago.naver.com/?sk=auto&tk=zh-CN&st={}", encoded)
    } else {
        "https://papago.naver.com/".to_string()
    };

    // Papago
    let papago_webview = window
        .add_child(
            WebviewBuilder::new(
                "translator-papago",
                WebviewUrl::External(papago_url.parse().unwrap()),
            )
            .auto_resize(),
            LogicalPosition::new(0.0, 50.0),
            LogicalSize::new(1000.0, 750.0),
        )
        .map_err(|e| e.to_string())?;

    // Yandex: https://translate.yandex.com/?text=...
    let yandex_url = if let Some(t) = &text {
        format!("https://translate.yandex.com/?text={}", encoded)
    } else {
        "https://translate.yandex.com/".to_string()
    };

    // Yandex
    let yandex_webview = window
        .add_child(
            WebviewBuilder::new(
                "translator-yandex",
                WebviewUrl::External(yandex_url.parse().unwrap()),
            )
            .auto_resize(),
            LogicalPosition::new(0.0, 50.0),
            LogicalSize::new(1000.0, 750.0),
        )
        .map_err(|e| e.to_string())?;

    // Hide others initially
    deepl_webview.hide().map_err(|e| e.to_string())?;
    bing_webview.hide().map_err(|e| e.to_string())?;
    baidu_webview.hide().map_err(|e| e.to_string())?;
    sougou_webview.hide().map_err(|e| e.to_string())?;
    tencent_webview.hide().map_err(|e| e.to_string())?;
    caiyun_webview.hide().map_err(|e| e.to_string())?;
    youdao_webview.hide().map_err(|e| e.to_string())?;
    papago_webview.hide().map_err(|e| e.to_string())?;
    yandex_webview.hide().map_err(|e| e.to_string())?;

    // 5. Listen for switch event
    // The UI webview will emit "translator_switch".
    
    let google_webview_clone = google_webview.clone();
    let deepl_webview_clone = deepl_webview.clone();
    let bing_webview_clone = bing_webview.clone();
    let baidu_webview_clone = baidu_webview.clone();
    let sougou_webview_clone = sougou_webview.clone();
    let tencent_webview_clone = tencent_webview.clone();
    let caiyun_webview_clone = caiyun_webview.clone();
    let youdao_webview_clone = youdao_webview.clone();
    let papago_webview_clone = papago_webview.clone();
    let yandex_webview_clone = yandex_webview.clone();

    window.listen("translator_switch", move |event| {
        let payload = event.payload();
        println!("Received switch event: {}", payload);
        
        // Simple check. ideally parse JSON.
        // Payload format: {"engine":"..."}
        
        let engine = if payload.contains("deepl") {
            "deepl"
        } else if payload.contains("bing") {
            "bing"
        } else if payload.contains("baidu") {
            "baidu"
        } else if payload.contains("sougou") {
            "sougou"
        } else if payload.contains("tencent") {
            "tencent"
        } else if payload.contains("caiyun") {
            "caiyun"
        } else if payload.contains("youdao") {
            "youdao"
        } else if payload.contains("papago") {
            "papago"
        } else if payload.contains("yandex") {
            "yandex"
        } else {
            "google"
        };
        
        // Hide all first
        let _ = google_webview_clone.hide();
        let _ = deepl_webview_clone.hide();
        let _ = bing_webview_clone.hide();
        let _ = baidu_webview_clone.hide();
        let _ = sougou_webview_clone.hide();
        let _ = tencent_webview_clone.hide();
        let _ = caiyun_webview_clone.hide();
        let _ = youdao_webview_clone.hide();
        let _ = papago_webview_clone.hide();
        let _ = yandex_webview_clone.hide();

        match engine {
            "deepl" => {
                let _ = deepl_webview_clone.show();
                let _ = deepl_webview_clone.set_focus();
            }
             "bing" => {
                let _ = bing_webview_clone.show();
                let _ = bing_webview_clone.set_focus();
            }
             "baidu" => {
                let _ = baidu_webview_clone.show();
                let _ = baidu_webview_clone.set_focus();
            }
             "sougou" => {
                let _ = sougou_webview_clone.show();
                let _ = sougou_webview_clone.set_focus();
            }
             "tencent" => {
                let _ = tencent_webview_clone.show();
                let _ = tencent_webview_clone.set_focus();
            }
             "caiyun" => {
                let _ = caiyun_webview_clone.show();
                let _ = caiyun_webview_clone.set_focus();
            }
             "youdao" => {
                let _ = youdao_webview_clone.show();
                let _ = youdao_webview_clone.set_focus();
            }
             "papago" => {
                let _ = papago_webview_clone.show();
                let _ = papago_webview_clone.set_focus();
            }
             "yandex" => {
                let _ = yandex_webview_clone.show();
                let _ = yandex_webview_clone.set_focus();
            }
            _ => { // google
                let _ = google_webview_clone.show();
                let _ = google_webview_clone.set_focus();
            }
        }
    });

    Ok(())
}
