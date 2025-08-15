use std::collections::HashMap;
use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager, State};
use tokio::fs;
use tokio::sync::RwLock;
use walkdir::WalkDir;

#[derive(Debug, thiserror::Error)]
pub enum PluginError {
    #[error("Plugin not found: {0}")]
    NotFound(String),
    
    #[error("Invalid plugin manifest: {0}")]
    InvalidManifest(String),
    
    #[error("Plugin load failed: {0}")]
    LoadFailed(String),
    
    #[error("Permission denied: {0}")]
    PermissionDenied(String),
    
    #[error("Version incompatible: required {required}, found {found}")]
    VersionIncompatible { required: String, found: String },
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("JSON parsing error: {0}")]
    Json(#[from] serde_json::Error),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PluginManifest {
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub main: String,
    pub permissions: Vec<String>,
    pub engines: HashMap<String, String>,
    #[serde(default)]
    pub keywords: Vec<String>,
    #[serde(default)]
    pub repository: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PluginInfo {
    pub manifest: PluginManifest,
    pub path: PathBuf,
    pub enabled: bool,
    pub loaded: bool,
    pub status: PluginStatus,
}

#[derive(Debug, Clone, Serialize)]
pub enum PluginStatus {
    Active,
    Inactive,
    Error(String),
    Loading,
}

pub struct PluginManager {
    plugins: HashMap<String, PluginInfo>,
    plugin_dir: PathBuf,
}

// Wrapper for thread-safe plugin manager state
pub struct PluginManagerState(pub RwLock<PluginManager>);

impl PluginManager {
    pub fn new(plugin_dir: PathBuf) -> Self {
        Self {
            plugins: HashMap::new(),
            plugin_dir,
        }
    }

    /// Discover all plugins in the plugin directory
    pub async fn discover_plugins(&mut self) -> Result<(), PluginError> {
        tracing::info!("Starting plugin discovery in: {:?}", self.plugin_dir);
        
        // Ensure plugin directory exists
        if !self.plugin_dir.exists() {
            fs::create_dir_all(&self.plugin_dir).await?;
            tracing::info!("Created plugin directory: {:?}", self.plugin_dir);
            return Ok(());
        }

        // Clear existing plugins
        self.plugins.clear();

        // Walk through plugin directory
        for entry in WalkDir::new(&self.plugin_dir)
            .max_depth(2) // Limit depth to avoid deep recursion
            .into_iter()
            .filter_map(|e| e.ok())
        {
            if entry.file_type().is_file() && entry.file_name() == "plugin.json" {
                let plugin_dir = entry.path().parent().unwrap();
                match self.load_plugin_manifest(plugin_dir).await {
                    Ok(plugin_info) => {
                        tracing::info!("Discovered plugin: {}", plugin_info.manifest.name);
                        self.plugins.insert(plugin_info.manifest.name.clone(), plugin_info);
                    }
                    Err(e) => {
                        tracing::warn!("Failed to load plugin from {:?}: {}", plugin_dir, e);
                        // Continue discovering other plugins even if one fails
                    }
                }
            }
        }

        tracing::info!("Plugin discovery completed. Found {} plugins", self.plugins.len());
        Ok(())
    }

    /// Load and parse a plugin manifest from a directory
    async fn load_plugin_manifest(&self, plugin_dir: &Path) -> Result<PluginInfo, PluginError> {
        let manifest_path = plugin_dir.join("plugin.json");
        
        // Read manifest file
        let manifest_content = fs::read_to_string(&manifest_path).await
            .map_err(|e| PluginError::InvalidManifest(format!("Cannot read manifest: {}", e)))?;
        
        // Parse manifest JSON
        let manifest: PluginManifest = serde_json::from_str(&manifest_content)
            .map_err(|e| PluginError::InvalidManifest(format!("Invalid JSON: {}", e)))?;
        
        // Validate manifest
        self.validate_plugin_manifest(&manifest, plugin_dir)?;
        
        // Create plugin info
        let plugin_info = PluginInfo {
            manifest,
            path: plugin_dir.to_path_buf(),
            enabled: false, // Default to disabled
            loaded: false,
            status: PluginStatus::Inactive,
        };
        
        Ok(plugin_info)
    }

    /// Validate plugin manifest and check compatibility
    fn validate_plugin_manifest(&self, manifest: &PluginManifest, plugin_dir: &Path) -> Result<(), PluginError> {
        // Check required fields
        if manifest.name.is_empty() {
            return Err(PluginError::InvalidManifest("Plugin name cannot be empty".to_string()));
        }
        
        if manifest.version.is_empty() {
            return Err(PluginError::InvalidManifest("Plugin version cannot be empty".to_string()));
        }
        
        if manifest.main.is_empty() {
            return Err(PluginError::InvalidManifest("Plugin main file cannot be empty".to_string()));
        }
        
        // Check if main file exists
        let main_file_path = plugin_dir.join(&manifest.main);
        if !main_file_path.exists() {
            return Err(PluginError::InvalidManifest(
                format!("Main file not found: {}", manifest.main)
            ));
        }
        
        // Validate version format (basic semver check)
        if !self.is_valid_semver(&manifest.version) {
            return Err(PluginError::InvalidManifest(
                format!("Invalid version format: {}", manifest.version)
            ));
        }
        
        // Check engine compatibility
        if let Some(required_version) = manifest.engines.get("baize") {
            if !self.is_engine_compatible(required_version) {
                return Err(PluginError::VersionIncompatible {
                    required: required_version.clone(),
                    found: env!("CARGO_PKG_VERSION").to_string(),
                });
            }
        }
        
        // Validate permissions
        for permission in &manifest.permissions {
            if !self.is_valid_permission(permission) {
                return Err(PluginError::InvalidManifest(
                    format!("Invalid permission: {}", permission)
                ));
            }
        }
        
        Ok(())
    }

    /// Basic semver validation
    fn is_valid_semver(&self, version: &str) -> bool {
        let parts: Vec<&str> = version.split('.').collect();
        if parts.len() != 3 {
            return false;
        }
        
        parts.iter().all(|part| part.parse::<u32>().is_ok())
    }

    /// Check if the required engine version is compatible
    fn is_engine_compatible(&self, required_version: &str) -> bool {
        // For now, do a simple version comparison
        // In a real implementation, you'd want proper semver comparison
        let current_version = env!("CARGO_PKG_VERSION");
        
        // Simple compatibility check - exact match or current >= required
        if required_version.starts_with(">=") {
            let required = required_version.trim_start_matches(">=");
            self.compare_versions(current_version, required) >= 0
        } else if required_version.starts_with('>') {
            let required = required_version.trim_start_matches('>');
            self.compare_versions(current_version, required) > 0
        } else {
            // Exact match
            current_version == required_version
        }
    }

    /// Simple version comparison (returns -1, 0, or 1)
    fn compare_versions(&self, v1: &str, v2: &str) -> i32 {
        let v1_parts: Vec<u32> = v1.split('.').filter_map(|s| s.parse().ok()).collect();
        let v2_parts: Vec<u32> = v2.split('.').filter_map(|s| s.parse().ok()).collect();
        
        for i in 0..3 {
            let v1_part = v1_parts.get(i).unwrap_or(&0);
            let v2_part = v2_parts.get(i).unwrap_or(&0);
            
            if v1_part > v2_part {
                return 1;
            } else if v1_part < v2_part {
                return -1;
            }
        }
        
        0
    }

    /// Validate if a permission is allowed
    fn is_valid_permission(&self, permission: &str) -> bool {
        // Define allowed permissions
        const ALLOWED_PERMISSIONS: &[&str] = &[
            "notifications",
            "storage",
            "dialogs",
            "filesystem",
            "network",
            "clipboard",
        ];
        
        ALLOWED_PERMISSIONS.contains(&permission)
    }

    /// Get list of all discovered plugins
    pub fn get_plugin_list(&self) -> Vec<&PluginInfo> {
        self.plugins.values().collect()
    }

    /// Get a specific plugin by name
    pub fn get_plugin(&self, name: &str) -> Option<&PluginInfo> {
        self.plugins.get(name)
    }

    /// Get mutable reference to a plugin
    fn get_plugin_mut(&mut self, name: &str) -> Option<&mut PluginInfo> {
        self.plugins.get_mut(name)
    }

    /// Check if a plugin exists
    pub fn has_plugin(&self, name: &str) -> bool {
        self.plugins.contains_key(name)
    }

    /// Get the number of discovered plugins
    pub fn plugin_count(&self) -> usize {
        self.plugins.len()
    }

    /// Load a plugin (prepare it for activation)
    pub async fn load_plugin(&mut self, name: &str) -> Result<(), PluginError> {
        let plugin = self.get_plugin_mut(name)
            .ok_or_else(|| PluginError::NotFound(name.to_string()))?;

        if plugin.loaded {
            tracing::warn!("Plugin {} is already loaded", name);
            return Ok(());
        }

        tracing::info!("Loading plugin: {}", name);
        
        // Update status to loading
        plugin.status = PluginStatus::Loading;

        // Validate plugin files exist
        let main_file = plugin.path.join(&plugin.manifest.main);
        if !main_file.exists() {
            let error_msg = format!("Main file not found: {}", plugin.manifest.main);
            plugin.status = PluginStatus::Error(error_msg.clone());
            return Err(PluginError::LoadFailed(error_msg));
        }

        // TODO: In a real implementation, you would:
        // 1. Load the JavaScript/WASM module
        // 2. Validate the plugin exports
        // 3. Set up the plugin's sandbox environment
        // 4. Initialize the plugin context
        
        // For now, just mark as loaded
        plugin.loaded = true;
        plugin.status = if plugin.enabled {
            PluginStatus::Active
        } else {
            PluginStatus::Inactive
        };

        tracing::info!("Plugin {} loaded successfully", name);
        Ok(())
    }

    /// Unload a plugin (cleanup and remove from memory)
    pub async fn unload_plugin(&mut self, name: &str) -> Result<(), PluginError> {
        // Check if plugin exists and is loaded
        let (is_loaded, is_enabled) = {
            let plugin = self.get_plugin(name)
                .ok_or_else(|| PluginError::NotFound(name.to_string()))?;
            (plugin.loaded, plugin.enabled)
        };

        if !is_loaded {
            tracing::warn!("Plugin {} is not loaded", name);
            return Ok(());
        }

        tracing::info!("Unloading plugin: {}", name);

        // If plugin is enabled, disable it first
        if is_enabled {
            self.disable_plugin(name).await?;
        }

        // TODO: In a real implementation, you would:
        // 1. Call the plugin's deactivate method
        // 2. Clean up plugin resources
        // 3. Remove event listeners
        // 4. Clear plugin storage if requested
        // 5. Unload the JavaScript/WASM module

        let plugin = self.get_plugin_mut(name).unwrap();
        plugin.loaded = false;
        plugin.status = PluginStatus::Inactive;

        tracing::info!("Plugin {} unloaded successfully", name);
        Ok(())
    }

    /// Enable a plugin (activate it)
    pub async fn enable_plugin(&mut self, name: &str) -> Result<(), PluginError> {
        // Check current state
        let (is_enabled, is_loaded) = {
            let plugin = self.get_plugin(name)
                .ok_or_else(|| PluginError::NotFound(name.to_string()))?;
            (plugin.enabled, plugin.loaded)
        };

        if is_enabled {
            tracing::warn!("Plugin {} is already enabled", name);
            return Ok(());
        }

        tracing::info!("Enabling plugin: {}", name);

        // Load plugin if not already loaded
        if !is_loaded {
            self.load_plugin(name).await?;
        }

        // Enable the plugin
        self.enable_plugin_internal(name).await?;

        tracing::info!("Plugin {} enabled successfully", name);
        Ok(())
    }

    /// Disable a plugin (deactivate it but keep loaded)
    pub async fn disable_plugin(&mut self, name: &str) -> Result<(), PluginError> {
        let is_enabled = {
            let plugin = self.get_plugin(name)
                .ok_or_else(|| PluginError::NotFound(name.to_string()))?;
            plugin.enabled
        };

        if !is_enabled {
            tracing::warn!("Plugin {} is already disabled", name);
            return Ok(());
        }

        tracing::info!("Disabling plugin: {}", name);
        self.disable_plugin_internal(name).await?;
        tracing::info!("Plugin {} disabled successfully", name);
        Ok(())
    }

    /// Internal method to enable a plugin
    async fn enable_plugin_internal(&mut self, name: &str) -> Result<(), PluginError> {
        // Get manifest for validation
        let manifest = {
            let plugin = self.get_plugin(name).unwrap();
            plugin.manifest.clone()
        };

        // Update status to loading
        {
            let plugin = self.get_plugin_mut(name).unwrap();
            plugin.status = PluginStatus::Loading;
        }

        // TODO: In a real implementation, you would:
        // 1. Check plugin permissions
        // 2. Set up plugin context (API access, storage, etc.)
        // 3. Call the plugin's activate method
        // 4. Register plugin event handlers
        // 5. Handle any activation errors with proper isolation

        // Simulate activation process
        match self.simulate_plugin_activation(&manifest).await {
            Ok(_) => {
                let plugin = self.get_plugin_mut(name).unwrap();
                plugin.enabled = true;
                plugin.status = PluginStatus::Active;
                Ok(())
            }
            Err(e) => {
                let plugin = self.get_plugin_mut(name).unwrap();
                plugin.status = PluginStatus::Error(e.to_string());
                Err(e)
            }
        }
    }

    /// Internal method to disable a plugin
    async fn disable_plugin_internal(&mut self, name: &str) -> Result<(), PluginError> {
        // TODO: In a real implementation, you would:
        // 1. Call the plugin's deactivate method
        // 2. Remove plugin event handlers
        // 3. Clean up plugin resources (but keep loaded)
        // 4. Handle any deactivation errors with proper isolation

        let plugin = self.get_plugin_mut(name).unwrap();
        plugin.enabled = false;
        plugin.status = PluginStatus::Inactive;
        Ok(())
    }

    /// Simulate plugin activation (placeholder for real implementation)
    async fn simulate_plugin_activation(&self, manifest: &PluginManifest) -> Result<(), PluginError> {
        // Simulate some validation checks
        if manifest.name.contains("error") {
            return Err(PluginError::LoadFailed("Simulated activation error".to_string()));
        }

        // Simulate async activation work
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        Ok(())
    }

    /// Handle plugin errors and implement recovery strategies
    pub async fn handle_plugin_error(&mut self, name: &str, error: PluginError) -> Result<(), PluginError> {
        tracing::error!("Plugin {} encountered error: {}", name, error);

        let plugin = self.get_plugin_mut(name)
            .ok_or_else(|| PluginError::NotFound(name.to_string()))?;

        // Set error status
        plugin.status = PluginStatus::Error(error.to_string());

        // Implement recovery strategies based on error type
        match error {
            PluginError::LoadFailed(_) => {
                // Try to unload and reload the plugin
                tracing::info!("Attempting to recover plugin {} by reloading", name);
                plugin.loaded = false;
                plugin.enabled = false;
                // Could attempt reload here if desired
            }
            PluginError::PermissionDenied(_) => {
                // Disable plugin due to permission issues
                tracing::info!("Disabling plugin {} due to permission error", name);
                plugin.enabled = false;
                plugin.status = PluginStatus::Inactive;
            }
            _ => {
                // For other errors, just disable the plugin
                plugin.enabled = false;
            }
        }

        Ok(())
    }

    /// Get plugins by status
    pub fn get_plugins_by_status(&self, status: &PluginStatus) -> Vec<&PluginInfo> {
        self.plugins.values()
            .filter(|plugin| std::mem::discriminant(&plugin.status) == std::mem::discriminant(status))
            .collect()
    }

    /// Get enabled plugins
    pub fn get_enabled_plugins(&self) -> Vec<&PluginInfo> {
        self.plugins.values()
            .filter(|plugin| plugin.enabled)
            .collect()
    }

    /// Get loaded plugins
    pub fn get_loaded_plugins(&self) -> Vec<&PluginInfo> {
        self.plugins.values()
            .filter(|plugin| plugin.loaded)
            .collect()
    }

    /// Toggle plugin state (enable/disable)
    pub async fn toggle_plugin(&mut self, name: &str) -> Result<bool, PluginError> {
        let is_enabled = self.get_plugin(name)
            .ok_or_else(|| PluginError::NotFound(name.to_string()))?
            .enabled;

        if is_enabled {
            self.disable_plugin(name).await?;
            Ok(false)
        } else {
            self.enable_plugin(name).await?;
            Ok(true)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use tokio::fs;

    #[tokio::test]
    async fn test_plugin_discovery_empty_directory() {
        let temp_dir = TempDir::new().unwrap();
        let mut manager = PluginManager::new(temp_dir.path().to_path_buf());
        
        let result = manager.discover_plugins().await;
        assert!(result.is_ok());
        assert_eq!(manager.plugin_count(), 0);
    }

    #[tokio::test]
    async fn test_plugin_discovery_with_valid_plugin() {
        let temp_dir = TempDir::new().unwrap();
        let plugin_dir = temp_dir.path().join("test-plugin");
        fs::create_dir_all(&plugin_dir).await.unwrap();
        
        // Create a valid plugin manifest
        let manifest = PluginManifest {
            name: "test-plugin".to_string(),
            version: "1.0.0".to_string(),
            description: "A test plugin".to_string(),
            author: "Test Author".to_string(),
            main: "index.js".to_string(),
            permissions: vec!["notifications".to_string()],
            engines: {
                let mut engines = HashMap::new();
                engines.insert("baize".to_string(), ">=0.1.0".to_string());
                engines
            },
            keywords: vec!["test".to_string()],
            repository: Some("https://github.com/test/test-plugin".to_string()),
        };
        
        // Write manifest file
        let manifest_content = serde_json::to_string_pretty(&manifest).unwrap();
        fs::write(plugin_dir.join("plugin.json"), manifest_content).await.unwrap();
        
        // Create main file
        fs::write(plugin_dir.join("index.js"), "// Test plugin").await.unwrap();
        
        let mut manager = PluginManager::new(temp_dir.path().to_path_buf());
        let result = manager.discover_plugins().await;
        
        assert!(result.is_ok());
        assert_eq!(manager.plugin_count(), 1);
        assert!(manager.has_plugin("test-plugin"));
        
        let plugin = manager.get_plugin("test-plugin").unwrap();
        assert_eq!(plugin.manifest.name, "test-plugin");
        assert_eq!(plugin.manifest.version, "1.0.0");
        assert!(!plugin.enabled);
        assert!(!plugin.loaded);
    }

    #[tokio::test]
    async fn test_plugin_discovery_with_invalid_manifest() {
        let temp_dir = TempDir::new().unwrap();
        let plugin_dir = temp_dir.path().join("invalid-plugin");
        fs::create_dir_all(&plugin_dir).await.unwrap();
        
        // Create invalid manifest (missing required fields)
        fs::write(plugin_dir.join("plugin.json"), r#"{"name": ""}"#).await.unwrap();
        
        let mut manager = PluginManager::new(temp_dir.path().to_path_buf());
        let result = manager.discover_plugins().await;
        
        // Should succeed but skip invalid plugin
        assert!(result.is_ok());
        assert_eq!(manager.plugin_count(), 0);
    }

    #[test]
    fn test_version_comparison() {
        let manager = PluginManager::new(PathBuf::new());
        
        assert_eq!(manager.compare_versions("1.0.0", "1.0.0"), 0);
        assert_eq!(manager.compare_versions("1.1.0", "1.0.0"), 1);
        assert_eq!(manager.compare_versions("1.0.0", "1.1.0"), -1);
        assert_eq!(manager.compare_versions("2.0.0", "1.9.9"), 1);
    }

    #[test]
    fn test_semver_validation() {
        let manager = PluginManager::new(PathBuf::new());
        
        assert!(manager.is_valid_semver("1.0.0"));
        assert!(manager.is_valid_semver("0.1.0"));
        assert!(manager.is_valid_semver("10.20.30"));
        
        assert!(!manager.is_valid_semver("1.0"));
        assert!(!manager.is_valid_semver("1.0.0.0"));
        assert!(!manager.is_valid_semver("invalid"));
        assert!(!manager.is_valid_semver(""));
    }

    #[test]
    fn test_permission_validation() {
        let manager = PluginManager::new(PathBuf::new());
        
        assert!(manager.is_valid_permission("notifications"));
        assert!(manager.is_valid_permission("storage"));
        assert!(manager.is_valid_permission("dialogs"));
        
        assert!(!manager.is_valid_permission("invalid_permission"));
        assert!(!manager.is_valid_permission(""));
    }

    #[tokio::test]
    async fn test_plugin_lifecycle_management() {
        let temp_dir = TempDir::new().unwrap();
        let plugin_dir = temp_dir.path().join("lifecycle-plugin");
        fs::create_dir_all(&plugin_dir).await.unwrap();
        
        // Create a valid plugin
        let manifest = PluginManifest {
            name: "lifecycle-plugin".to_string(),
            version: "1.0.0".to_string(),
            description: "A lifecycle test plugin".to_string(),
            author: "Test Author".to_string(),
            main: "index.js".to_string(),
            permissions: vec!["notifications".to_string()],
            engines: {
                let mut engines = HashMap::new();
                engines.insert("baize".to_string(), ">=0.1.0".to_string());
                engines
            },
            keywords: vec![],
            repository: None,
        };
        
        let manifest_content = serde_json::to_string_pretty(&manifest).unwrap();
        fs::write(plugin_dir.join("plugin.json"), manifest_content).await.unwrap();
        fs::write(plugin_dir.join("index.js"), "// Lifecycle test plugin").await.unwrap();
        
        let mut manager = PluginManager::new(temp_dir.path().to_path_buf());
        manager.discover_plugins().await.unwrap();
        
        let plugin_name = "lifecycle-plugin";
        
        // Initially plugin should be discovered but not loaded or enabled
        let plugin = manager.get_plugin(plugin_name).unwrap();
        assert!(!plugin.loaded);
        assert!(!plugin.enabled);
        assert!(matches!(plugin.status, PluginStatus::Inactive));
        
        // Load plugin
        manager.load_plugin(plugin_name).await.unwrap();
        let plugin = manager.get_plugin(plugin_name).unwrap();
        assert!(plugin.loaded);
        assert!(!plugin.enabled);
        assert!(matches!(plugin.status, PluginStatus::Inactive));
        
        // Enable plugin
        manager.enable_plugin(plugin_name).await.unwrap();
        let plugin = manager.get_plugin(plugin_name).unwrap();
        assert!(plugin.loaded);
        assert!(plugin.enabled);
        assert!(matches!(plugin.status, PluginStatus::Active));
        
        // Disable plugin
        manager.disable_plugin(plugin_name).await.unwrap();
        let plugin = manager.get_plugin(plugin_name).unwrap();
        assert!(plugin.loaded);
        assert!(!plugin.enabled);
        assert!(matches!(plugin.status, PluginStatus::Inactive));
        
        // Unload plugin
        manager.unload_plugin(plugin_name).await.unwrap();
        let plugin = manager.get_plugin(plugin_name).unwrap();
        assert!(!plugin.loaded);
        assert!(!plugin.enabled);
        assert!(matches!(plugin.status, PluginStatus::Inactive));
    }

    #[tokio::test]
    async fn test_plugin_toggle() {
        let temp_dir = TempDir::new().unwrap();
        let plugin_dir = temp_dir.path().join("toggle-plugin");
        fs::create_dir_all(&plugin_dir).await.unwrap();
        
        // Create a valid plugin
        let manifest = PluginManifest {
            name: "toggle-plugin".to_string(),
            version: "1.0.0".to_string(),
            description: "A toggle test plugin".to_string(),
            author: "Test Author".to_string(),
            main: "index.js".to_string(),
            permissions: vec![],
            engines: HashMap::new(),
            keywords: vec![],
            repository: None,
        };
        
        let manifest_content = serde_json::to_string_pretty(&manifest).unwrap();
        fs::write(plugin_dir.join("plugin.json"), manifest_content).await.unwrap();
        fs::write(plugin_dir.join("index.js"), "// Toggle test plugin").await.unwrap();
        
        let mut manager = PluginManager::new(temp_dir.path().to_path_buf());
        manager.discover_plugins().await.unwrap();
        
        let plugin_name = "toggle-plugin";
        
        // Toggle from disabled to enabled
        let enabled = manager.toggle_plugin(plugin_name).await.unwrap();
        assert!(enabled);
        assert!(manager.get_plugin(plugin_name).unwrap().enabled);
        
        // Toggle from enabled to disabled
        let enabled = manager.toggle_plugin(plugin_name).await.unwrap();
        assert!(!enabled);
        assert!(!manager.get_plugin(plugin_name).unwrap().enabled);
    }

    #[tokio::test]
    async fn test_plugin_error_handling() {
        let temp_dir = TempDir::new().unwrap();
        let plugin_dir = temp_dir.path().join("error-plugin");
        fs::create_dir_all(&plugin_dir).await.unwrap();
        
        // Create a plugin that will trigger an error (name contains "error")
        let manifest = PluginManifest {
            name: "error-plugin".to_string(),
            version: "1.0.0".to_string(),
            description: "An error test plugin".to_string(),
            author: "Test Author".to_string(),
            main: "index.js".to_string(),
            permissions: vec![],
            engines: HashMap::new(),
            keywords: vec![],
            repository: None,
        };
        
        let manifest_content = serde_json::to_string_pretty(&manifest).unwrap();
        fs::write(plugin_dir.join("plugin.json"), manifest_content).await.unwrap();
        fs::write(plugin_dir.join("index.js"), "// Error test plugin").await.unwrap();
        
        let mut manager = PluginManager::new(temp_dir.path().to_path_buf());
        manager.discover_plugins().await.unwrap();
        
        let plugin_name = "error-plugin";
        
        // Try to enable plugin (should fail due to name containing "error")
        let result = manager.enable_plugin(plugin_name).await;
        assert!(result.is_err());
        
        let plugin = manager.get_plugin(plugin_name).unwrap();
        assert!(matches!(plugin.status, PluginStatus::Error(_)));
        assert!(!plugin.enabled);
    }

    #[tokio::test]
    async fn test_plugin_filtering() {
        let temp_dir = TempDir::new().unwrap();
        
        // Create multiple plugins with different states
        for (name, _should_enable) in [("plugin1", true), ("plugin2", false), ("plugin3", true)] {
            let plugin_dir = temp_dir.path().join(name);
            fs::create_dir_all(&plugin_dir).await.unwrap();
            
            let manifest = PluginManifest {
                name: name.to_string(),
                version: "1.0.0".to_string(),
                description: format!("Test plugin {}", name),
                author: "Test Author".to_string(),
                main: "index.js".to_string(),
                permissions: vec![],
                engines: HashMap::new(),
                keywords: vec![],
                repository: None,
            };
            
            let manifest_content = serde_json::to_string_pretty(&manifest).unwrap();
            fs::write(plugin_dir.join("plugin.json"), manifest_content).await.unwrap();
            fs::write(plugin_dir.join("index.js"), format!("// {}", name)).await.unwrap();
        }
        
        let mut manager = PluginManager::new(temp_dir.path().to_path_buf());
        manager.discover_plugins().await.unwrap();
        
        // Enable some plugins
        manager.enable_plugin("plugin1").await.unwrap();
        manager.enable_plugin("plugin3").await.unwrap();
        
        // Test filtering
        let enabled_plugins = manager.get_enabled_plugins();
        assert_eq!(enabled_plugins.len(), 2);
        
        let loaded_plugins = manager.get_loaded_plugins();
        assert_eq!(loaded_plugins.len(), 2); // plugin1 and plugin3 should be loaded
        
        let inactive_plugins = manager.get_plugins_by_status(&PluginStatus::Inactive);
        assert_eq!(inactive_plugins.len(), 1); // plugin2 should be inactive
    }

    #[tokio::test]
    async fn test_plugin_manager_state() {
        let temp_dir = TempDir::new().unwrap();
        let plugin_dir = temp_dir.path().join("state-test-plugin");
        fs::create_dir_all(&plugin_dir).await.unwrap();
        
        // Create a valid plugin
        let manifest = PluginManifest {
            name: "state-test-plugin".to_string(),
            version: "1.0.0".to_string(),
            description: "A state test plugin".to_string(),
            author: "Test Author".to_string(),
            main: "index.js".to_string(),
            permissions: vec!["notifications".to_string()],
            engines: HashMap::new(),
            keywords: vec![],
            repository: None,
        };
        
        let manifest_content = serde_json::to_string_pretty(&manifest).unwrap();
        fs::write(plugin_dir.join("plugin.json"), manifest_content).await.unwrap();
        fs::write(plugin_dir.join("index.js"), "// State test plugin").await.unwrap();
        
        // Test PluginManagerState wrapper
        let manager = PluginManager::new(temp_dir.path().to_path_buf());
        let state = PluginManagerState(RwLock::new(manager));
        
        // Test discovery through state
        {
            let mut manager = state.0.write().await;
            manager.discover_plugins().await.unwrap();
            assert_eq!(manager.plugin_count(), 1);
        }
        
        // Test read access
        {
            let manager = state.0.read().await;
            assert!(manager.has_plugin("state-test-plugin"));
        }
        
        // Test write access for enabling plugin
        {
            let mut manager = state.0.write().await;
            manager.enable_plugin("state-test-plugin").await.unwrap();
            assert!(manager.get_plugin("state-test-plugin").unwrap().enabled);
        }
    }
}

// Tauri Commands for Plugin Management

/// Get list of all plugins
#[tauri::command]
pub async fn get_plugin_list(
    plugin_manager: State<'_, PluginManagerState>,
) -> Result<Vec<PluginInfo>, String> {
    let manager = plugin_manager.0.read().await;
    Ok(manager.get_plugin_list().into_iter().cloned().collect())
}

/// Get a specific plugin by name
#[tauri::command]
pub async fn get_plugin(
    name: String,
    plugin_manager: State<'_, PluginManagerState>,
) -> Result<Option<PluginInfo>, String> {
    let manager = plugin_manager.0.read().await;
    Ok(manager.get_plugin(&name).cloned())
}

/// Discover plugins in the plugin directory
#[tauri::command]
pub async fn discover_plugins(
    plugin_manager: State<'_, PluginManagerState>,
) -> Result<usize, String> {
    let mut manager = plugin_manager.0.write().await;
    manager.discover_plugins().await
        .map_err(|e| e.to_string())?;
    Ok(manager.plugin_count())
}

/// Enable a plugin
#[tauri::command]
pub async fn enable_plugin(
    name: String,
    plugin_manager: State<'_, PluginManagerState>,
) -> Result<(), String> {
    let mut manager = plugin_manager.0.write().await;
    manager.enable_plugin(&name).await
        .map_err(|e| e.to_string())
}

/// Disable a plugin
#[tauri::command]
pub async fn disable_plugin(
    name: String,
    plugin_manager: State<'_, PluginManagerState>,
) -> Result<(), String> {
    let mut manager = plugin_manager.0.write().await;
    manager.disable_plugin(&name).await
        .map_err(|e| e.to_string())
}

/// Toggle plugin state (enable/disable)
#[tauri::command]
pub async fn toggle_plugin(
    name: String,
    plugin_manager: State<'_, PluginManagerState>,
) -> Result<bool, String> {
    let mut manager = plugin_manager.0.write().await;
    manager.toggle_plugin(&name).await
        .map_err(|e| e.to_string())
}

/// Load a plugin
#[tauri::command]
pub async fn load_plugin(
    name: String,
    plugin_manager: State<'_, PluginManagerState>,
) -> Result<(), String> {
    let mut manager = plugin_manager.0.write().await;
    manager.load_plugin(&name).await
        .map_err(|e| e.to_string())
}

/// Unload a plugin
#[tauri::command]
pub async fn unload_plugin(
    name: String,
    plugin_manager: State<'_, PluginManagerState>,
) -> Result<(), String> {
    let mut manager = plugin_manager.0.write().await;
    manager.unload_plugin(&name).await
        .map_err(|e| e.to_string())
}

/// Get plugins by status
#[tauri::command]
pub async fn get_plugins_by_status(
    status: String,
    plugin_manager: State<'_, PluginManagerState>,
) -> Result<Vec<PluginInfo>, String> {
    let manager = plugin_manager.0.read().await;
    
    let target_status = match status.as_str() {
        "active" => PluginStatus::Active,
        "inactive" => PluginStatus::Inactive,
        "loading" => PluginStatus::Loading,
        "error" => PluginStatus::Error("".to_string()), // Will match any error
        _ => return Err("Invalid status".to_string()),
    };
    
    Ok(manager.get_plugins_by_status(&target_status).into_iter().cloned().collect())
}

/// Get enabled plugins
#[tauri::command]
pub async fn get_enabled_plugins(
    plugin_manager: State<'_, PluginManagerState>,
) -> Result<Vec<PluginInfo>, String> {
    let manager = plugin_manager.0.read().await;
    Ok(manager.get_enabled_plugins().into_iter().cloned().collect())
}

/// Get loaded plugins
#[tauri::command]
pub async fn get_loaded_plugins(
    plugin_manager: State<'_, PluginManagerState>,
) -> Result<Vec<PluginInfo>, String> {
    let manager = plugin_manager.0.read().await;
    Ok(manager.get_loaded_plugins().into_iter().cloned().collect())
}

/// Check if a plugin exists
#[tauri::command]
pub async fn has_plugin(
    name: String,
    plugin_manager: State<'_, PluginManagerState>,
) -> Result<bool, String> {
    let manager = plugin_manager.0.read().await;
    Ok(manager.has_plugin(&name))
}

/// Get plugin count
#[tauri::command]
pub async fn get_plugin_count(
    plugin_manager: State<'_, PluginManagerState>,
) -> Result<usize, String> {
    let manager = plugin_manager.0.read().await;
    Ok(manager.plugin_count())
}

/// Initialize plugin manager with app handle
pub fn initialize_plugin_manager(app_handle: &AppHandle) -> PluginManagerState {
    // Get the app's data directory for plugins
    let app_data_dir = app_handle
        .path()
        .app_data_dir()
        .expect("Failed to get app data directory");
    
    let plugin_dir = app_data_dir.join("plugins");
    
    tracing::info!("Initializing plugin manager with directory: {:?}", plugin_dir);
    
    let manager = PluginManager::new(plugin_dir);
    PluginManagerState(RwLock::new(manager))
}

/// Setup plugin manager and discover plugins on app startup
pub async fn setup_plugin_manager(app_handle: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    let plugin_manager_state = initialize_plugin_manager(app_handle);
    
    // Discover plugins on startup
    {
        let mut manager = plugin_manager_state.0.write().await;
        if let Err(e) = manager.discover_plugins().await {
            tracing::error!("Failed to discover plugins on startup: {}", e);
        } else {
            tracing::info!("Discovered {} plugins on startup", manager.plugin_count());
        }
    }
    
    // Manage the plugin manager state
    app_handle.manage(plugin_manager_state);
    
    Ok(())
}