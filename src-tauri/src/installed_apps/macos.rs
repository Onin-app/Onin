use std::fs;

pub fn get_apps() -> Result<Vec<String>, String> {
    let mut apps = vec![];
    if let Ok(entries) = fs::read_dir("/Applications") {
        for entry in entries.flatten() {
            if let Ok(file_type) = entry.file_type() {
                if file_type.is_dir() {
                    let name = entry.file_name().to_string_lossy().to_string();
                    if name.ends_with(".app") {
                        apps.push(name);
                    }
                }
            }
        }
    }
    Ok(apps)
}
