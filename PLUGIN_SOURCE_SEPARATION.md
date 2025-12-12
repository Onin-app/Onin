# 插件安装来源分离方案

## 目标

支持同一插件的本地开发版本和市场版本共存，通过目录名后缀区分。

## 实现方案

### 1. 目录命名规则

```
plugins/
├── web-translate.20251202@local/     # 本地导入（符号链接）
└── web-translate.20251202@market/    # 市场下载（实际文件）
```

### 2. 数据结构变更

#### Rust 端

```rust
// 新增枚举
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum InstallSource {
    #[serde(rename = "local")]
    Local,
    #[serde(rename = "marketplace")]
    Marketplace,
}

// 修改 LoadedPlugin
pub struct LoadedPlugin {
    pub manifest: PluginManifest,
    pub dir_name: String,  // 包含后缀，如 "web-translate.20251202@local"
    pub enabled: bool,
    pub settings: Option<PluginSettingsSchema>,
    pub install_source: InstallSource,  // 新增
}

// 辅助函数
fn parse_plugin_dir_name(dir_name: &str) -> (String, InstallSource);
fn make_plugin_dir_name(plugin_id: &str, source: InstallSource) -> String;
```

### 3. 需要修改的函数

#### 3.1 `load_plugins_internal`
- 解析目录名，提取 plugin_id 和 install_source
- 设置 LoadedPlugin 的 install_source 字段

#### 3.2 `import_plugin`
- 使用 `make_plugin_dir_name(plugin_id, InstallSource::Local)` 生成目录名
- 创建符号链接时使用带后缀的目录名
- 设置 install_source 为 Local

#### 3.3 `download_and_install_plugin`
- 使用 `make_plugin_dir_name(plugin_id, InstallSource::Marketplace)` 生成目录名
- **移除 plugin_id 匹配验证**（使用 manifest 中的真实 ID）
- 检查市场版本是否已存在
- 设置 install_source 为 Marketplace

#### 3.4 `uninstall_plugin`
- 支持删除带后缀的目录

### 4. 修改步骤

1. ✅ 添加 InstallSource 枚举
2. ✅ 修改 LoadedPlugin 结构
3. ✅ 添加辅助函数
4. ✅ 修改 load_plugins_internal
5. ✅ 修改 import_plugin
6. ✅ 添加 download_and_install_plugin
7. ✅ 注册 Tauri command
8. ⏳ 测试

### 5. 关键代码片段

#### 辅助函数实现

```rust
fn parse_plugin_dir_name(dir_name: &str) -> (String, InstallSource) {
    if let Some(at_pos) = dir_name.rfind('@') {
        let plugin_id = dir_name[..at_pos].to_string();
        let suffix = &dir_name[at_pos + 1..];
        
        let source = match suffix {
            "market" | "marketplace" => InstallSource::Marketplace,
            "local" => InstallSource::Local,
            _ => InstallSource::Local,
        };
        
        (plugin_id, source)
    } else {
        (dir_name.to_string(), InstallSource::Local)
    }
}

fn make_plugin_dir_name(plugin_id: &str, source: InstallSource) -> String {
    match source {
        InstallSource::Local => format!("{}@local", plugin_id),
        InstallSource::Marketplace => format!("{}@market", plugin_id),
    }
}
```

### 6. 注意事项

- 设置文件仍使用不带后缀的 plugin_id，两个版本共享设置
- 同一 plugin_id 的多个版本可以共存，但只能启用一个
- UI 需要显示来源标识
- 卸载时需要明确指定 dir_name（包含后缀）

### 7. 测试场景

1. 本地导入插件 → 目录名应为 `{id}@local`
2. 市场下载插件 → 目录名应为 `{id}@market`
3. 同时存在两个版本 → 都能正常加载
4. 卸载指定版本 → 只删除对应目录
5. 设置共享 → 两个版本使用相同设置

## 当前状态

✅ 已完成实施

### Store 结构
- Key: `dir_name`（包含后缀，如 `plugin-id@local`）
- Value: `LoadedPlugin`

### 启用互斥逻辑
- 同一 `manifest.id` 的多个版本可以共存
- 但同时只能启用一个
- 启用一个版本时，自动禁用其他版本

### UI 显示
- 两个版本都会显示在插件列表中
- 需要在卡片上添加来源标识（本地/市场）
- 需要显示哪个版本当前启用
