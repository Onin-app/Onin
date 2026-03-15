use super::*;

#[test]
fn test_parse_plugin_dir_name() {
    // 无后缀：默认为 Marketplace
    let (id, source) = parse_plugin_dir_name("com.pomodoro.timer");
    assert_eq!(id, "com.pomodoro.timer");
    assert_eq!(source, InstallSource::Marketplace);

    // 本地插件：必须带 @local 后缀
    let (id, source) = parse_plugin_dir_name("plugin-demo@local");
    assert_eq!(id, "plugin-demo");
    assert_eq!(source, InstallSource::Local);

    // 以前带 @market 的，现在不识别后缀，则整体视为 ID
    let (id, source) = parse_plugin_dir_name("byper.web-translate.onin@market");
    assert_eq!(id, "byper.web-translate.onin@market");
    assert_eq!(source, InstallSource::Marketplace);
}

#[test]
fn test_make_plugin_dir_name() {
    // Marketplace：不带后缀
    assert_eq!(
        make_plugin_dir_name("com.pomodoro.timer", InstallSource::Marketplace),
        "com.pomodoro.timer".to_string()
    );

    // Local：带 @local 后缀
    assert_eq!(
        make_plugin_dir_name("plugin-demo", InstallSource::Local),
        "plugin-demo@local".to_string()
    );
}
