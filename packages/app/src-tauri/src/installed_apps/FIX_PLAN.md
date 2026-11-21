# 应用发现「卸载项」问题修复方案

## 背景 / 已确认的问题

`windows.rs` 的已安装应用发现存在三类缺陷，分别对应三个复现案例：

1. **百度网盘**：出现 `百度网盘` 与 `卸载百度网盘` 两条，都指向 `uninst.exe`。
2. **钉钉**：搜索时只剩一条卸载项，正常启动项消失。
3. **小米电脑管家**：一条干净名字的项指向 `C:\...\Uninstall.exe`。

根因（均已通过源码逐行核对）：

| 缺陷                                    | 位置                                                     | 说明                                                                                                                     |
| --------------------------------------- | -------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------ |
| A. DisplayIcon 分支无卸载检查           | `extract_exe_path` `windows.rs:160-181`                  | 只要 `DisplayIcon` 以 `.exe/.msi/.bat` 结尾就直接作为启动路径返回，`...\Uninstall.exe` 也能通过                          |
| B. UninstallString 回退只认 `uninstall` | `windows.rs:196-209`                                     | `uninst.exe` / `unins000.exe` 不含连续字符串 `uninstall`，守卫失效；还会抓到 `rundll32.exe` / `setup.exe` 等错误 token   |
| C. 快捷方式过滤器只查目标、且模式太窄   | `is_uninstall_path` `windows.rs:714-722`                 | 只检查 `local_base_path()` 目标，从不检查 `.lnk` 自身文件名；`uninst` / `unins000` 不匹配                                |
| D. lnk 目标路径解析丢失后缀             | `extract_target_from_lnk_with_link` `windows.rs:406-415` | 只取 `local_base_path()`，忽略 `common_path_suffix()`，导致「目录 + 文件名」结构的目标被当成目录，随后被扩展名检查丢弃   |
| E. 按名称单键去重压掉好条目             | `get_apps` `windows.rs:650-676`                          | 注册表用 `insert`（后写覆盖）、快捷方式/AppsFolder 用 `or_insert`；坏的注册表项（→ Uninstall.exe）会挡掉路径正确的启动项 |
| F. DisplayIcon 解析能力不足             | `extract_exe_path` `windows.rs:160-181`                  | 不支持带引号路径、`dll,0` 图标索引、带参数路径；`InstallLocation` 只猜 `start.exe`                                       |
| G. 名称归一化不全                       | `normalize_app_name` `windows.rs:126-153`                | 不处理全角括号、不剥离「卸载/Uninstall」，造成重复项或残留卸载前缀                                                       |

## 修复方案（对齐 Flow Launcher / PowerToys 社区最佳实践）

### 1. 统一卸载器识别（双轴：名称 + 路径）

新增纯函数并集中使用，模式覆盖：

- 精确文件名：`uninst.exe`, `unins000.exe`, `uninst000.exe`, `uninstall.exe`
- 前缀：`uninstall`, `uninst`, `unins`, `卸载`, `卸載`（对应 Flow Launcher 的多语言前缀思路）
- 应用于**应用显示名**与**候选 exe 路径的文件名**两个维度。
- `uninst` 作为前缀能同时覆盖 `uninst.exe` 与 `uninstall.exe`（避免只匹配 `uninstall` 的漏网）。

### 2. 修复 `extract_exe_path`

- 所有分支返回路径前统一过 `is_uninstaller_candidate`。
- `DisplayIcon` 支持：去引号、`exe/msi/bat,dll` 索引形式（`path,0`）、带参数路径；去掉 `.ico/.png` 等非 exe 作为**启动路径**的用法（仅用做图标）。
- `UninstallString` 回退：用更稳的正则提取带引号 exe 路径，避免抓 `rundll32.exe`；对 `MsiExec.exe` 直接放弃。
- `InstallLocation` 猜测：除 `start.exe` 外，不再盲猜；保留但同样过滤卸载器。

