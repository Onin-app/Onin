<script lang="ts">
  import { onMount } from "svelte";
  import { listen } from "@tauri-apps/api/event";
  import { escapeHandler } from "$lib/stores/escapeHandler";
  import { requestInputFocus } from "$lib/stores/focusInput";
  import { detachWindowShortcut } from "$lib/stores/shortcuts";
  import { get } from "svelte/store";
  import { invoke } from "@tauri-apps/api/core";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { page } from "$app/state";
  import { goto } from "$app/navigation";
  import { setupPluginConsoleListener } from "$lib/plugin-console";
  import { Toaster, toast } from "svelte-sonner";
  import { startColorPickerFlow } from "$lib/utils/colorPicker";
  import WindowResizer from "$lib/components/WindowResizer.svelte";
  import UpdateDialog from "$lib/components/UpdateDialog.svelte";
  import type { AppConfig } from "$lib/type";
  import {
    updateDialogOpen,
    appVersion,
    latestVersion,
    releaseNotes,
    downloadUrl,
    checkUpdate,
    closeUpdateDialog,
  } from "$lib/stores/update";

  // Setup plugin console listener to forward plugin console output to webview devtools
  setupPluginConsoleListener();

  // 简易且完全兼容 Tauri v2 的 Aptabase 统计上报实现，规避不兼容 Tauri v2 的 @aptabase/tauri npm 包
  async function trackEvent(
    name: string,
    props?: Record<string, string | number>,
  ): Promise<boolean> {
    try {
      await invoke("plugin:aptabase|track_event", { name, props });
      return true;
    } catch (err) {
      console.error("[Aptabase] 追踪事件失败:", err);
      return false;
    }
  }

  // 竞态锁变量：防止 onMount 与 visibility 并发触发重复上报结算
  let isTrackingActive = false;

  // 核心：基于次日结算机制的活跃与频次心跳统计
  async function checkAndTrackActive() {
    if (isTrackingActive) return;
    isTrackingActive = true;

    try {
      const now = new Date();
      const todayStr = `${now.getFullYear()}-${String(now.getMonth() + 1).padStart(2, "0")}-${String(now.getDate()).padStart(2, "0")}`;
      const lastActiveDate = localStorage.getItem("onin_last_active_date");

      if (lastActiveDate !== todayStr) {
        // 1. 跨天了，需要结账上报前一天的打开次数
        const savedCount = localStorage.getItem("onin_today_open_count");

        // 偏执型类型防守：既防范了被外部篡改或空值导致 parseInt 得到 NaN 的崩溃风险，又完美保留了数字 0 的客观语义
        const parsedCount = savedCount !== null ? parseInt(savedCount, 10) : 1;
        const previousDayOpens = isNaN(parsedCount) ? 1 : parsedCount;

        // 发送结算上报，合并昨日唤醒频次
        const success = await trackEvent("app_started", {
          trigger: "active_wake",
          previous_day_opens: previousDayOpens,
        });

        // 2. 只有上报成功后才更新本地标记并清空计数；若失败（如断网），本地保留原数据供下一次唤醒时重试结算
        if (success) {
          localStorage.setItem("onin_last_active_date", todayStr);
          localStorage.setItem("onin_today_open_count", "1");
        }
      } else {
        // 3. 同一天内的后续唤醒，不发网络请求，仅在本地累加计数
        const currentCount = localStorage.getItem("onin_today_open_count");
        const newCount = currentCount ? parseInt(currentCount, 10) + 1 : 1;
        localStorage.setItem("onin_today_open_count", String(newCount));
      }
    } catch (err) {
      console.error("[Aptabase] 活跃心跳统计失败:", err);
    } finally {
      isTrackingActive = false;
    }
  }

  interface ToastPayload {
    message: string;
    kind: "default" | "success" | "error" | "warning" | "info";
    duration?: number;
  }

  // Subscribe to shortcuts store to trigger auto-loading
  // The subscription itself triggers the load in the store's start function
  $detachWindowShortcut;

  // Focus input when navigating to main page
  $effect(() => {
    if (page.route.id === "/") {
      requestInputFocus();
    }
  });

  // This onMount block sets up a single, persistent listener for the 'esc_key_pressed' event.
  // It will live for the entire duration of the app, avoiding setup/teardown during page navigation.
  onMount(() => {
    // 首次冷启动时，如果窗口处于可见状态，才上报跨天活跃统计，避开静默后台开机自启
    getCurrentWindow()
      .isVisible()
      .then((visible) => {
        if (visible) {
          checkAndTrackActive();
        }
      });

    const listenersPromise = (async () => {
      // Restore Listener:
      // Listen to "escape_pressed" (standardized event)
      // BUT only handle it if we are NOT on the main page.
      // The main page (+page.svelte) has its own listener.
      const unlisten = await listen("escape_pressed", () => {
        // Check if we are on the main page
        if (page.route.id === "/") {
          // Do NOTHING. Let +page.svelte handle it.
        } else {
          // If not on main page, check if there is a registered handler
          const handler = get(escapeHandler);
          if (handler && typeof handler === "function") {
            handler();
          } else {
            // Fallback
            window.history.back();
          }
        }
      });

      const unlistenVisibility = await listen<boolean>(
        "window_visibility",
        (event) => {
          // When window becomes visible, check if we are on the main page.
          if (event.payload && page.route.id === "/") {
            requestInputFocus();
          }
          if (event.payload) {
            // 当窗口重新变为可见时（唤醒时），触发每日活跃结算或本地累加
            checkAndTrackActive();
          }
        },
      );

      const unlistenCommand = await listen<string>(
        "execute_command_by_name",
        async (event) => {
          const commandName = event.payload;

          if (commandName === "extension:color:pick") {
            await startColorPickerFlow({
              closeOnSuccess: false,
              restoreMainWindow: false,
              useToastOverlay: true,
            });
            return;
          }

          // Handle page routing for global shortcuts of extensions
          if (commandName === "extension:ai:chat") {
            goto("/extensions/ai").then(() => {
              invoke("show_main_window_cmd");
            });
            return;
          }
          if (commandName === "extension:clipboard:history") {
            goto("/extensions/clipboard").then(() => {
              invoke("show_main_window_cmd");
            });
            return;
          }
          if (commandName === "extension:emoji:search") {
            goto("/extensions/emoji").then(() => {
              invoke("show_main_window_cmd");
            });
            return;
          }
          if (commandName === "extension:file_search:search") {
            goto("/extensions/filesearch").then(() => {
              invoke("show_main_window_cmd");
            });
            return;
          }
          if (commandName === "extension:color:convert") {
            goto("/extensions/color").then(() => {
              invoke("show_main_window_cmd");
            });
            return;
          }

          invoke("execute_command", { name: commandName });
        },
      );

      const unlistenToast = await listen<ToastPayload>(
        "plugin-toast",
        (event) => {
          const { message, kind, duration } = event.payload;
          const options = duration ? { duration } : {};

          switch (kind) {
            case "success":
              toast.success(message, options);
              break;
            case "error":
              toast.error(message, options);
              break;
            case "warning":
              toast.warning(message, options);
              break;
            case "info":
              toast.info(message, options);
              break;
            default:
              toast(message, options);
              break;
          }
        },
      );

      return {
        unlisten,
        unlistenVisibility,
        unlistenCommand,
        unlistenToast,
      };
    })();

    let autoUpdateIntervalId: ReturnType<typeof setInterval> | null = null;

    // 加载配置判定是否执行自动检查更新
    const setupAutoCheckUpdate = async () => {
      try {
        const config = await invoke<AppConfig>("get_app_config");
        if (config.auto_check_update ?? true) {
          // 启动后延迟 2 秒，避免阻塞窗口首屏密集渲染
          setTimeout(() => {
            checkUpdate(true);
          }, 2000);

          // 注册每 12 小时的后台轮询检测 (12 * 60 * 60 * 1000 = 43200000ms)
          autoUpdateIntervalId = setInterval(() => {
            checkUpdate(true);
          }, 43200000);
        }
      } catch (err) {
        console.error("加载自动检查更新配置失败:", err);
      }
    };

    setupAutoCheckUpdate();

    // The returned cleanup function will only run if the entire layout is destroyed.
    return () => {
      if (autoUpdateIntervalId) {
        clearInterval(autoUpdateIntervalId);
      }
      listenersPromise.then(
        ({ unlisten, unlistenVisibility, unlistenCommand, unlistenToast }) => {
          unlisten();
          unlistenVisibility();
          unlistenCommand();
          unlistenToast();
        },
      );
    };
  });

  let { children } = $props();
</script>

{@render children()}

<WindowResizer />
<Toaster richColors position="top-center" />

<UpdateDialog
  bind:open={$updateDialogOpen}
  currentVersion={$appVersion}
  latestVersion={$latestVersion}
  releaseNotes={$releaseNotes}
  downloadUrl={$downloadUrl}
  onClose={closeUpdateDialog}
/>
