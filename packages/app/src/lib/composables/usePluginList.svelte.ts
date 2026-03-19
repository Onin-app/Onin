/**
 * Plugin List Manager Composable
 *
 * 管理插件列表的状态和逻辑
 * 遵循单一职责原则，只处理插件列表相关功能
 */
import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { toast } from "svelte-sonner";
import type { PluginSettingsSchema } from "$lib/types/plugin-settings";

export interface PluginManifest {
  id: string;
  name: string;
  version: string;
  description: string;
  entry: string;
  author?: string;
  icon?: string;
  downloads?: number;
  stars?: number;
  enabled?: boolean;
  auto_detach?: boolean;
  terminate_on_bg?: boolean;
  run_at_startup?: boolean;
  settings?: PluginSettingsSchema;
  dir_name?: string;
  install_source?: "local" | "marketplace";
}

export interface PluginListState {
  plugins: PluginManifest[];
  searchQuery: string;
  imageErrors: Set<string>;
}

export interface PluginListReturn {
  state: PluginListState;
  // Methods
  loadPlugins: (forceReload?: boolean) => Promise<void>;
  refreshPlugins: () => Promise<void>;
  importPlugin: () => Promise<void>;
  togglePlugin: (pluginId: string, enabled: boolean) => Promise<void>;
  toggleAutoDetach: (pluginId: string, autoDetach: boolean) => Promise<void>;
  toggleTerminateOnBg: (
    pluginId: string,
    terminateOnBg: boolean,
  ) => Promise<void>;
  toggleRunAtStartup: (
    pluginId: string,
    runAtStartup: boolean,
  ) => Promise<void>;
  uninstallPlugin: (pluginId: string) => Promise<void>;
  executePlugin: (pluginId: string) => Promise<void>;
  handleImageError: (pluginId: string) => void;
  setSearchQuery: (query: string) => void;
  // Computed
  filteredPlugins: PluginManifest[];
  // Lifecycle
  setupListeners: () => Promise<UnlistenFn>;
}

/**
 * 创建插件列表管理器
 */