### 3. 修复快捷方式解析

- `extract_target_from_lnk_with_link`：拼接 `local_base_path()` + `common_path_suffix()`（并用 `*_unicode` 变体兜底处理中文），环境变量展开；若整体不存在则回退到 base（保持兼容）。
- `app_from_shortcut`：`is_uninstall_path` 改为接受 `.lnk` 文件名 + 目标路径两个参数，文件名含卸载前缀或目标为卸载器则丢弃（对齐 Flow `HideUninstallersFilter` 第二轴）。

### 4. 修复注册表扫描

- `get_apps_from_hkey`：对 `DisplayName` / 候选路径统一做卸载器检查，构建 `AppInfo` 前把卸载器过滤掉。
- 保留 `SystemComponent`、`ParentKeyName`、`msiexec.exe` 过滤。

### 5. 修复去重

- 改用**稳定身份**去重：优先以规范化后的 exe 路径（小写）为键；无路径时回退到名称。
- 来源优先级反转为：**AppsFolder > 快捷方式 > 注册表**（AppsFolder 是 Shell 策展的“所有应用”，最可信），并在合并时**合并元数据**（图标缺失时从其他来源补齐），而不是丢弃后来者。
- 这样即使注册表里有坏的 `Uninstall.exe` 项，也不会挡掉 AppsFolder/快捷方式里的正确启动项。

### 6. 名称归一化

- 增加全角括号 `（）` `【】` 处理；保留现有 ASCII 逻辑。

### 7. 跨平台考虑

- 上述缺陷全部在 `installed_apps/windows.rs`（仅 Windows 编译：`#[cfg(target_os = "windows")] mod windows;`）。
- **macOS** (`macos.rs`)：扫描 `/Applications` 等目录下的 `.app` 包，不存在注册表/`.lnk`/Uninstall 概念，无此类问题；检查其 `app_map` 按 `.app` 名去重，无需修改。
- **Linux** (`linux.rs`)：`dpkg --get-selections` 只取包名、`path: None`，无启动路径解析，不受影响。
- 修复代码集中在 Windows 分支，通过 `#[cfg]` 保证不影响其它平台编译；`mod.rs` 中 `AppInfo` 在非 Windows 上 `origin` 被 `serde(skip_serializing)`，保持不变。
- 单元测试里与平台相关的辅助函数用 `#[cfg(test)]` 挂在该文件内，仅 Windows 目标编译；纯逻辑函数（卸载器判断、路径提取、名称归一化）设计为不依赖系统 API，便于测试。

## 验证计划

1. `cargo fmt --manifest-path packages/app/src-tauri/Cargo.toml`
2. `cargo test --manifest-path packages/app/src-tauri/Cargo.toml --lib installed_apps::windows::tests`
3. `cargo check --manifest-path packages/app/src-tauri/Cargo.toml`（Windows 目标）
4. 针对三案例编写单元测试：
   - `uninst.exe` / `unins000.exe` / `Uninstall.exe` 均被识别为卸载器；
   - `卸载钉钉.lnk` 文件名被识别；
   - `DisplayIcon = ...\Uninstall.exe` 不会成为启动路径；
   - `C:\App\app.exe,0` / `"C:\App\app.exe"` 能正确提取；
   - 去重保留正确启动项、名称归一化处理全角括号。

---

# 第二轮修复：机器特定指令的持久化与跨机同步

## 背景 / 复现

第一轮修复（本文件上半部分）修正了 `windows.rs` 的扫描逻辑，单元测试与本机真实扫描均通过（卸载器残留 0 条）。但用户安装新包后反馈两个问题：

1. **装了新版仍显示旧路径**：搜索「小米电脑管家」依旧显示 `C:\...\5.5.2.10\Uninstall.exe`，说明修复后的扫描代码根本没在用户机器上生效。
2. **备份恢复把 A 电脑的程序指令带到 B 电脑**：不同电脑 installed apps 与安装路径不同，恢复后指令路径全错。

