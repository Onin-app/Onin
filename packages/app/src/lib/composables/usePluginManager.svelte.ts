/**
 * Plugin Manager Composable
 *
 * 管理插件内联显示的状态和逻辑
 * 遵循单一职责原则，只处理插件相关功能
 */
import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

export interface PluginState {
  showPluginInline: boolean;
  currentPluginUrl: string;
  currentPluginId: string;
  currentPluginAutoDetach: boolean;
  currentPluginTerminateOnBg: boolean;
  currentPluginRunAtStartup: boolean;
}

export interface PluginManagerReturn {
  // State (reactive)
  state: PluginState;
  // Methods
  closePlugin: () => void;
  detachPlugin: () => Promise<void>;
  toggleAutoDetach: (checked: boolean) => Promise<void>;
  toggleTerminateOnBg: (checked: boolean) => Promise<void>;
  toggleRunAtStartup: (checked: boolean) => Promise<void>;
  openDevTools: () => Promise<void>;
  uninstallPlugin: () => Promise<void>;
  sendLifecycleEvent: (
    event: "show" | "hide" | "focus" | "blur" | "cleanup",
  ) => void;
  // Lifecycle
  setupListeners: () => Promise<UnlistenFn>;
}

/**
 * 创建插件管理器
 *
 * 使用 Svelte 5 runes 管理插件状态
 */
