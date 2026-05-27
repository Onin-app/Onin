<script lang="ts">
  import { onMount } from "svelte";
  import { listen } from "@tauri-apps/api/event";
  import { escapeHandler } from "$lib/stores/escapeHandler";
  import { requestInputFocus } from "$lib/stores/focusInput";
  import { detachWindowShortcut } from "$lib/stores/shortcuts";
  import { get } from "svelte/store";
  import { invoke } from "@tauri-apps/api/core";
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
  ) {
    try {
      await invoke("plugin:aptabase|track_event", { name, props });
    } catch (err) {
      console.error("[Aptabase] 追踪事件失败:", err);
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
    // 跟踪应用启动事件，用以统计日活/用户数
    trackEvent("app_started");

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