## 根因（源码已逐行核对，本质是同一个）

`commands.json` 把机器特定的 `CommandAction::App(path)` / `File(path)` 持久化了：

| 环节               | 位置                                             | 问题                                                                                                                                                                                                                       |
| ------------------ | ------------------------------------------------ | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| 生成               | `generators/app.rs` / `generators/file.rs`       | 扫描本机的绝对路径 / `shell:AppsFolder\AUMID` 写进 `Command.action`                                                                                                                                                        |
| 持久化             | `storage.rs:39,221`                              | `generate_and_save_commands` 与每次 `load` 结尾 `save_commands` 全量写盘                                                                                                                                                   |
| 加载（问题1 主因） | `storage.rs:92-103` `merge_commands`             | `other_commands` 把 Application / FileCommand 原样 `clone` 保留，**从不重新扫描**；注释写明「无需每次加载都重新扫描」。故 `windows.rs` 修复对已有 `commands.json` 的用户永远不生效，且每次 load 又把旧值写回磁盘，永久固化 |
| 同步（问题2 主因） | `sync/mod.rs:114,152` + `sync/zip_utils.rs:9-23` | 备份整目录打包、恢复整目录覆盖，黑名单仅含窗口状态/剪贴板/日志；`commands.json`、`file_commands.json` 原样跨机搬运                                                                                                         |
| 恢复时序           | `sync/mod.rs:159-168` + `setup.rs:202-229`       | 恢复只热重载 `app_config`，不重扫程序、不通知命令列表刷新；开机自动同步还在 `command_manager::init` 之后执行                                                                                                               |

## 修复方案

### 1. Application / FileCommand 改为「权威源实时重建」，不信任持久化路径

- `merge_commands` 不再把 saved 里的 Application / FileCommand 原样保留，而是：
  - Application：用当前扫描结果（`get_initial_app_commands`）**整体替换**路径 / 图标 / origin；
  - FileCommand：用权威源 `file_commands.json`（`get_initial_file_commands`）**整体替换**。
- 用户自定义**保留**：按 `name` 匹配 saved 命令，把用户添加的关键词、`disabled` 状态合并进新命令（默认关键词仍取当前扫描结果）。
- 这样：
  - 问题1：装新版后首次 load 即用新 `windows.rs` 逻辑重建，旧 `Uninstall.exe` 项消失；
  - 问题2：即便备份把旧路径带过来，加载时也被本机扫描 / 本机 `file_commands.json` 覆盖。

### 2. 进程内缓存避免每次 load 重扫（性能）

- `windows.rs` 扫描含图标 Base64 提取，开销大；`get_all_launchable_items` 每次 `fetchApps` 都会 `load_commands`，不能每次重扫。
- 在 `generators/app.rs` 增加进程内缓存（`OnceLock<RwLock<Option<Vec<Command>>>>`）：
  - 首次 load 扫描一次并缓存；后续 load 复用内存结果；
  - `generate_and_save_commands`（手动刷新 / 首次生成）主动失效缓存，强制重扫；
  - 进程重启（用户装新版后启动）缓存自然为空 → 用新代码重新扫描。
- FileCommand 的权威源 `FileCommandManager` 已有内存缓存，`get_initial_file_commands` 足够快，无需额外缓存。

### 3. 同步侧：剥离机器特定数据

- `sync/zip_utils.rs` 黑名单加入 `file_commands.json`（用户手动添加的本地文件/程序路径，机器特定，不应跨机）。
- `commands.json` 继续同步（保留系统/插件/扩展命令的关键词配置等可移植数据），但其中 Application / FileCommand 项在加载时被第 1 步重建覆盖，不会残留错误路径。
- `sync/mod.rs` 恢复完成后 emit `apps_updated`，触发前端重新 `fetchApps`，使恢复即时生效（含开机自动同步）。

