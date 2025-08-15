import { writable, derived } from 'svelte/store';
import type { PluginInfo, PluginError } from '$lib/types/plugin';

// Plugin state store
export const plugins = writable<PluginInfo[]>([]);
export const pluginErrors = writable<PluginError[]>([]);
export const loading = writable<boolean>(false);

// Derived stores for filtered data
export const enabledPlugins = derived(plugins, ($plugins) =>
  $plugins.filter(plugin => plugin.enabled)
);

export const disabledPlugins = derived(plugins, ($plugins) =>
  $plugins.filter(plugin => !plugin.enabled)
);

export const activePlugins = derived(plugins, ($plugins) =>
  $plugins.filter(plugin => plugin.enabled && plugin.loaded && plugin.status === 'active')
);

export const errorPlugins = derived(plugins, ($plugins) =>
  $plugins.filter(plugin => plugin.status === 'error')
);

// Plugin management functions
export const pluginActions = {
  // Load all plugins from backend
  async loadPlugins(): Promise<void> {
    loading.set(true);
    try {
      // TODO: Replace with actual Tauri invoke call
      // const pluginList = await invoke('get_plugin_list');
      
      // Temporary mock data for development
      const mockPlugins: PluginInfo[] = [
        {
          name: "hello-world",
          version: "1.0.0",
          description: "一个简单的 Hello World 插件示例",
          author: "Baize Team",
          enabled: true,
          loaded: true,
          status: "active",
          permissions: ["notifications"],
          path: "./plugins/hello-world"
        },
        {
          name: "advanced-example",
          version: "0.2.1",
          description: "展示高级功能的示例插件",
          author: "Plugin Developer",
          enabled: false,
          loaded: false,
          status: "inactive",
          permissions: ["storage", "events"],
          path: "./plugins/advanced-example"
        }
      ];
      
      plugins.set(mockPlugins);
    } catch (error) {
      console.error('Failed to load plugins:', error);
      pluginErrors.update(errors => [...errors, {
        plugin: 'system',
        message: `Failed to load plugins: ${error}`,
        type: 'load_failed'
      }]);
    } finally {
      loading.set(false);
    }
  },

  // Enable a plugin
  async enablePlugin(pluginName: string): Promise<void> {
    try {
      // TODO: Replace with actual Tauri invoke call
      // await invoke('enable_plugin', { name: pluginName });
      
      // Update local state
      plugins.update(pluginList => 
        pluginList.map(plugin => 
          plugin.name === pluginName 
            ? { ...plugin, enabled: true, loaded: true, status: 'active' as const }
            : plugin
        )
      );

      // Remove any previous errors for this plugin
      pluginErrors.update(errors => 
        errors.filter(error => error.plugin !== pluginName)
      );

      console.log(`Plugin ${pluginName} enabled successfully`);
    } catch (error) {
      console.error(`Failed to enable plugin ${pluginName}:`, error);
      
      // Update plugin status to error
      plugins.update(pluginList => 
        pluginList.map(plugin => 
          plugin.name === pluginName 
            ? { ...plugin, status: 'error' as const }
            : plugin
        )
      );

      // Add error to error store
      pluginErrors.update(errors => [...errors, {
        plugin: pluginName,
        message: `Failed to enable plugin: ${error}`,
        type: 'load_failed'
      }]);
    }
  },

  // Disable a plugin
  async disablePlugin(pluginName: string): Promise<void> {
    try {
      // TODO: Replace with actual Tauri invoke call
      // await invoke('disable_plugin', { name: pluginName });
      
      // Update local state
      plugins.update(pluginList => 
        pluginList.map(plugin => 
          plugin.name === pluginName 
            ? { ...plugin, enabled: false, loaded: false, status: 'inactive' as const }
            : plugin
        )
      );

      // Remove any previous errors for this plugin
      pluginErrors.update(errors => 
        errors.filter(error => error.plugin !== pluginName)
      );

      console.log(`Plugin ${pluginName} disabled successfully`);
    } catch (error) {
      console.error(`Failed to disable plugin ${pluginName}:`, error);
      
      // Add error to error store
      pluginErrors.update(errors => [...errors, {
        plugin: pluginName,
        message: `Failed to disable plugin: ${error}`,
        type: 'load_failed'
      }]);
    }
  },

  // Toggle plugin state
  async togglePlugin(pluginName: string): Promise<void> {
    const currentPlugins = await new Promise<PluginInfo[]>(resolve => {
      const unsubscribe = plugins.subscribe(value => {
        resolve(value);
        unsubscribe();
      });
    });

    const plugin = currentPlugins.find(p => p.name === pluginName);
    if (!plugin) {
      console.error(`Plugin ${pluginName} not found`);
      return;
    }

    if (plugin.enabled) {
      await this.disablePlugin(pluginName);
    } else {
      await this.enablePlugin(pluginName);
    }
  },

  // Refresh plugin list
  async refreshPlugins(): Promise<void> {
    await this.loadPlugins();
  },

  // Clear all errors
  clearErrors(): void {
    pluginErrors.set([]);
  },

  // Clear specific plugin error
  clearPluginError(pluginName: string): void {
    pluginErrors.update(errors => 
      errors.filter(error => error.plugin !== pluginName)
    );
  },

  // Import plugin from file
  async importPlugin(filePath: string): Promise<void> {
    try {
      loading.set(true);
      // TODO: Replace with actual Tauri invoke call
      // await invoke('import_plugin', { path: filePath });
      
      console.log(`Importing plugin from ${filePath}`);
      
      // Refresh plugin list after import
      await this.loadPlugins();
    } catch (error) {
      console.error('Failed to import plugin:', error);
      pluginErrors.update(errors => [...errors, {
        plugin: 'import',
        message: `Failed to import plugin: ${error}`,
        type: 'load_failed'
      }]);
    } finally {
      loading.set(false);
    }
  }
};

// Export a combined store object for easier usage
export const pluginStore = {
  plugins,
  pluginErrors,
  loading,
  enabledPlugins,
  disabledPlugins,
  activePlugins,
  errorPlugins,
  actions: pluginActions
};