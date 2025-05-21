use std::process::Command;

pub fn get_apps() -> Result<Vec<String>, String> {
    let output = Command::new("sh")
        .arg("-c")
        .arg("dpkg --get-selections")
        .output()
        .map_err(|e| e.to_string())?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let apps: Vec<String> = stdout
        .lines()
        .filter_map(|line| line.split_whitespace().next().map(|s| s.to_string()))
        .collect();

    Ok(apps)
}