### 4. 跨平台考虑

- Application / FileCommand 的权威源生成函数（`get_initial_app_commands` / `get_initial_file_commands`）均为平台无关封装：
  - Windows 走修复后的 `windows.rs`；macOS 走 `macos.rs`（`.app` 包）；Linux 走 `linux.rs`（`dpkg`，`path: None`，几乎无 Application 命令）。
- 缓存与关键词合并逻辑放在 `command_manager`，纯 Rust，不依赖平台 API。
- 同步黑名单为路径字符串匹配，平台无关；`file_commands.json` 在三个平台都是用户本地文件指令，跨机不同，统一排除。
- 单元测试覆盖：缓存重建、关键词合并保留用户自定义、黑名单包含 `file_commands.json`。

## 第二轮验证计划

1. `cargo fmt --manifest-path packages/app/src-tauri/Cargo.toml`
2. `cargo test --manifest-path packages/app/src-tauri/Cargo.toml --lib`（全部单测，含新增）
3. 真实场景：构造含旧 `Uninstall.exe` 路径的 `commands.json`，验证 load 后该路径被本机扫描结果替换；
4. 重新打包 NSIS 安装包供用户安装验证两个复现案例。

---

# 第三轮：代码审查 findings 核实与修复

对两轴代码审查报告的逐项核实结果（对照真实代码 + 本机实际数据）：

## Spec 轴

| finding                                             | 结论                                                                                                                                                                                                                                                           | 处理                                                                                                                                                                                                   |
| --------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| 去重键未按 exe 路径（规范第52行）                   | **误报**。规范文字是"优先 exe 路径为键"，但同一应用在三来源里路径本就不同（注册表 exe / 快捷方式 exe / AppsFolder AUMID），按路径为键会导致同一应用出现多条；名称键 + 来源优先级合并才是正确设计，且修复前 252 条应用全部 name 唯一                            | 不修改；在代码注释中已说明按名称去重+合并元数据的理由                                                                                                                                                  |
| 第二轮验证项未全链路覆盖                            | **属实**                                                                                                                                                                                                                                                       | 新增 `rebuild_machine_specific_commands` 纯函数并配套端到端测试 `rebuild_replaces_stale_uninstaller_path_and_keeps_user_keywords`（旧 commands.json 的 Uninstall.exe 被正确路径替换 + 用户关键词保留） |
| NON_APP_EXECUTABLES 全局过滤 setup.exe 等属范围蔓延 | **属实**                                                                                                                                                                                                                                                       | 拆分为 `NON_APP_EXECUTABLES`（系统执行器，全局过滤）与 `INSTALLER_EXECUTABLES`（仅 UninstallString 兜底排除）；新增测试验证合法 setup.exe 作为 DisplayIcon 被保留                                      |
| AppsFolder 无条件覆盖真实 exe 路径                  | **部分属实但非缺陷**。spec 明确规定优先级 AppsFolder > Shortcut > Registry；且本机真实数据显示小米电脑管家正是靠 AppsFolder 的 AUMID 才得到正确启动路径（`shell:AppsFolder\{GUID}\...\XiaomiPcManager.exe`），改成 Shortcut 优先反而会回退到旧的 Uninstall.exe | 不修改（符合 spec 且经真实数据验证）                                                                                                                                                                   |
| 用户关键词自定义可能丢失（名称归一化前后不一致）    | **属实**                                                                                                                                                                                                                                                       | 新增 `normalize_app_name_for_match`（installed_apps/mod.rs）+ `normalize_saved_app_name`（storage.rs），saved 命令按归一化名匹配；新增端到端测试覆盖"名字带全角括号的旧命令仍保留用户关键词"           |

## Standards 轴（均为 judgement call）

