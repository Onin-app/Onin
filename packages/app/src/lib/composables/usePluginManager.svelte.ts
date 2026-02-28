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
}

export interface PluginManagerReturn {
  // State (reactive)
  state: PluginState;
  // Methods
  closePlugin: () => void;
  detachPlugin: () => Promise<void>;
  toggleAutoDetach: (checked: boolean) => Promise<void>;
  handlePluginMessage: (event: MessageEvent) => Promise<void>; // Keep for compatibility if needed, but likely unused
  sendLifecycleEvent: (event: "show" | "hide" | "focus" | "blur") => void;
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
  });

  // ===== Methods =====

  /**
   * 关闭插件内联显示
   */
  const closePlugin = () => {
    // 发送隐藏事件给插件
    sendLifecycleEvent("hide");

    state.showPluginInline = false;
    state.currentPluginUrl = "";
    state.currentPluginId = "";
    state.currentPluginAutoDetach = false;

    // Create side effect to close (destroy) webview
    invoke("close_inline_plugin").catch(console.error);
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
      // 步骤 1：先发送 hide 生命周期事件，并更新 UI 状态
      sendLifecycleEvent("hide");
      state.showPluginInline = false;
      state.currentPluginUrl = "";
      state.currentPluginId = "";
      state.currentPluginAutoDetach = false;

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
   * 处理来自插件的消息
   * (Native Webview 模式下不再需要代理 invoke，但保留空函数以防调用)
   */
  const handlePluginMessage = async (event: MessageEvent) => {
    // No-op for native webview
  };

  /**
   * 发送生命周期事件给插件
   */
  const sendLifecycleEvent = (event: "show" | "hide" | "focus" | "blur") => {
    invoke("send_inline_plugin_message", {
      message: {
        type: "plugin-lifecycle-event",
        event,
      },
    }).catch((err) => {
      // Ignore error if webview is not ready or closed
      // console.error("Failed to send lifecycle event:", err);
    });
    console.log("[PluginManager] Sent lifecycle event:", event);
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
      console.log(
        "Received show_plugin_inline event for:",
        event.payload.plugin_id,
      );

      state.showPluginInline = true;
      state.currentPluginUrl = event.payload.plugin_url;
      state.currentPluginId = event.payload.plugin_id;

      // 获取插件的 auto_detach 状态
      try {
        const plugin = await invoke<any>("get_plugin_with_schema", {
          pluginId: event.payload.plugin_id,
        });
        state.currentPluginAutoDetach = plugin?.auto_detach ?? false;
        console.log(
          `Plugin ${event.payload.plugin_id} auto_detach state:`,
          state.currentPluginAutoDetach,
        );
      } catch (error) {
        console.error("Failed to get plugin auto_detach state:", error);
        state.currentPluginAutoDetach = false;
      }
    });

    // 监听分离窗口快捷键事件
    const unlistenDetachWindow = await listen("detach_window_shortcut", () => {
      console.log("Detach window shortcut triggered");
      detachPlugin();
    });

    return () => {
      unlistenPluginInline();
      unlistenDetachWindow();
    };
  };

  return {
    get state() {
      return state;
    },
    closePlugin,
    detachPlugin,
    toggleAutoDetach,
    handlePluginMessage,
    sendLifecycleEvent,
    setupListeners,
  };
}
