use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use tauri::{AppHandle, Manager, Runtime};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Default)]
pub struct PermissionStore(HashMap<String, HashSet<String>>);

pub struct PermissionManager {
    store: PermissionStore,
    store_path: PathBuf,
}

impl PermissionManager {
    pub fn new<R: Runtime>(app_handle: &AppHandle<R>) -> Self {
        let store_path = app_handle.path().app_data_dir().expect("Failed to get app data dir").join("permissions.json");
        let store = Self::load_store(&store_path);
        Self { store, store_path }
    }

    fn load_store(path: &Path) -> PermissionStore {
        if !path.exists() {
            return PermissionStore::default();
        }
        fs::read_to_string(path)
            .ok()
            .and_then(|content| serde_json::from_str(&content).ok())
            .unwrap_or_default()
    }

    fn save_store(&self) {
        let content = serde_json::to_string_pretty(&self.store).unwrap();
        fs::write(&self.store_path, content).expect("Failed to save permissions");
    }

    pub fn grant_permission(&mut self, plugin_id: &str, permission: &str) {
        let permissions = self.store.0.entry(plugin_id.to_string()).or_default();
        permissions.insert(permission.to_string());
        self.save_store();
    }

    pub fn revoke_permission(&mut self, plugin_id: &str, permission: &str) {
        if let Some(permissions) = self.store.0.get_mut(plugin_id) {
            permissions.remove(permission);
            self.save_store();
        }
    }

    pub fn has_permission(&self, plugin_id: &str, permission: &str) -> bool {
        self.store.0
            .get(plugin_id)
            .map_or(false, |perms| perms.contains(permission))
    }

    pub fn get_permissions(&self, plugin_id: &str) -> Option<&HashSet<String>> {
        self.store.0.get(plugin_id)
    }
}