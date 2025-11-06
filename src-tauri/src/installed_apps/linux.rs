use std::process::Command;

pub fn get_apps() -> Result<Vec<(String, Option<String>)>, String> {
    let output = Command::new("sh")
        .arg("-c")
        .arg("dpkg --get-selections")
        .output()
        .map_err(|e| e.to_string())?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let apps: Vec<(String, Option<String>)> = stdout
        .lines()
        .filter_map(|line| {
            line.split_whitespace()
                .next()
                .map(|s| (s.to_string(), None))
        })
        .collect();

    Ok(apps)
}

pub fn open_app(path: &str) -> Result<(), String> {
    // 在 Linux 上，使用 nohup 和后台运行确保进程独立
    Command::new("sh")
        .arg("-c")
        .arg(format!("nohup xdg-open '{}' > /dev/null 2>&1 &", path))
        .spawn()
        .map_err(|e| e.to_string())?;
    Ok(())
}
