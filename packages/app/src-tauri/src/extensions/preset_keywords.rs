use std::collections::HashMap;
use std::sync::LazyLock;

/// 所有预设的额外指令别名（排除了指令自身名称与拼音别名）
pub static PRESET_KEYWORDS: LazyLock<HashMap<&'static str, &'static [&'static str]>> =
    LazyLock::new(|| {
        let mut m = HashMap::new();
        // ============================================================================
        // 系统命令
        // ============================================================================
        m.insert("shutdown", &["shutdown"][..]);
        m.insert("reboot", &["reboot", "restart"][..]);
        m.insert("sleep", &["sleep"][..]);
        m.insert("lock_screen", &["lock"][..]);
        m.insert("logout", &["logout"][..]);
        m.insert("open_app_data_dir", &["app data"][..]);
        m.insert("refresh_list", &["refresh"][..]);

        // ============================================================================
        // 内置扩展
        // ============================================================================
        m.insert("extension:ai:chat", &["ai", "chat"][..]);
        m.insert("extension:ai:action", &["ai", "action"][..]);
        m.insert("extension:bookmarks:search", &["bookmarks", "bookmark"][..]);
        m.insert(
            "extension:calculator:calculate",
            &["calc", "calculator"][..],
        );
        m.insert("extension:clipboard:history", &["clipboard", "剪贴板"][..]);
        m.insert(
            "extension:color:convert",
            &[
                "color", "colour", "色值", "hex", "rgb", "rgba", "hsl", "hsla",
            ][..],
        );
        m.insert(
            "extension:color:pick",
            &["color", "colour", "拾色", "picker", "pick color"][..],
        );
        m.insert("extension:emoji:search", &["emoticon", "emoji"][..]);
        m.insert(
            "extension:file_search:search",
            &["file", "search", "本地搜索", "find"][..],
        );
        m.insert(
            "extension:translator:open",
            &["translate", "translator"][..],
        );
        m.insert("extension:web:open_url", &["web", "url"][..]);
        m.insert("extension:web:search_google", &["web", "google"][..]);
        m.insert("extension:web:search_bing", &["web", "bing"][..]);
        m.insert("extension:web:search_baidu", &["web", "baidu"][..]);
        m
    });
