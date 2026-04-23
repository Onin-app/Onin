use crate::extension::registry::Extension;
use crate::extension::types::{
    ExtensionCommand, ExtensionManifest, ExtensionPreview, ExtensionResult, ExtensionResultType,
    StaticCommandMatch,
};
use url::Url;

pub fn init(_app: &tauri::AppHandle) {}

pub static WEB_MANIFEST: ExtensionManifest = ExtensionManifest {
    id: "web",
    name: "网页搜索/打开链接",
    description: "打开网址，或用搜索引擎搜索文本",
    icon: "globeSimple",
    commands: &[
        ExtensionCommand {
            code: "open_url",
            keywords: &["web", "url", "open", "link", "网址", "打开"],
            matches: Some(&[StaticCommandMatch {
                match_type: "text",
                name: "网址",
                description: "检测网址并直接打开",
                regexp: Some(
                    r"^(https?:\/\/)?((localhost)|(([a-zA-Z0-9-]+\.)+[a-zA-Z]{2,})|(\d{1,3}(\.\d{1,3}){3}))(:\d+)?(\/\S*)?$",
                ),
                min: Some(1),
                max: None,
            }]),
        },
        ExtensionCommand {
            code: "search_google",
            keywords: &["google", "搜索", "search"],
            matches: Some(&[StaticCommandMatch {
                match_type: "text",
                name: "普通文本",
                description: "普通文本使用 Google 搜索",
                regexp: Some(
                    r"^(?!(https?:\/\/)?((localhost)|(([a-zA-Z0-9-]+\.)+[a-zA-Z]{2,})|(\d{1,3}(\.\d{1,3}){3}))(:\d+)?(\/\S*)?$).+",
                ),
                min: Some(1),
                max: None,
            }]),
        },
        ExtensionCommand {
            code: "search_bing",
            keywords: &["bing", "搜索", "search"],
            matches: Some(&[StaticCommandMatch {
                match_type: "text",
                name: "普通文本",
                description: "普通文本使用 Bing 搜索",
                regexp: Some(
                    r"^(?!(https?:\/\/)?((localhost)|(([a-zA-Z0-9-]+\.)+[a-zA-Z]{2,})|(\d{1,3}(\.\d{1,3}){3}))(:\d+)?(\/\S*)?$).+",
                ),
                min: Some(1),
                max: None,
            }]),
        },
        ExtensionCommand {
            code: "search_baidu",
            keywords: &["baidu", "百度", "搜索", "search"],
            matches: Some(&[StaticCommandMatch {
                match_type: "text",
                name: "普通文本",
                description: "普通文本使用百度搜索",
                regexp: Some(
                    r"^(?!(https?:\/\/)?((localhost)|(([a-zA-Z0-9-]+\.)+[a-zA-Z]{2,})|(\d{1,3}(\.\d{1,3}){3}))(:\d+)?(\/\S*)?$).+",
                ),
                min: Some(1),
                max: None,
            }]),
        },
    ],
};

pub struct WebExtension;

pub static WEB_EXTENSION: WebExtension = WebExtension;

impl Extension for WebExtension {
    fn manifest(&self) -> &'static ExtensionManifest {
        &WEB_MANIFEST
    }

    fn execute(&self, input: &str) -> ExtensionResult {
        self.execute_command("open_url", input)
    }

    fn execute_command(&self, command_code: &str, input: &str) -> ExtensionResult {
        let trimmed = input.trim();
        let target = match command_code {
            "open_url" => {
                let Some(url) = normalize_url(trimmed) else {
                    return ExtensionResult::error("无法识别网址".to_string());
                };
                url
            }
            "search_google" => build_search_url("google", trimmed),
            "search_bing" => build_search_url("bing", trimmed),
            "search_baidu" => build_search_url("baidu", trimmed),
            _ => return ExtensionResult::error(format!("未知命令: {}", command_code)),
        };

        match opener::open(&target) {
            Ok(_) => ExtensionResult {
                success: true,
                value: Some("已在默认浏览器中打开".to_string()),
                result_type: ExtensionResultType::Conversion,
                copyable: None,
                subtitle: None,
                error: None,
            },
            Err(err) => ExtensionResult::error(format!("打开失败: {}", err)),
        }
    }

    fn preview(&self, _input: &str) -> Option<ExtensionPreview> {
        None
    }
}

fn normalize_url(input: &str) -> Option<String> {
    if input.contains(char::is_whitespace) {
        return None;
    }

    if let Ok(url) = Url::parse(input) {
        if matches!(url.scheme(), "http" | "https") && url.host_str().is_some() {
            return Some(url.to_string());
        }
    }

    let candidate = format!("https://{}", input);
    if let Ok(url) = Url::parse(&candidate) {
        if is_probable_host(input, &url) {
            return Some(url.to_string());
        }
    }

    None
}

fn is_probable_host(raw: &str, url: &Url) -> bool {
    let Some(host) = url.host_str() else {
        return false;
    };

    host == "localhost" || host.parse::<std::net::IpAddr>().is_ok() || raw.contains('.')
}

fn build_search_url(provider: &str, query: &str) -> String {
    let encoded: String = url::form_urlencoded::byte_serialize(query.as_bytes()).collect();
    match provider {
        "google" => format!("https://www.google.com/search?q={}", encoded),
        "bing" => format!("https://www.bing.com/search?q={}", encoded),
        "baidu" => format!("https://www.baidu.com/s?wd={}", encoded),
        _ => format!("https://www.google.com/search?q={}", encoded),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_http_url() {
        let url = normalize_url("https://example.com/path").unwrap();
        assert_eq!(url, "https://example.com/path".to_string());
    }

    #[test]
    fn detects_bare_domain() {
        let url = normalize_url("github.com/openai").unwrap();
        assert_eq!(url, "https://github.com/openai".to_string());
    }

    #[test]
    fn builds_google_search_url() {
        let url = build_search_url("google", "rust tauri");
        assert_eq!(
            url,
            "https://www.google.com/search?q=rust+tauri".to_string()
        );
    }

    #[test]
    fn ignores_plain_text_as_url() {
        assert!(normalize_url("rust tauri").is_none());
    }
}
