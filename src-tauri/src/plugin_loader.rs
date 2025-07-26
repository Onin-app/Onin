use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use tauri::{AppHandle, Manager, Runtime};
use crate::plugin_manifest::PluginManifest;
use semver::{Version, VersionReq};

pub struct PluginLoader<R: Runtime> {
    pub plugins: HashMap<String, PluginManifest>,
    app_handle: AppHandle<R>,
}

impl<R: Runtime> PluginLoader<R> {
    pub fn new(app_handle: AppHandle<R>) -> Self {
        Self {
            plugins: HashMap::new(),
            app_handle,
        }
    }

    pub fn load_plugins(&mut self) -> Result<(), String> {
        let plugins_dir = self.app_handle.path().app_data_dir()
            .map_err(|e| e.to_string())?
            .join("plugins");
        if !plugins_dir.exists() {
            fs::create_dir_all(&plugins_dir).map_err(|e| e.to_string())?;
            return Ok(());
        }

        let mut manifests = HashMap::new();
        let mut to_load: Vec<String> = Vec::new();

        for entry in fs::read_dir(&plugins_dir).map_err(|e| e.to_string())?.filter_map(Result::ok) {
            if entry.path().is_dir() {
                let manifest_path = entry.path().join("manifest.json");
                if manifest_path.exists() {
                    let content = fs::read_to_string(&manifest_path).map_err(|e| e.to_string())?;
                    let mut manifest: PluginManifest = serde_json::from_str(&content).map_err(|e| e.to_string())?;
                    manifest.path = Some(entry.file_name().to_str().unwrap().to_string());
                    manifest.validate()?;
                    manifests.insert(manifest.id.clone(), manifest);
                }
            }
        }

        let mut loaded: HashMap<String, PluginManifest> = HashMap::new();
        let mut loading_queue: Vec<_> = manifests.keys().cloned().collect();
        
        while let Some(plugin_id) = loading_queue.pop() {
            if loaded.contains_key(&plugin_id) {
                continue;
            }

            let manifest = manifests.get(&plugin_id).unwrap();
            let mut deps_satisfied = true;
            for (dep_id, version_req_str) in &manifest.dependencies {
                if let Some(dep_manifest) = loaded.get(dep_id) {
                    let version_req = semver::VersionReq::parse(version_req_str).map_err(|e| e.to_string())?;
                    let dep_version = semver::Version::parse(&dep_manifest.version).map_err(|e| e.to_string())?;
                    if !version_req.matches(&dep_version) {
                        return Err(format!("Plugin {} requires {} version {} but found version {}", plugin_id, dep_id, version_req_str, dep_version));
                    }
                } else {
                    if !manifests.contains_key(dep_id) {
                        return Err(format!("Unresolved dependency for plugin '{}': '{}' is not available.", plugin_id, dep_id));
                    }
                    deps_satisfied = false;
                    loading_queue.push(plugin_id.clone()); // Re-queue
                    if !loading_queue.contains(&dep_id) {
                        loading_queue.push(dep_id.clone());
                    }
                    break;
                }
            }

            if deps_satisfied {
                let manifest_clone = manifest.clone();
                println!("Loaded plugin: {} ({})", manifest_clone.name, manifest_clone.id);
                self.plugins.insert(plugin_id.clone(), manifest_clone.clone());
                loaded.insert(plugin_id.clone(), manifest_clone);
            }
        }

        if self.plugins.len() != manifests.len() {
            return Err("Could not load all plugins due to circular or missing dependencies.".to_string());
        }

        println!("[PLUGIN_LOADER] Finished loading plugins. Total loaded: {}", self.plugins.len());
        Ok(())
    }

    fn load_plugin(&mut self, path: &PathBuf) -> Result<(), String> {
        let manifest_path = path.join("manifest.json");
        if !manifest_path.exists() {
            return Err("manifest.json not found".to_string());
        }

        let manifest_content = fs::read_to_string(manifest_path).map_err(|e| e.to_string())?;
        let mut manifest: PluginManifest = serde_json::from_str(&manifest_content).map_err(|e| e.to_string())?;
        manifest.path = Some(path.file_name().unwrap().to_str().unwrap().to_string());
        manifest.validate()?;

        println!("Loaded plugin: {} ({})", manifest.name, manifest.id);
        self.plugins.insert(manifest.id.clone(), manifest);

        Ok(())
    }

    pub fn install_plugin(&mut self, path: &tauri_plugin_dialog::FilePath, permission_manager: &mut crate::permission_manager::PermissionManager) -> Result<PluginManifest, String> {
        let file_path = path.as_path().ok_or_else(|| "Invalid file path".to_string())?;
        let file = fs::File::open(file_path).map_err(|e| e.to_string())?;
        let mut archive = zip::ZipArchive::new(file).map_err(|e| e.to_string())?;

        let plugins_dir = self.app_handle.path().app_data_dir()
            .map_err(|e| e.to_string())?
            .join("plugins");
        if !plugins_dir.exists() {
            fs::create_dir_all(&plugins_dir).map_err(|e| e.to_string())?;
        }

        // Extract the archive to a temporary directory first to validate it
        let temp_dir = tempfile::Builder::new().prefix("plugin-install-").tempdir().map_err(|e| e.to_string())?;
        archive.extract(temp_dir.path()).map_err(|e| e.to_string())?;

        // Find the root directory of the plugin
        let mut root_entries: Vec<_> = fs::read_dir(temp_dir.path()).map_err(|e| e.to_string())?.filter_map(Result::ok).collect();
        if root_entries.len() != 1 || !root_entries[0].path().is_dir() {
            return Err("Plugin archive must contain a single root directory.".to_string());
        }
        let plugin_root = root_entries.remove(0).path();
        let root_dir_name = plugin_root.file_name().unwrap().to_str().unwrap().to_string();

        let manifest_path = plugin_root.join("manifest.json");
        if !manifest_path.exists() {
            return Err("manifest.json not found in plugin root".to_string());
        }

        let manifest_content = fs::read_to_string(&manifest_path).map_err(|e| e.to_string())?;
        let mut manifest: PluginManifest = serde_json::from_str(&manifest_content).map_err(|e| e.to_string())?;
        manifest.validate()?;
        manifest.path = Some(root_dir_name.clone());

        // Move the validated plugin to the final destination
        let final_plugin_path = plugins_dir.join(&root_dir_name);
        if final_plugin_path.exists() {
            fs::remove_dir_all(&final_plugin_path).map_err(|e| e.to_string())?;
        }
        fs::rename(&plugin_root, &final_plugin_path).map_err(|e| e.to_string())?;

        for permission in &manifest.permissions {
            permission_manager.grant_permission(&manifest.id, permission);
        }

        self.plugins.insert(manifest.id.clone(), manifest.clone());

        Ok(manifest)
    }

    pub fn uninstall_plugin(&mut self, plugin_id: &str, permission_manager: &mut crate::permission_manager::PermissionManager) -> Result<(), String> {
        let plugin = self.plugins.get(plugin_id).ok_or("Plugin not found")?;
        let plugins_dir = self.app_handle.path().app_data_dir()
            .map_err(|e| e.to_string())?
            .join("plugins");
        
        if let Some(path) = &plugin.path {
            let plugin_path = plugins_dir.join(path);
            if plugin_path.exists() {
                fs::remove_dir_all(plugin_path).map_err(|e| e.to_string())?;
            }
        }

        if let Some(permissions) = permission_manager.get_permissions(plugin_id) {
            for permission in permissions.clone() {
                permission_manager.revoke_permission(plugin_id, &permission);
            }
        }

        self.plugins.remove(plugin_id);

        Ok(())
    }
}