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
    },
    SystemCommandInfo {
        name: "reboot",
        title: "重启",
        description: "重新启动计算机",
        english_name: "Restart",
        keywords: &["restart", "reboot", "重启"],
        icon: "restart",
        action: |_| reboot(),
    },
    SystemCommandInfo {
        name: "sleep",
        title: "睡眠",
        description: "使计算机进入睡眠模式",
        english_name: "Sleep",
        keywords: &["sleep", "睡眠"],
        icon: "sleep",
        action: |_| sleep(),
    },
    SystemCommandInfo {
        name: "lock_screen",
        title: "锁屏",
        description: "锁定计算机屏幕",
        english_name: "Lock Screen",
        keywords: &["lock", "锁屏"],
        icon: "lock",
        action: |_| lock_screen(),
    },
    SystemCommandInfo {
        name: "logout",
        title: "注销",
        description: "注销当前用户",
        english_name: "Logout",
        keywords: &["logout", "注销"],
        icon: "logout",
        action: |_| logout(),
    },
    SystemCommandInfo {
        name: "open_app_data_dir",
        title: "打开应用数据目录",
        description: "打开应用程序的数据存储目录",
        english_name: "Open App Data Directory",
        keywords: &["数据目录"],
        icon: "folder",
        action: open_app_data_dir,
    },
    SystemCommandInfo {
        name: "refresh_list",
        title: "刷新列表",
        description: "刷新应用和命令列表",
        english_name: "Refresh List",
        keywords: &["refresh", "刷新"],
        icon: "arrowsClockwise",
        action: refresh_list,
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
    window: tauri::WebviewWindow,
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
                if let Err(e) = installed_apps::open_app(path.clone(), window) {
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
                command_code,
            } => {
                // Extension 命令由前端处理（导航到独立页面）
                // 后端只需记录日志
                tracing::info!(
                    "Extension command triggered: {}:{}",
                    extension_id,
                    command_code
                );
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

fn open_app_data_dir(app: AppHandle) {
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

/// 模拟 Ctrl+V 粘贴操作
/// 用于在窗口关闭后将剪贴板内容粘贴到之前活动的应用
#[tauri::command]
pub fn simulate_paste() -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        use std::mem;
        use windows::Win32::UI::Input::KeyboardAndMouse::{
            SendInput, INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYBD_EVENT_FLAGS,
            KEYEVENTF_KEYUP, VIRTUAL_KEY, VK_CONTROL, VK_V,
        };

        unsafe {
            let mut inputs: [INPUT; 4] = mem::zeroed();

            // Key down: Ctrl
            inputs[0] = INPUT {
                r#type: INPUT_KEYBOARD,
                Anonymous: INPUT_0 {
                    ki: KEYBDINPUT {
                        wVk: VK_CONTROL,
                        wScan: 0,
                        dwFlags: KEYBD_EVENT_FLAGS(0),
                        time: 0,
                        dwExtraInfo: 0,
                    },
                },
            };

            // Key down: V
            inputs[1] = INPUT {
                r#type: INPUT_KEYBOARD,
                Anonymous: INPUT_0 {
                    ki: KEYBDINPUT {
                        wVk: VK_V,
                        wScan: 0,
                        dwFlags: KEYBD_EVENT_FLAGS(0),
                        time: 0,
                        dwExtraInfo: 0,
                    },
                },
            };

            // Key up: V
            inputs[2] = INPUT {
                r#type: INPUT_KEYBOARD,
                Anonymous: INPUT_0 {
                    ki: KEYBDINPUT {
                        wVk: VK_V,
                        wScan: 0,
                        dwFlags: KEYEVENTF_KEYUP,
                        time: 0,
                        dwExtraInfo: 0,
                    },
                },
            };

            // Key up: Ctrl
            inputs[3] = INPUT {
                r#type: INPUT_KEYBOARD,
                Anonymous: INPUT_0 {
                    ki: KEYBDINPUT {
                        wVk: VK_CONTROL,
                        wScan: 0,
                        dwFlags: KEYEVENTF_KEYUP,
                        time: 0,
                        dwExtraInfo: 0,
                    },
                },
            };

            let sent = SendInput(&inputs, mem::size_of::<INPUT>() as i32);
            if sent != 4 {
                return Err(format!("SendInput failed: only {} of 4 inputs sent", sent));
            }
        }

        println!("[simulate_paste] Ctrl+V simulated on Windows");
        Ok(())
    }

    #[cfg(target_os = "macos")]
    {
        // macOS: 使用 CGEvent 模拟 Cmd+V
        use std::process::Command;
        
        // 使用 osascript 来模拟按键
        let result = Command::new("osascript")
            .args(["-e", "tell application \"System Events\" to keystroke \"v\" using command down"])
            .output();
        
        match result {
            Ok(output) => {
                if output.status.success() {
                    println!("[simulate_paste] Cmd+V simulated on macOS");
                    Ok(())
                } else {
                    Err(format!("osascript failed: {:?}", String::from_utf8_lossy(&output.stderr)))
                }
            }
            Err(e) => Err(format!("Failed to execute osascript: {}", e)),
        }
    }

    #[cfg(target_os = "linux")]
    {
        // Linux: 使用 xdotool 模拟 Ctrl+V
        use std::process::Command;
        
        let result = Command::new("xdotool")
            .args(["key", "ctrl+v"])
            .output();
        
        match result {
            Ok(output) => {
                if output.status.success() {
                    println!("[simulate_paste] Ctrl+V simulated on Linux");
                    Ok(())
                } else {
                    Err(format!("xdotool failed: {:?}", String::from_utf8_lossy(&output.stderr)))
                }
            }
            Err(e) => Err(format!("Failed to execute xdotool: {}", e)),
        }
    }
}