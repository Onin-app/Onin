use crate::command_manager;
use crate::installed_apps;
use crate::shared_types::{Command, CommandAction};
use tauri::{async_runtime, command, AppHandle, Manager};

// ============================================================================
// 系统命令定义
// ============================================================================

/// 系统命令信息
pub struct SystemCommandInfo {
    pub name: &'static str,
    pub title: &'static str,
    pub description: &'static str,
    pub english_name: &'static str,
    pub keywords: &'static [&'static str],
    pub icon: &'static str,
    pub action: fn(AppHandle),
    pub requires_confirmation: bool,
}

/// 所有系统命令列表
pub static SYSTEM_COMMANDS: &[SystemCommandInfo] = &[
    SystemCommandInfo {
        name: "shutdown",
        title: "关机",
        description: "关闭计算机",
        english_name: "Shutdown",
        keywords: &["shutdown", "关机"],
        icon: "shutdown",
        action: |_| shutdown(),
        requires_confirmation: true,
    },
    SystemCommandInfo {
        name: "reboot",
        title: "重启",
        description: "重新启动计算机",
        english_name: "Restart",
        keywords: &["restart", "reboot", "重启"],
        icon: "restart",
        action: |_| reboot(),
        requires_confirmation: true,
    },
    SystemCommandInfo {
        name: "sleep",
        title: "睡眠",
        description: "使计算机进入睡眠模式",
        english_name: "Sleep",
        keywords: &["sleep", "睡眠"],
        icon: "sleep",
        action: |_| sleep(),
        requires_confirmation: false,
    },
    SystemCommandInfo {
        name: "lock_screen",
        title: "锁屏",
        description: "锁定计算机屏幕",
        english_name: "Lock Screen",
        keywords: &["lock", "锁屏"],
        icon: "lock",
        action: |_| lock_screen(),
        requires_confirmation: false,
    },
    SystemCommandInfo {
        name: "logout",
        title: "注销",
        description: "注销当前用户",
        english_name: "Logout",
        keywords: &["logout", "注销"],
        icon: "logout",
        action: |_| logout(),
        requires_confirmation: true,
    },
    SystemCommandInfo {
        name: "open_app_data_dir",
        title: "打开应用数据目录",
        description: "打开应用程序的数据存储目录",
        english_name: "Open App Data Directory",
        keywords: &["数据目录"],
        icon: "folder",
        action: open_app_data_dir,
        requires_confirmation: false,
    },
    SystemCommandInfo {
        name: "refresh_list",
        title: "刷新列表",
        description: "刷新应用和命令列表",
        english_name: "Refresh List",
        keywords: &["refresh", "刷新"],
        icon: "arrowsClockwise",
        action: refresh_list,
        requires_confirmation: false,
    },
];

// ============================================================================
// Tauri 命令
// ============================================================================

#[command]
pub async fn get_basic_commands(app: AppHandle) -> Vec<Command> {
    command_manager::load_commands(&app).await
}

#[command]
pub async fn execute_command(
    name: String,
    app: AppHandle,
    args: Option<serde_json::Value>,
) {
    // 记录使用情况
    let tracker_state = app.state::<crate::usage_tracker::UsageTrackerState>();
    if let Err(e) =
        crate::usage_tracker::record_command_usage(app.clone(), tracker_state, name.clone())
    {
        eprintln!("Failed to record command usage: {}", e);
    }

    let commands = command_manager::load_commands(&app).await;
    if let Some(command) = commands.iter().find(|cmd| cmd.name == name) {
        match &command.action {
            CommandAction::System(sys_cmd_name) => {
                if let Some(cmd_info) = SYSTEM_COMMANDS.iter().find(|&cmd| cmd.name == sys_cmd_name)
                {
                    (cmd_info.action)(app);
                } else {
                    eprintln!("Unknown system command: {}", sys_cmd_name);
                }
            }
            CommandAction::App(path) => {
                if let Err(e) = installed_apps::open_app(path.clone(), app.clone()) {
                    eprintln!("Failed to open app {}: {}", path, e);
                }
            }
            CommandAction::File(path) => {
                if let Err(e) = opener::open(path) {
                    eprintln!("Failed to open file {}: {}", path, e);
                }
            }
            CommandAction::Plugin(plugin_id) => {
                let plugin_store = app.state::<crate::plugin::PluginStore>();
                if let Err(e) =
                    crate::plugin::execute_plugin_entry(app.clone(), plugin_store, plugin_id.clone())
                {
                    eprintln!("Failed to execute plugin {}: {}", plugin_id, e);
                }
            }
            CommandAction::PluginCommand {
                plugin_id,
                command_code,
            } => {
                use crate::plugin_api::command::execute_plugin_command;
                let plugin_store = app.state::<crate::plugin::PluginStore>();
                let execution_store =
                    app.state::<crate::plugin_api::command::CommandExecutionStore>();

                match execute_plugin_command(
                    app.clone(),
                    plugin_store,
                    execution_store,
                    plugin_id.clone(),
                    command_code.clone(),
                    args,
                )
                .await
                {
                    Ok(result) => {
                        if !result.success {
                            eprintln!("Plugin command failed: {:?}", result.error);
                        }
                    }
                    Err(e) => {
                        eprintln!("Failed to execute plugin command {}: {}", command_code, e);
                    }
                }
            }
            CommandAction::Extension {
                extension_id,
                command_code: _command_code,
            } => {
                tracing::info!(
                    "Extension command triggered: {}",
                    extension_id
                );
                
                let result = crate::extension::execute_extension_command(extension_id, "");
                if let Some(error) = result.error {
                     eprintln!("Extension command failed: {}", error);
                }
            }
        }
    } else {
        eprintln!("Command not found: {}", name);
    }
}

