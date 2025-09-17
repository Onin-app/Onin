use crate::command_manager;
use crate::{installed_apps, plugin_manager};
use crate::shared_types::{Command, CommandAction, IconType, ItemSource, ItemType, LaunchableItem};
use tauri::{command, AppHandle, Manager};

// --- 1. Single Source of Truth ---

pub struct SystemCommandInfo {
    pub name: &'static str,
    pub title: &'static str,
    pub english_name: &'static str,
    pub keywords: &'static [&'static str],
    pub icon: &'static str,
    pub action: fn(AppHandle),
}

pub static SYSTEM_COMMANDS: &[SystemCommandInfo] = &[
    SystemCommandInfo {
        name: "shutdown",
        title: "关机",
        english_name: "Shutdown",
        keywords: &["shutdown", "关机"],
        icon: "shutdown",
        action: |_| shutdown(),
    },
    SystemCommandInfo {
        name: "reboot",
        title: "重启",
        english_name: "Restart",
        keywords: &["restart", "reboot", "重启"],
        icon: "restart",
        action: |_| reboot(),
    },
    SystemCommandInfo {
        name: "sleep",
        title: "睡眠",
        english_name: "Sleep",
        keywords: &["sleep", "睡眠"],
        icon: "sleep",
        action: |_| sleep(),
    },
    SystemCommandInfo {
        name: "lock_screen",
        title: "锁屏",
        english_name: "Lock Screen",
        keywords: &["lock", "锁屏"],
        icon: "lock",
        action: |_| lock_screen(),
    },
    SystemCommandInfo {
        name: "logout",
        title: "注销",
        english_name: "Logout",
        keywords: &["logout", "注销"],
        icon: "logout",
        action: |_| logout(),
    },
    SystemCommandInfo {
        name: "open_app_data_dir",
        title: "打开应用数据目录",
        english_name: "Open App Data Directory",
        keywords: &["数据目录"],
        icon: "folder",
        action: |app| open_app_data_dir(app),
    },
];

// --- 2. Derived Data Functions ---

#[command]
pub async fn get_basic_commands(app: AppHandle) -> Vec<Command> {
    command_manager::load_commands(&app).await
}

pub async fn get_system_commands_as_launchable_items(app: AppHandle) -> Vec<LaunchableItem> {
    let commands = command_manager::load_commands(&app).await;
    commands
        .iter()
        .map(|cmd| LaunchableItem {
            name: cmd.english_name.clone(),
            keywords: cmd
                .keywords
                .iter()
                .filter(|kw| kw.disabled.is_none() || !kw.disabled.unwrap())
                .map(|kw| kw.clone())
                .collect(),
            path: "".to_string(),
            icon: cmd.icon.clone(),
            icon_type: IconType::Iconfont,
            item_type: ItemType::App,
            source: ItemSource::Command,
            action: Some(cmd.name.clone()),
            origin: None,
        })
        .collect()
}

// --- 3. Unified Command Executor ---

#[command]
pub async fn execute_command(name: String, app: AppHandle, window: tauri::WebviewWindow) {
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
                let plugin_store = app.state::<plugin_manager::PluginStore>();
                if let Err(e) =
                    plugin_manager::execute_plugin_entry(app.clone(), plugin_store, plugin_id.clone())
                {
                    eprintln!("Failed to execute plugin {}: {}", plugin_id, e);
                }
            }
        }
    } else {
        eprintln!("Command not found: {}", name);
    }
}

// --- 4. Private Implementation Details ---
fn shutdown() {
    println!("System shutdown initiated");
    #[cfg(target_os = "windows")]
    {
        if let Err(e) = std::process::Command::new("shutdown").args(&["/s", "/t", "0"]).output() {
            eprintln!("Failed to execute shutdown on Windows: {}", e);
        }
    }
    #[cfg(target_os = "macos")]
    {
        if let Err(e) = std::process::Command::new("osascript").args(&["-e", "tell app \"System Events\" to shut down"]).output() {
            eprintln!("Failed to execute shutdown on macOS: {}", e);
        }
    }
    #[cfg(target_os = "linux")]
    {
        if let Err(e) = std::process::Command::new("shutdown").arg("now").output() {
            eprintln!("Failed to execute shutdown on Linux: {}", e);
        }
    }
}

fn reboot() {
    println!("System reboot initiated");
    #[cfg(target_os = "windows")]
    {
        if let Err(e) = std::process::Command::new("shutdown").args(&["/r", "/t", "0"]).output() {
            eprintln!("Failed to execute reboot on Windows: {}", e);
        }
    }
    #[cfg(target_os = "macos")]
    {
        if let Err(e) = std::process::Command::new("osascript").args(&["-e", "tell app \"System Events\" to restart"]).output() {
            eprintln!("Failed to execute reboot on macOS: {}", e);
        }
    }
    #[cfg(target_os = "linux")]
    {
        if let Err(e) = std::process::Command::new("reboot").output() {
            eprintln!("Failed to execute reboot on Linux: {}", e);
        }
    }
}

fn sleep() {
    println!("System sleep initiated");
    #[cfg(target_os = "windows")]
    {
        if let Err(e) = std::process::Command::new("rundll32.exe").args(&["powrprof.dll,SetSuspendState", "0", "1", "0"]).output() {
            eprintln!("Failed to execute sleep on Windows: {}", e);
        }
    }
    #[cfg(target_os = "macos")]
    {
        if let Err(e) = std::process::Command::new("pmset").arg("sleepnow").output() {
            eprintln!("Failed to execute sleep on macOS: {}", e);
        }
    }
    #[cfg(target_os = "linux")]
    {
        if let Err(e) = std::process::Command::new("systemctl").arg("suspend").output() {
            eprintln!("Failed to execute sleep on Linux: {}", e);
        }
    }
}

fn lock_screen() {
    println!("Screen lock initiated");
    #[cfg(target_os = "windows")]
    {
        if let Err(e) = std::process::Command::new("rundll32.exe").args(&["user32.dll,LockWorkStation"]).output() {
            eprintln!("Failed to execute lock_screen on Windows: {}", e);
        }
    }
    #[cfg(target_os = "macos")]
    {
        if let Err(e) = std::process::Command::new("pmset").arg("displaysleepnow").output() {
            eprintln!("Failed to execute lock_screen on macOS: {}", e);
        }
    }
    #[cfg(target_os = "linux")]
    {
        if let Err(e) = std::process::Command::new("xdg-screensaver").arg("lock").output() {
            eprintln!("Failed to execute lock_screen on Linux with xdg-screensaver: {}. Trying other methods.", e);
        }
    }
}

fn logout() {
    println!("User logout initiated");
    #[cfg(target_os = "windows")]
    {
        if let Err(e) = std::process::Command::new("shutdown").args(&["/l"]).output() {
            eprintln!("Failed to execute logout on Windows: {}", e);
        }
    }
    #[cfg(target_os = "macos")]
    {
        if let Err(e) = std::process::Command::new("osascript").args(&["-e", "tell app \"System Events\" to log out"]).output() {
            eprintln!("Failed to execute logout on macOS: {}", e);
        }
    }
    #[cfg(target_os = "linux")]
    {
        if let Err(e) = std::process::Command::new("pkill").arg("-KILL").arg("-u").arg(whoami::username()).output() {
            eprintln!("Failed to execute logout on Linux: {}", e);
        }
    }
}

fn open_app_data_dir(app: AppHandle) {
    if let Ok(path) = app.path().app_data_dir() {
        if let Err(e) = opener::open(&path) {
            eprintln!("Failed to open app data dir: {}", e);
        }
    }
}