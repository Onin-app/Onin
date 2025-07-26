use std::collections::HashMap;
use tauri::{AppHandle, Manager, Runtime, WebviewWindow, WebviewUrl, WebviewWindowBuilder};
use crate::plugin_manifest::PluginManifest;

pub struct PluginWebviewManager<R: Runtime> {
    pub webviews: HashMap<String, WebviewWindow<R>>,
    app_handle: AppHandle<R>,
}

impl<R: Runtime> PluginWebviewManager<R> {
    pub fn new(app_handle: AppHandle<R>) -> Self {
        Self {
            webviews: HashMap::new(),
            app_handle,
        }
    }

    pub fn create_plugin_webview(&mut self, manifest: &PluginManifest) -> Result<(), String> {
        let plugin_id = &manifest.id;
        let entry = &manifest.entry;
        let label = format!("plugin-{}", plugin_id.replace('.', "_"));

        let plugins_dir = self.app_handle.path().app_data_dir().map_err(|e| e.to_string())?.join("plugins");
        let plugin_path_segment = manifest.path.as_ref().ok_or("Plugin path not set in manifest")?;
        let entry_path = plugins_dir.join(plugin_path_segment).join(entry);

        let url_string = format!("file://{}", entry_path.to_string_lossy());
        let url = WebviewUrl::External(url_string.parse().unwrap());

        let window = WebviewWindowBuilder::new(&self.app_handle, label, url)
            .title(&manifest.name)
            .visible(false)
            .build()
            .map_err(|e| e.to_string())?;

        self.webviews.insert(plugin_id.clone(), window);

        Ok(())
    }

    pub fn get_plugin_webview(&self, plugin_id: &str) -> Option<&WebviewWindow<R>> {
        self.webviews.get(plugin_id)
    }
}