export function usePluginManager(): PluginManagerReturn {
  // ===== State =====
  let state = $state<PluginState>({
    showPluginInline: false,
    currentPluginUrl: "",
    currentPluginId: "",
    currentPluginAutoDetach: false,
    currentPluginTerminateOnBg: false,
    currentPluginRunAtStartup: false,
  });

  // ===== Methods =====

  /**
   * 关闭插件内联显示
   */
  const closePlugin = () => {
    // 内联返回列表时，先通知失焦，再通知隐藏，便于插件做窗口级清理。
    sendLifecycleEvent("blur");
    sendLifecycleEvent("hide");

    // 缓存当前配置，避免清理 state 后丢失判断依据
    const shouldTerminate = state.currentPluginTerminateOnBg;

    state.showPluginInline = false;
    state.currentPluginUrl = "";
    state.currentPluginId = "";
    state.currentPluginAutoDetach = false;
    state.currentPluginTerminateOnBg = false;
    state.currentPluginRunAtStartup = false;

    if (shouldTerminate) {
      sendLifecycleEvent("cleanup");
    }

    // 未勾选 terminate_on_bg 时，仅隐藏并保活；勾选时才销毁
    invoke(
      shouldTerminate ? "close_inline_plugin" : "hide_inline_plugin",
    ).catch(console.error);
  };

  /**
   * 分离插件到独立窗口
   *
   * 必须顺序执行：先关闭 inline webview，再创建独立窗口。
   * 若并发操作，两者同时调用 Win32 窗口管理器会产生死锁导致应用卡死。
   */
  const detachPlugin = async () => {
    if (!state.currentPluginId) return;

    // 保存 pluginId，因为 closePlugin() 会清空 state.currentPluginId
    const pluginId = state.currentPluginId;

    try {
      // 步骤 1：先发送 blur/hide 生命周期事件，并更新 UI 状态
      sendLifecycleEvent("blur");
      sendLifecycleEvent("hide");
      sendLifecycleEvent("cleanup");
      state.showPluginInline = false;
      state.currentPluginUrl = "";
      state.currentPluginId = "";
      state.currentPluginAutoDetach = false;
      state.currentPluginTerminateOnBg = false;
      state.currentPluginRunAtStartup = false;

      // 步骤 2：await 销毁 inline webview，确保完全关闭后再创建新窗口
      await invoke("close_inline_plugin");

      // 步骤 3：打开独立窗口（inline webview 已销毁，无死锁风险）
      await invoke("open_plugin_in_window", { pluginId });
    } catch (error) {
      console.error("Failed to detach plugin:", error);
    }
  };

  /**
   * 切换自动分离设置
   */
  const toggleAutoDetach = async (checked: boolean) => {
    if (!state.currentPluginId) {
      console.error("No current plugin ID");
      return;
    }

    const previousState = state.currentPluginAutoDetach;

    try {
      state.currentPluginAutoDetach = checked;

      await invoke("toggle_plugin_auto_detach", {
        pluginId: state.currentPluginId,
        autoDetach: checked,
      });

      await invoke("show_notification", {
        options: {
          title: checked ? "已启用自动分离" : "已禁用自动分离",
          body: `插件将${checked ? "自动" : "不再自动"}在独立窗口中打开`,
        },
      });
    } catch (error) {
      console.error("Failed to toggle auto detach:", error);
      state.currentPluginAutoDetach = previousState;
      await invoke("show_notification", {
        options: {
          title: "操作失败",
          body: "无法切换自动分离设置",
        },
      });
    }
  };

  /**
   * 发送生命周期事件给插件
   */
  const sendLifecycleEvent = (
    event: "show" | "hide" | "focus" | "blur" | "cleanup",
  ) => {
    invoke("post_inline_plugin_message", {
      message: {
        type: "plugin-lifecycle-event",
        event,
      },
    }).catch((err) => {
      // Ignore error if webview is not ready or closed
      // console.error("Failed to send lifecycle event:", err);
    });
  };

  /**
   * 切换退出到后台立即结束设置
   */
  const toggleTerminateOnBg = async (checked: boolean) => {
    if (!state.currentPluginId) return;
    const previousState = state.currentPluginTerminateOnBg;
    try {
      state.currentPluginTerminateOnBg = checked;
      await invoke("toggle_plugin_terminate_on_bg", {
        pluginId: state.currentPluginId,
        terminateOnBg: checked,
      });
      await invoke("show_notification", {
        options: {
          title: checked ? "已启用后台自动停止" : "已禁用后台自动停止",
          body: `插件在${checked ? "退出到后台后将立即停止" : "退出到后台后将继续运行"}`,
        },
      });
    } catch (error) {
      console.error("Failed to toggle terminate on bg:", error);
      state.currentPluginTerminateOnBg = previousState;
    }
  };

  /**
   * 切换随主程序启动设置
   */
  const toggleRunAtStartup = async (checked: boolean) => {
    if (!state.currentPluginId) return;
    const previousState = state.currentPluginRunAtStartup;
    try {
      state.currentPluginRunAtStartup = checked;
      await invoke("toggle_plugin_run_at_startup", {
        pluginId: state.currentPluginId,
        runAtStartup: checked,
      });
      await invoke("show_notification", {
        options: {
          title: checked ? "已启用随主程序初始化" : "已禁用随主程序初始化",
          body: `插件将${checked ? "随主程序启动时自动运行" : "不再随主程序启动"}`,
        },
      });
    } catch (error) {
      console.error("Failed to toggle run at startup:", error);
      state.currentPluginRunAtStartup = previousState;
    }
  };

  /**
   * 打开开发者工具
   */
  const openDevTools = async () => {
    try {
      await invoke("open_inline_plugin_devtools");
    } catch (error) {
      console.error("Failed to open devtools:", error);
    }
  };

  /**
   * 卸载当前插件
   */
  const uninstallPlugin = async () => {
    if (!state.currentPluginId) return;
    try {
      const pluginId = state.currentPluginId;
      await invoke("uninstall_plugin", { pluginId });
      closePlugin();
      // Should also notify usePluginList to refresh, but it should listen to event
    } catch (error) {
      console.error("Failed to uninstall plugin:", error);
    }
  };

  /**
   * 设置事件监听器
   * 返回清理函数
   */
  const setupListeners = async (): Promise<UnlistenFn> => {
    // 监听插件内联显示事件
    const unlistenPluginInline = await listen<{
      plugin_id: string;
      plugin_name: string;
      plugin_url: string;
    }>("show_plugin_inline", async (event) => {
      state.showPluginInline = true;
      state.currentPluginUrl = event.payload.plugin_url;
      state.currentPluginId = event.payload.plugin_id;

      // 获取插件的 auto_detach 状态
      try {
        const plugin = await invoke<any>("get_plugin_with_schema", {
          pluginId: event.payload.plugin_id,
        });
        state.currentPluginAutoDetach = plugin?.auto_detach ?? false;
        state.currentPluginTerminateOnBg = plugin?.terminate_on_bg ?? false;
        state.currentPluginRunAtStartup = plugin?.run_at_startup ?? false;
      } catch (error) {
        console.error("Failed to get plugin settings:", error);
        state.currentPluginAutoDetach = false;
        state.currentPluginTerminateOnBg = false;
        state.currentPluginRunAtStartup = false;
      }
    });

    // 监听窗口可见性事件
    const unlistenVisibility = await listen<boolean>(
      "window_visibility",
      (event) => {
        const isVisible = event.payload;
        if (!isVisible && state.showPluginInline) {
          closePlugin();
        }
      },
    );

    // 监听分离窗口快捷键事件
    const unlistenDetachWindow = await listen("detach_window_shortcut", () => {
      detachPlugin();
    });

    return () => {
      unlistenPluginInline();
      unlistenDetachWindow();
      unlistenVisibility();
    };
  };

  return {
    get state() {
      return state;
    },
    closePlugin,
    detachPlugin,
    toggleAutoDetach,
    toggleTerminateOnBg,
    toggleRunAtStartup,
    openDevTools,
    uninstallPlugin,
    sendLifecycleEvent,
    setupListeners,
  };
}
