use winreg::enums::{HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE};
use winreg::{RegKey, HKEY};

const UNINSTALL_PATHS: &[(&str, HKEY)] = &[
    (
        "SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Uninstall",
        HKEY_LOCAL_MACHINE,
    ),
    (
        "SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Uninstall",
        HKEY_CURRENT_USER,
    ),
    (
        "SOFTWARE\\Wow6432Node\\Microsoft\\Windows\\CurrentVersion\\Uninstall",
        HKEY_LOCAL_MACHINE,
    ),
];

pub fn get_apps() -> Result<Vec<String>, String> {
    let mut apps = vec![];

    for (path, hive) in UNINSTALL_PATHS {
        let root = RegKey::predef(*hive);
        if let Ok(uninstall_key) = root.open_subkey(path) {
            for item in uninstall_key.enum_keys().filter_map(Result::ok) {
                if let Ok(subkey) = uninstall_key.open_subkey(&item) {
                    // Skip if no DisplayName
                    let display_name: Result<String, _> = subkey.get_value("DisplayName");
                    if display_name.is_err() {
                        continue;
                    }

                    // Skip if SystemComponent == 1
                    let system_component: Result<u32, _> = subkey.get_value("SystemComponent");
                    if matches!(system_component, Ok(1)) {
                        continue;
                    }

                    // Skip if ParentKeyName or ParentDisplayName exists
                    let has_parent_key = subkey.get_value::<String, _>("ParentKeyName").is_ok();
                    let has_parent_display =
                        subkey.get_value::<String, _>("ParentDisplayName").is_ok();
                    if has_parent_key || has_parent_display {
                        continue;
                    }

                    apps.push(display_name.unwrap());
                }
            }
        }
    }

    Ok(apps)
}
