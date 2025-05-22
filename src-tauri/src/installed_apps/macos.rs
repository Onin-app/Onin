use std::fs;
use std::process::Command;

pub fn get_apps() -> Result<Vec<(String, Option<String>)>, String> {
    let mut apps = vec![];
    if let Ok(entries) = fs::read_dir("/Applications") {
        for entry in entries.flatten() {
            if let Ok(file_type) = entry.file_type() {
                if file_type.is_dir() {
                    let name = entry.file_name().to_string_lossy().to_string();
                    if name.ends_with(".app") {
                        let path = format!("/Applications/{}", name);
                        apps.push((name, Some(path)));
                    }
                }
            }
        }
    }
    Ok(apps)
}

pub fn open_app(path: &str) -> Result<(), String> {
    Command::new("open")
        .arg(path)
        .spawn()
        .map_err(|e| e.to_string())?;
    Ok(())
}