| finding                                                             | 结论             | 处理                                                                                                                                                                                |
| ------------------------------------------------------------------- | ---------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `looks_like_uninstaller` / `is_non_app_executable` 取文件名骨架重复 | **属实**         | 抽取 `file_name_of(path)` 共用                                                                                                                                                      |
| `!looks_like_uninstaller && !is_non_app_executable` 在 7 处重复     | **属实**         | 抽取 `is_launchable_target(path)`，全部调用点替换                                                                                                                                   |
| `saved_app_map` / `saved_file_map` 两条平行流水线重复               | **属实**         | 抽取 `rebuild_machine_specific_commands` 纯函数统一处理                                                                                                                             |
| `other_commands` 的 `!=` 级联与新增 map 需同步改 source             | **属实（轻微）** | 第二轮复核发现首次处理未真正落地；已抽取 `is_machine_specific_source()` + `merged_from_current_sources` 常量数组，机器特定分类单点定义，`other_commands` 过滤与重建函数共用同一语义 |
| `origin_rank` 用 u8 魔数                                            | 轻微             | 保留（局部函数，语义清晰，改动收益低）                                                                                                                                              |
| ItemSource `!=` 长级联                                              | 轻微             | 已改为常量数组 + `is_machine_specific_source()`，不再有长级联                                                                                                                       |

## 第三轮验证

- `cargo fmt`：通过
- `cargo test --lib`：**196 passed; 0 failed**（较第二轮 +4：安装器范围、启动目标判断、端到端重建、来源分类）
- 真实数据核验：本机 `commands.json`（20:13）卸载器残留 0，小米电脑管家路径为 `shell:AppsFolder\...\XiaomiPcManager.exe`

## 第四轮（二次代码审查复核）

- Standards 复核发现两处残留：`windows.rs` 去重最后防线漏用 `is_launchable_target`、`other_commands` 的 source `!=` 级联未真正集中。
- 已修复：`windows.rs:889` 改用 `is_launchable_target`；`storage.rs` 新增 `is_machine_specific_source()` 与 `merged_from_current_sources` 常量数组，消除 source 长级联。
- Spec 复核：五项 adjudication 全部成立，无新功能缺陷；文件级 JSON→load→回写集成测试为 judgement call（核心行为已由纯函数端到端测试覆盖）。
- `cargo fmt` + `cargo test --lib`：**196 passed; 0 failed**。

## 第五轮（三次代码审查复核）

第三次两轴审查结果：

- Spec 轴：核心行为全部正确，仅指出端到端测试用全角括号名在非 Windows 上会失败（该归一化仅 Windows 具备）。
- Standards 轴：6 项历史问题全部修复；新增 1 处轻微重复（`is_non_app_executable` / `is_installer_executable` 同构）。

已修复：

- `windows.rs` 抽取 `file_name_in(path, names)`，两个判断函数共用。
- `storage.rs` 端到端测试拆为两部分：平台无关的「旧卸载器路径替换 + 关键词保留」测试（全平台运行），以及 `#[cfg(target_os = "windows")]` 限定的「名称归一化后保留关键词」测试（仅 Windows）。

验证：`cargo fmt` 通过；`cargo test --lib` **197 passed; 0 failed**。

## 第六轮（最终确认）

第四次审查 Standards 轴指出最后一处轻微重复：`app_from_shortcut` 中快捷方式目标被 `extract_target_from_lnk_with_link` 解析两次（一次在 `should_filter_shortcut` 内、一次在构造 AppInfo 时）。

已修复：`app_from_shortcut` 只解析一次目标，`target_path.as_deref()` 借给 `should_filter_shortcut`，随后消费所有权构造 AppInfo；`should_filter_shortcut` 的 local_base_path 兜底分支仅在该解析返回 None 时执行，保持可达。

第五次确认审查结论：**两轴全部干净，无遗留问题**（helpers、重建函数、source 分类、执行器拆分、名称归一化、平台门控测试、Spec 行为全部成立）。`cargo fmt` 通过；`cargo test --lib` 197 passed / 0 failed。
