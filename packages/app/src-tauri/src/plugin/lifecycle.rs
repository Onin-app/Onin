use std::path::{Path, PathBuf};

use super::types::{InstallSource, PluginManifest};

pub fn resolve_lifecycle_script_path(
    plugin_dir: &Path,
    manifest: &PluginManifest,
    install_source: InstallSource,
) -> Option<PathBuf> {
    let entry_path = plugin_dir.join(&manifest.entry);
    let extension = Path::new(&manifest.entry)
        .extension()
        .and_then(|s| s.to_str());
    let is_local_dev_mode = install_source == InstallSource::Local
        && manifest.dev_mode
        && manifest.dev_server.is_some();

    match extension {
        Some("js") if entry_path.is_file() => Some(entry_path),
        Some("html") => {
            if is_local_dev_mode {
                if let Some(background_file) = manifest.background.as_deref() {
                    let bg_path = plugin_dir.join(background_file);
                    if bg_path.is_file() {
                        return Some(bg_path);
                    }
                }

                let dev_background_path = plugin_dir.join("background.js");
                if dev_background_path.is_file() {
                    return Some(dev_background_path);
                }
            }

            if !entry_path.is_file() {
                return None;
            }

            let background_file = manifest
                .background
                .as_deref()
                .unwrap_or(PluginManifest::default_background_entry());
            let bg_path = plugin_dir.join(background_file);

            if bg_path.is_file() {
                Some(bg_path)
            } else {
                None
            }
        }
        _ => None,
    }
}