export function usePluginList(): PluginListReturn {
  // ===== State =====
  let state = $state<PluginListState>({
    plugins: [],
    searchQuery: "",
    imageErrors: new Set(),
  });

  // ===== Computed =====
  const filteredPlugins = $derived(
    state.plugins.filter((plugin) =>
      plugin.name.toLowerCase().includes(state.searchQuery.toLowerCase()),
    ),
  );

  // ===== Methods =====

  const loadPlugins = async (forceReload = false) => {
    try {
      const command = forceReload ? "load_plugins" : "get_loaded_plugins";
      const result = await invoke<PluginManifest[]>(command);
      state.plugins = result.map((plugin) => ({
        ...plugin,
        stars: plugin.stars ?? Math.floor(Math.random() * 1000),
        downloads: plugin.downloads ?? Math.floor(Math.random() * 10000),
        enabled: plugin.enabled ?? true,
      }));
    } catch (error) {
      console.error("Failed to load plugins via invoke:", error);
    }
  };

  const refreshPlugins = async () => {
    try {
      const result = await invoke<PluginManifest[]>("refresh_plugins");
      state.plugins = result.map((plugin) => ({
        ...plugin,
        stars: plugin.stars ?? Math.floor(Math.random() * 1000),
        downloads: plugin.downloads ?? Math.floor(Math.random() * 10000),
      }));

      await invoke("show_notification", {
        options: {
          title: "刷新成功",
          body: `已刷新 ${state.plugins.length} 个插件`,
        },
      });
    } catch (error) {
      console.error("刷新插件失败:", error);

      await invoke("show_notification", {
        options: {
          title: "刷新失败",
          body: "无法刷新插件列表，请重试",
        },
      });
    }
  };

  const importPlugin = async () => {
    try {
      await invoke("acquire_window_close_lock");

      const { open } = await import("@tauri-apps/plugin-dialog");

      const selected = await open({
        directory: true,
        multiple: false,
        title: "选择插件目录",
      });

      await invoke("release_window_close_lock");

      if (!selected) {
        return;
      }

      const result = await invoke<PluginManifest>("import_plugin", {
        sourcePath: selected,
      });

      await loadPlugins(false);

      await invoke("show_notification", {
        options: {
          title: "导入成功",
          body: `插件 ${result.name} 已成功导入`,
        },
      });
    } catch (error) {
      console.error("导入插件失败:", error);

      try {
        await invoke("release_window_close_lock");
      } catch (unlockError) {
        console.error("释放窗口锁失败:", unlockError);
      }

      await invoke("show_notification", {
        options: {
          title: "导入失败",
          body: String(error),
        },
      });
    }
  };

  const togglePlugin = async (pluginId: string, enabled: boolean) => {
    const plugin = state.plugins.find(
      (p) => p.dir_name === pluginId || p.id === pluginId,
    );
    const pluginName = plugin?.name || pluginId;

    try {
      await invoke("toggle_plugin", { pluginId, enabled });

      state.plugins = state.plugins.map((p) => {
        const pId = p.dir_name || p.id;
        return pId === pluginId ? { ...p, enabled } : p;
      });

      try {
        await invoke("refresh_commands");
      } catch (refreshError) {
        console.error("Failed to refresh commands:", refreshError);
      }

      await invoke("show_notification", {
        options: {
          title: enabled ? "插件已启用" : "插件已禁用",
          body: `${pluginName} ${enabled ? "已启用" : "已禁用"}`,
        },
      });
    } catch (error) {
      console.error("Failed to toggle plugin:", error);

      await invoke("show_notification", {
        options: {
          title: "操作失败",
          body: `无法${enabled ? "启用" : "禁用"}插件 ${pluginName}`,
        },
      });
    }
  };

  const toggleAutoDetach = async (pluginId: string, autoDetach: boolean) => {
    try {
      await invoke("toggle_plugin_auto_detach", { pluginId, autoDetach });
      state.plugins = state.plugins.map((p) => {
        const pId = p.dir_name || p.id;
        return pId === pluginId ? { ...p, auto_detach: autoDetach } : p;
      });
    } catch (error) {
      console.error("Failed to toggle auto detach:", error);
    }
  };

  const toggleTerminateOnBg = async (
    pluginId: string,
    terminateOnBg: boolean,
  ) => {
    try {
      await invoke("toggle_plugin_terminate_on_bg", {
        pluginId,
        terminateOnBg,
      });
      state.plugins = state.plugins.map((p) => {
        const pId = p.dir_name || p.id;
        return pId === pluginId ? { ...p, terminate_on_bg: terminateOnBg } : p;
      });
    } catch (error) {
      console.error("Failed to toggle terminate on bg:", error);
    }
  };

  const toggleRunAtStartup = async (
    pluginId: string,
    runAtStartup: boolean,
  ) => {
    try {
      await invoke("toggle_plugin_run_at_startup", { pluginId, runAtStartup });
      state.plugins = state.plugins.map((p) => {
        const pId = p.dir_name || p.id;
        return pId === pluginId ? { ...p, run_at_startup: runAtStartup } : p;
      });
    } catch (error) {
      console.error("Failed to toggle run at startup:", error);
    }
  };

  const uninstallPlugin = async (pluginId: string) => {
    const plugin = state.plugins.find((p) => p.id === pluginId);
    const pluginName = plugin?.name || pluginId;

    try {
      await invoke("uninstall_plugin", { pluginId });

      try {
        const refreshedPlugins =
          await invoke<PluginManifest[]>("refresh_plugins");
        state.plugins = refreshedPlugins.map((plugin) => ({
          ...plugin,
          downloads: plugin.downloads ?? Math.floor(Math.random() * 10000),
        }));
      } catch (refreshError) {
        console.error("Failed to refresh plugins:", refreshError);
        state.plugins = state.plugins.filter((p) => p.id !== pluginId);
      }

      try {
        await invoke("refresh_commands");
      } catch (refreshError) {
        console.error("Failed to refresh commands:", refreshError);
      }

      await invoke("show_notification", {
        options: {
          title: "卸载成功",
          body: `插件 ${pluginName} 已卸载`,
        },
      });
    } catch (error) {
      console.error("Failed to uninstall plugin:", error);

      await invoke("show_notification", {
        options: {
          title: "卸载失败",
          body: `无法卸载插件 ${pluginName}: ${error}`,
        },
      });
    }
  };

  const executePlugin = async (pluginId: string) => {
    try {
      await invoke("execute_plugin_entry", { pluginId });
    } catch (error) {
      const plugin = state.plugins.find(
        (p) => (p.dir_name || p.id) === pluginId || p.id === pluginId,
      );
      const formatted = formatPluginExecutionError(plugin, error);

      console.error(`Failed to execute plugin with ID ${pluginId}:`, error);

      toast.error(formatted.title, {
        id: `plugin-execute-error:${pluginId}`,
        description: formatted.body,
      });

      await invoke("show_notification", {
        options: {
          title: formatted.title,
          body: formatted.body,
        },
      });
    }
  };

  const handleImageError = (pluginId: string) => {
    state.imageErrors.add(pluginId);
    state.imageErrors = state.imageErrors; // 触发响应式更新
  };

  const formatPluginExecutionError = (
    plugin: PluginManifest | undefined,
    error: unknown,
  ) => {
    const pluginName = plugin?.name || plugin?.id || "插件";
    const rawMessage =
      error instanceof Error
        ? error.message
        : typeof error === "string"
          ? error
          : String(error);

    if (rawMessage.includes("插件入口文件未找到")) {
      return {
        title: "插件入口缺失",
        body: `${pluginName} 的入口文件不存在，请检查插件包是否完整。`,
      };
    }

    if (
      rawMessage.includes("background.js") ||
      rawMessage.includes("后台入口")
    ) {
      return {
        title: "插件后台入口缺失",
        body: `${pluginName} 缺少 dist/background.js，设置、命令注册和启动初始化可能不会生效。`,
      };
    }

    if (rawMessage.includes("devMode=true 但未指定 devServer")) {
      return {
        title: "开发模式配置不完整",
        body: `${pluginName} 启用了 devMode，但没有可用的 devServer。`,
      };
    }

    return {
      title: "打开插件失败",
      body: `${pluginName}: ${rawMessage}`,
    };
  };

  const setSearchQuery = (query: string) => {
    state.searchQuery = query;
  };

  const setupListeners = async (): Promise<UnlistenFn> => {
    const unlistenInstalled = await listen<string>(
      "plugin-installed",
      async () => {
        await loadPlugins(true);
      },
    );

    const unlistenSchema = await listen<string>(
      "plugin-settings-schema-registered",
      async (event) => {
        const pluginId = event.payload;

        try {
          const updatedPlugin = await invoke<PluginManifest>(
            "get_plugin_with_schema",
            {
              pluginId,
            },
          );

          state.plugins = state.plugins.map((p) =>
            p.id === pluginId ? { ...p, ...updatedPlugin } : p,
          );
        } catch (error) {
          console.error(
            "[Plugins Page] Failed to refresh plugin schema:",
            error,
          );
        }
      },
    );

    const unlistenError = await listen<{
      plugin_id: string;
      plugin_name: string;
      error: string;
    }>("plugin-init-error", async (event) => {
      const { plugin_id, plugin_name, error } = event.payload;
      console.error("[Plugins Page] Plugin init error:", event.payload);

      toast.error("插件初始化失败", {
        id: `plugin-init-error:${plugin_id}`,
        description: `${plugin_name}: ${error}`,
      });

      await invoke("show_notification", {
        options: {
          title: "插件初始化失败",
          body: `${plugin_name}: ${error}`,
        },
      });
    });

    const unlistenSuccess = await listen<string>(
      "plugin-init-success",
      (event) => {},
    );

    return () => {
      unlistenInstalled();
      unlistenSchema();
      unlistenError();
      unlistenSuccess();
    };
  };

  return {
    get state() {
      return state;
    },
    get filteredPlugins() {
      return filteredPlugins;
    },
    loadPlugins,
    refreshPlugins,
    importPlugin,
    togglePlugin,
    toggleAutoDetach,
    toggleTerminateOnBg,
    toggleRunAtStartup,
    uninstallPlugin,
    executePlugin,
    handleImageError,
    setSearchQuery,
    setupListeners,
  };
}
