use tauri::command;
use tauri::Manager;

#[command]
pub fn shutdown() {
    println!("System shutdown initiated");
    // 在这里添加特定平台的关机代码
    #[cfg(target_os = "windows")]
    {
        if let Err(e) = std::process::Command::new("shutdown")
            .args(&["/s", "/t", "0"])
            .output()
        {
            eprintln!("Failed to execute shutdown on Windows: {}", e);
        }
    }
    #[cfg(target_os = "macos")]
    {
        if let Err(e) = std::process::Command::new("osascript")
            .args(&["-e", "tell app \"System Events\" to shut down"])
            .output()
        {
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

#[command]
pub fn reboot() {
    println!("System reboot initiated");
    // 在这里添加特定平台的重启代码
     #[cfg(target_os = "windows")]
    {
        if let Err(e) = std::process::Command::new("shutdown")
            .args(&["/r", "/t", "0"])
            .output()
        {
            eprintln!("Failed to execute reboot on Windows: {}", e);
        }
    }
    #[cfg(target_os = "macos")]
    {
        if let Err(e) = std::process::Command::new("osascript")
            .args(&["-e", "tell app \"System Events\" to restart"])
            .output()
        {
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

#[command]
pub fn sleep() {
    println!("System sleep initiated");
    // 在这里添加特定平台的睡眠代码
    #[cfg(target_os = "windows")]
    {
        if let Err(e) = std::process::Command::new("rundll32.exe")
            .args(&["powrprof.dll,SetSuspendState", "0", "1", "0"])
            .output()
        {
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

#[command]
pub fn lock_screen() {
    println!("Screen lock initiated");
    // 在这里添加特定平台的锁屏代码
    #[cfg(target_os = "windows")]
    {
        if let Err(e) = std::process::Command::new("rundll32.exe")
            .args(&["user32.dll,LockWorkStation"])
            .output()
        {
            eprintln!("Failed to execute lock_screen on Windows: {}", e);
        }
    }
    #[cfg(target_os = "macos")]
    {
        if let Err(e) = std::process::Command::new("pmset")
            .arg("displaysleepnow")
            .output()
        {
            eprintln!("Failed to execute lock_screen on macOS: {}", e);
        }
    }
    #[cfg(target_os = "linux")]
    {
        // Linux的锁屏命令因桌面环境而异
        // 这里是一些常见的例子，可能需要用户配置
        if let Err(e) = std::process::Command::new("xdg-screensaver").arg("lock").output() {
            eprintln!("Failed to execute lock_screen on Linux with xdg-screensaver: {}. Trying other methods.", e);
            // 可以尝试其他命令
        }
    }
}

#[command]
pub fn logout() {
    println!("User logout initiated");
    // 在这里添加特定平台的注销代码
    #[cfg(target_os = "windows")]
    {
        if let Err(e) = std::process::Command::new("shutdown").args(&["/l"]).output() {
            eprintln!("Failed to execute logout on Windows: {}", e);
        }
    }
    #[cfg(target_os = "macos")]
    {
        if let Err(e) = std::process::Command::new("osascript")
            .args(&["-e", "tell app \"System Events\" to log out"])
            .output()
        {
            eprintln!("Failed to execute logout on macOS: {}", e);
        }
    }
    #[cfg(target_os = "linux")]
    {
        // Linux的注销命令也因桌面环境而异
        if let Err(e) = std::process::Command::new("pkill")
            .arg("-KILL")
            .arg("-u")
            .arg(whoami::username())
            .output()
        {
            eprintln!("Failed to execute logout on Linux: {}", e);
        }
    }
}

#[command]
pub fn open_app_data_dir(app: tauri::AppHandle) {
    if let Ok(path) = app.path().app_data_dir() {
        if let Err(e) = opener::open(&path) {
            eprintln!("Failed to open app data dir: {}", e);
        }
    }
}