// ============================================================================
// 跨平台命令执行
// ============================================================================

/// 跨平台系统命令配置
struct PlatformCommand<'a> {
    log_message: &'static str,
    windows: Option<(&'static str, &'a [&'a str])>,
    macos: Option<(&'static str, &'a [&'a str])>,
    linux: Option<(&'static str, &'a [&'a str])>,
}

/// 执行跨平台系统命令
fn execute_platform_command(cmd: &PlatformCommand) {
    println!("{}", cmd.log_message);

    #[cfg(target_os = "windows")]
    if let Some((program, args)) = cmd.windows {
        if let Err(e) = std::process::Command::new(program).args(args).output() {
            eprintln!("Failed to execute command on Windows: {}", e);
        }
    }

    #[cfg(target_os = "macos")]
    if let Some((program, args)) = cmd.macos {
        if let Err(e) = std::process::Command::new(program).args(args).output() {
            eprintln!("Failed to execute command on macOS: {}", e);
        }
    }

    #[cfg(target_os = "linux")]
    if let Some((program, args)) = cmd.linux {
        if let Err(e) = std::process::Command::new(program).args(args).output() {
            eprintln!("Failed to execute command on Linux: {}", e);
        }
    }
}

// ============================================================================
// 系统命令实现
// ============================================================================

fn shutdown() {
    execute_platform_command(&PlatformCommand {
        log_message: "System shutdown initiated",
        windows: Some(("shutdown", &["/s", "/t", "0"])),
        macos: Some(("osascript", &["-e", "tell app \"System Events\" to shut down"])),
        linux: Some(("shutdown", &["now"])),
    });
}

fn reboot() {
    execute_platform_command(&PlatformCommand {
        log_message: "System reboot initiated",
        windows: Some(("shutdown", &["/r", "/t", "0"])),
        macos: Some(("osascript", &["-e", "tell app \"System Events\" to restart"])),
        linux: Some(("reboot", &[])),
    });
}

fn sleep() {
    execute_platform_command(&PlatformCommand {
        log_message: "System sleep initiated",
        windows: Some(("rundll32.exe", &["powrprof.dll,SetSuspendState", "0", "1", "0"])),
        macos: Some(("pmset", &["sleepnow"])),
        linux: Some(("systemctl", &["suspend"])),
    });
}

fn lock_screen() {
    execute_platform_command(&PlatformCommand {
        log_message: "Screen lock initiated",
        windows: Some(("rundll32.exe", &["user32.dll,LockWorkStation"])),
        macos: Some(("pmset", &["displaysleepnow"])),
        linux: Some(("xdg-screensaver", &["lock"])),
    });
}

fn logout() {
    #[cfg(target_os = "linux")]
    let username = whoami::username();
    #[cfg(target_os = "linux")]
    let linux_args: &[&str] = &["-KILL", "-u", &username];

    execute_platform_command(&PlatformCommand {
        log_message: "User logout initiated",
        windows: Some(("shutdown", &["/l"])),
        macos: Some(("osascript", &["-e", "tell app \"System Events\" to log out"])),
        #[cfg(target_os = "linux")]
        linux: Some(("pkill", linux_args)),
        #[cfg(not(target_os = "linux"))]
        linux: None,
    });
}

#[tauri::command]
pub fn open_app_data_dir(app: AppHandle) {
    if let Ok(path) = app.path().app_data_dir() {
        if let Err(e) = opener::open(&path) {
            eprintln!("Failed to open app data dir: {}", e);
        }
    }
}

fn refresh_list(app: AppHandle) {
    async_runtime::spawn(async move {
        command_manager::commands::refresh_commands(app.clone()).await;
    });
}

// ============================================================================
// 键盘模拟
// ============================================================================

// ============================================================================
// 键盘模拟
// ============================================================================

/// 模拟 Ctrl+V / Cmd+V 粘贴操作
/// 这是一个内部辅助函数，非 Tauri Command
pub fn simulate_paste_native(app: &AppHandle) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        use std::mem;
        use windows::Win32::UI::Input::KeyboardAndMouse::{
            SendInput, INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYBD_EVENT_FLAGS,
            KEYEVENTF_KEYUP, VK_CONTROL, VK_V,
        };

        unsafe {
            let mut inputs: [INPUT; 4] = mem::zeroed();

            // Key down: Ctrl
            inputs[0].r#type = INPUT_KEYBOARD;
            inputs[0].Anonymous.ki.wVk = VK_CONTROL;

            // Key down: V
            inputs[1].r#type = INPUT_KEYBOARD;
            inputs[1].Anonymous.ki.wVk = VK_V;

            // Key up: V
            inputs[2].r#type = INPUT_KEYBOARD;
            inputs[2].Anonymous.ki.wVk = VK_V;
            inputs[2].Anonymous.ki.dwFlags = KEYEVENTF_KEYUP;

            // Key up: Ctrl
            inputs[3].r#type = INPUT_KEYBOARD;
            inputs[3].Anonymous.ki.wVk = VK_CONTROL;
            inputs[3].Anonymous.ki.dwFlags = KEYEVENTF_KEYUP;

            let sent = SendInput(&inputs, mem::size_of::<INPUT>() as i32);
            if sent != 4 {
                return Err(format!("SendInput failed: only {} of 4 inputs sent", sent));
            }
        }
        Ok(())
    }

    #[cfg(target_os = "macos")]
    {
        use std::process::Command;
        use std::thread;
        use std::time::Duration;
        
        println!("[SystemCommand] simulate_paste_native (macOS via osascript) started");

        // 获取记录的前一个应用的 Bundle ID
        let bundle_id = app.state::<MacOSPreviousApp>()
            .0.lock().ok()
            .and_then(|guard| guard.clone());
        
        if let Some(bundle_id) = bundle_id {
            println!("[SystemCommand] Activating previous app: {}", bundle_id);
            // 激活应用
            let _ = Command::new("osascript")
                .args(["-e", &format!(r#"tell application id "{}" to activate"#, bundle_id)])
                .output();
            
            // 给一点时间让窗口激活
            thread::sleep(Duration::from_millis(100));
        } else {
            println!("[SystemCommand] No previous app recorded, waiting generic time");
            thread::sleep(Duration::from_millis(200));
        }
        
        println!("[SystemCommand] Sending Cmd+V via osascript");
        // 模拟按键
        let result = Command::new("osascript")
            .args(["-e", "tell application \"System Events\" to keystroke \"v\" using command down"])
            .output();
        
        match result {
            Ok(output) if output.status.success() => {
                println!("[SystemCommand] Paste successful");
                Ok(())
            },
            Ok(output) => {
                let stderr = String::from_utf8_lossy(&output.stderr);
                eprintln!("[SystemCommand] osascript failed: {}", stderr);
                if stderr.contains("not allowed") || stderr.contains("not permitted") {
                    Err("需要辅助功能权限。请前往 系统设置 > 隐私与安全性 > 辅助功能，添加此应用。".to_string())
                } else {
                    Err(format!("osascript failed: {}", stderr))
                }
            }
            Err(e) => {
                eprintln!("[SystemCommand] Failed to execute osascript: {}", e);
                Err(format!("Failed to execute osascript: {}", e))
            },
        }
    }

    #[cfg(target_os = "linux")]
    {
        use std::process::Command;
        // xdotool is still the external command way, but reliable on X11
        // Wayland support would need ydotool or similar but that's out of scope
        let result = Command::new("xdotool")
            .args(["key", "ctrl+v"])
            .output();
        
        match result {
            Ok(output) if output.status.success() => Ok(()),
            Ok(output) => Err(format!("xdotool failed: {:?}", String::from_utf8_lossy(&output.stderr))),
            Err(e) => Err(format!("Failed to execute xdotool: {}", e)),
        }
    }
}

/// 对外暴露粘贴命令，内部复用 native 粘贴实现
#[tauri::command]
pub fn simulate_paste(app: AppHandle) -> Result<(), String> {
    simulate_paste_native(&app)
}

// ============================================================================
// macOS 特定：记录前一个应用
// ============================================================================

#[cfg(target_os = "macos")]
pub struct MacOSPreviousApp(pub std::sync::Mutex<Option<String>>);

#[cfg(target_os = "macos")]
pub fn get_frontmost_app_bundle_id() -> Option<String> {
    use objc2_app_kit::NSWorkspace;
    
    unsafe {
        let workspace = NSWorkspace::sharedWorkspace();
        workspace.frontmostApplication()
            .and_then(|app| app.bundleIdentifier())
            .map(|id| id.to_string())
    }
}
