//! # 插件桥接模块
//!
//! 负责插件与 Tauri 应用之间的通信桥接，包括：
//! - HTML 资源内联（CSS/JS）
//! - Tauri API 桥接脚本注入
//! - 窗口顶栏模板

use std::path::Path;

// ============================================================================
// 模板常量 - 已移除 (使用原生窗口装饰)
// ============================================================================

// ============================================================================
// 资源内联函数
// ============================================================================

/// 将 CSS 和 JS 资源内联到 HTML 内容中
///
/// 用于在数据 URL 模式下加载插件，避免外部资源加载问题
///
/// # 参数
/// - `html_content`: 原始 HTML 内容
/// - `html_dir`: HTML 文件所在目录，用于解析相对路径
///
/// # 返回
/// 资源已内联的 HTML 内容
pub fn inline_resources(html_content: &str, html_dir: &Path) -> String {
    let mut modified_html = html_content.to_string();

    // 内联 CSS 文件
    let css_regex =
        regex::Regex::new(r#"<link[^>]+href\s*=\s*["']([^"']+\.css)["'][^>]*>"#).unwrap();
    let css_matches: Vec<_> = css_regex.captures_iter(html_content).collect();

    for cap in css_matches {
        if let Some(css_path_match) = cap.get(1) {
            let css_path = css_path_match.as_str();
            let normalized_path = css_path.replace("/", std::path::MAIN_SEPARATOR_STR);
            let css_file_path = html_dir.join(
                normalized_path
                    .trim_start_matches("./")
                    .trim_start_matches(std::path::MAIN_SEPARATOR),
            );

            if let Ok(css_content) = std::fs::read_to_string(&css_file_path) {
                let inline_style = format!("<style>{}</style>", css_content);
                let original_tag = cap.get(0).unwrap().as_str();
                modified_html = modified_html.replace(original_tag, &inline_style);
            } else {
                eprintln!(
                    "[plugin/bridge] 警告: 读取 CSS 文件失败: {:?}",
                    css_file_path
                );
            }
        }
    }

    // 内联 JS 文件
    let js_regex =
        regex::Regex::new(r#"<script[^>]+src\s*=\s*["']([^"']+\.js)["'][^>]*></script>"#).unwrap();
    let js_matches: Vec<_> = js_regex.captures_iter(html_content).collect();

    for cap in js_matches {
        if let Some(js_path_match) = cap.get(1) {
            let js_path = js_path_match.as_str();
            let normalized_path = js_path.replace("/", std::path::MAIN_SEPARATOR_STR);
            let js_file_path = html_dir.join(
                normalized_path
                    .trim_start_matches("./")
                    .trim_start_matches(std::path::MAIN_SEPARATOR),
            );

            if let Ok(js_content) = std::fs::read_to_string(&js_file_path) {
                let original_tag = cap.get(0).unwrap().as_str();
                let is_module = original_tag.contains("type=\"module\"")
                    || original_tag.contains("type='module'");

                let inline_script = if is_module {
                    format!("<script type=\"module\">{}</script>", js_content)
                } else {
                    format!("<script>{}</script>", js_content)
                };

                modified_html = modified_html.replace(original_tag, &inline_script);
            } else {
                eprintln!("[plugin/bridge] 警告: 读取 JS 文件失败: {:?}", js_file_path);
            }
        }
    }

    modified_html
}

// ============================================================================
// Tauri API 桥接
// ============================================================================

/// 向 HTML 注入 Tauri API 桥接脚本
///
/// 为内嵌插件提供与主应用通信的能力
///
/// # 参数
/// - `html`: 原始 HTML 内容
/// - `plugin_id`: 插件唯一标识符
///
/// # 返回
/// 已注入桥接脚本的 HTML 内容
pub fn inject_tauri_bridge(html: &str, plugin_id: &str) -> String {
    let tauri_init_script = format!(
        r#"
<script>
(function() {{
  console.log('[Plugin Inline] 正在初始化 Tauri API 桥接');
  
  // 在全局上下文中设置插件 ID (保留兼容性)
  window.__PLUGIN_ID__ = '{}';
  
  // 注入运行时环境信息
  window.__ONIN_RUNTIME__ = {{
    mode: 'inline',
    pluginId: '{}',
    version: '0.1.0',
    mainWindowLabel: 'main'
  }};
  
  const createProxy = (command) => {{
    return (...args) => {{
      return new Promise((resolve, reject) => {{
        const messageId = 'tauri_' + Math.random().toString(36).substring(7) + '_' + Date.now();
        
        const handleResponse = (event) => {{
          if (event.data && event.data.messageId === messageId) {{
            window.removeEventListener('message', handleResponse);
            if (event.data.error) {{
              reject(new Error(event.data.error));
            }} else {{
              resolve(event.data.result);
            }}
          }}
        }};
        
        window.addEventListener('message', handleResponse);
        
        window.parent.postMessage({{
          type: 'plugin-tauri-call',
          messageId,
          command,
          args
        }}, '*');
        
        setTimeout(() => {{
          window.removeEventListener('message', handleResponse);
          reject(new Error('Tauri call timeout'));
        }}, 30000);
      }});
    }};
  }};
  
  const invokeProxy = createProxy('invoke');
  
  window.__TAURI__ = {{
    core: {{ invoke: invokeProxy }},
    event: {{
      emit: createProxy('emit'),
      listen: createProxy('listen')
    }},
    invoke: invokeProxy
  }};
  
  window.__TAURI_INVOKE__ = invokeProxy;
  
  console.log('[Plugin Inline] Tauri API 桥接就绪, mode: inline');
}})();
</script>
"#,
        plugin_id, plugin_id
    );

    if html.contains("<head>") {
        html.replace("<head>", &format!("<head>{}", tauri_init_script))
    } else if html.contains("<html>") {
        html.replace(
            "<html>",
            &format!("<html><head>{}</head>", tauri_init_script),
        )
    } else {
        format!("{}{}", tauri_init_script, html)
    }
}

/// 修复 HTML 中的绝对路径为相对路径
///
/// Vite 构建的插件使用绝对路径（如 /assets/...），但在我们的插件服务器中：
/// - 插件 HTML 的 URL 是：http://127.0.0.1:3457/plugin/plugin-id/dist/index.html
/// - 如果 HTML 中引用 /assets/style.css，浏览器会解析为 http://127.0.0.1:3457/assets/style.css（错误）
/// - 实际文件路径应该是：http://127.0.0.1:3457/plugin/plugin-id/dist/assets/style.css
///
/// 因此需要将绝对路径 / 转换为相对路径 ./，让浏览器相对于 HTML 文件所在目录解析资源
pub fn fix_asset_paths(html: &str) -> String {
    html.replace("=\"/", "=\"./").replace("='/", "='./")
}
