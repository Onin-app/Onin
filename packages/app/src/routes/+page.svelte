<script lang="ts">
  import { onDestroy, onMount } from "svelte";
  import { get } from "svelte/store";
  import autoAnimate from "@formkit/auto-animate";
  import type { Action } from "svelte/action";
  import { DropdownMenu, ScrollArea } from "bits-ui";
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
  import { WebviewWindow } from "@tauri-apps/api/webviewWindow";
  import {
    AppWindow,
    DotsThreeVertical,
    X,
    PuzzlePiece,
    CaretRight,
    Check,
    CaretLeft,
  } from "phosphor-svelte";
  import { goto } from "$app/navigation";
  import { fuzzyMatch } from "$lib/utils/fuzzyMatch";
  import { getMatchedCommands } from "$lib/utils/matchCommand";
  import { sortByUsage } from "$lib/utils/sortByUsage";
  import {
    Theme,
    type LaunchableItem,
    type CommandUsageStats,
    type AppConfig,
  } from "$lib/type";
  import { theme, getTheme } from "$lib/utils/theme";
  import { escapeHandler } from "$lib/stores/escapeHandler";
  import { focusInputTrigger } from "$lib/stores/focusInput";
  import { detachWindowShortcut } from "$lib/stores/shortcuts";
  import PhosphorIcon from "$lib/components/PhosphorIcon.svelte";
  import FileAttachment from "$lib/components/FileAttachment.svelte";
  import TextAttachment from "$lib/components/TextAttachment.svelte";

  import "../index.css";

  // AutoAnimate action for file attachments
  const animate: Action<HTMLElement> = (node) => {
    autoAnimate(node, {
      duration: 200,
      easing: "ease-in-out",
    });
  };

  let inputValue = $state<string>("");
  let originAppList = $state<LaunchableItem[]>([]);
  let appList = $state<LaunchableItem[]>([]);
  let matchedCommands = $state<LaunchableItem[]>([]);
  let selectedIndex = $state<number>(0);
  let currentTheme = $state<Theme>(Theme.DARK);
  let unlisten = $state<null | (() => void)>(null);
  let attachedFiles = $state<File[]>([]);
  let showAllFiles = $state<boolean>(false);
  let attachedText = $state<string>(""); // 粘贴的文本内容
  let usageStats = $state<CommandUsageStats[]>([]);
  let appConfig = $state<AppConfig>({
    auto_paste_time_limit: 5,
    auto_clear_time_limit: 0,
    sort_mode: "smart",
    enable_usage_tracking: true,
  });

  // Plugin inline display state
  let showPluginInline = $state<boolean>(false);
  let currentPluginUrl = $state<string>("");
  let currentPluginId = $state<string>("");
  let currentPluginAutoDetach = $state<boolean>(false);
  let pluginIframeElement = $state<HTMLIFrameElement | null>(null);

  // Refresh state for progress indicator
  let isRefreshing = $state<boolean>(false);

  // Input element reference
  let inputElement: HTMLInputElement;

  // Listen to focus requests from layout
  $effect(() => {
    $focusInputTrigger; // Subscribe to changes
    queueMicrotask(() => inputElement?.focus());
  });

  // Focus input when plugin is closed
  $effect(() => {
    if (!showPluginInline) {
      queueMicrotask(() => inputElement?.focus());
    }
  });

  // Forward main window visibility events to plugin iframe
  let mainWindowVisible = $state<boolean>(true);

  const handleEsc = () => {
    // If plugin is showing inline, close it first
    if (showPluginInline) {
      // Send hide event to plugin iframe before closing
      if (pluginIframeElement && pluginIframeElement.contentWindow) {
        pluginIframeElement.contentWindow.postMessage(
          {
            type: "plugin-lifecycle-event",
            event: "hide",
          },
          "*",
        );
      }

      showPluginInline = false;
      currentPluginUrl = "";
      currentPluginId = "";
      return;
    }
    inputValue = "";
    attachedText = "";
    clearAttachments();
    appList = originAppList;
    selectedIndex = 0;
    invoke("close_main_window");
  };

  // Handle Tauri API calls from plugin iframe
  const handlePluginMessage = async (event: MessageEvent) => {
    if (event.data?.type !== "plugin-tauri-call") return;

    const { messageId, command, args } = event.data;
    const iframe = document.querySelector("iframe");
    if (!iframe?.contentWindow) return;

    try {
      let result;
      if (command === "invoke") {
        result = await invoke(args[0], args[1] || {});
      } else if (command === "emit") {
        result = null; // Handle emit if needed
      }

      iframe.contentWindow.postMessage({ messageId, result }, "*");
    } catch (error) {
      iframe.contentWindow.postMessage(
        {
          messageId,
          error: error instanceof Error ? error.message : String(error),
        },
        "*",
      );
    }
  };

  onMount(async () => {
    console.log("Main page component has mounted");
    escapeHandler.set(handleEsc);

    // Set up plugin message handler
    pluginMessageHandler = handlePluginMessage;
    window.addEventListener("message", handlePluginMessage);

    // 加载配置和使用统计
    try {
      appConfig = await invoke<AppConfig>("get_app_config");
      usageStats = await invoke<CommandUsageStats[]>("get_usage_stats");
      console.log("Loaded config and usage stats:", { appConfig, usageStats });
    } catch (error) {
      console.error("Failed to load config or usage stats:", error);
    }

    // 1. 立即获取一次数据
    await fetchApps();

    // 监听窗口显示事件，自动粘贴剪贴板内容
    const unlistenWindowShow = await listen<boolean>(
      "window_visibility",
      async (event) => {
        mainWindowVisible = event.payload;

        if (event.payload) {
          await autoPasteClipboard();
        }

        // Forward visibility event to plugin iframe
        if (pluginIframeElement && pluginIframeElement.contentWindow) {
          const eventType = event.payload ? "show" : "hide";
          pluginIframeElement.contentWindow.postMessage(
            {
              type: "plugin-lifecycle-event",
              event: eventType,
            },
            "*",
          );
        }
      },
    );

    // Fetch initial data. The visibility listener is now handled in the layout.
    // (async () => {
    //   const res = await invoke<AppInfo[]>("get_installed_apps");
    //   if (res) {
    //     originAppList = res;
    //     appList = res;
    //   }
    // })();

    // 2. 监听后端的更新通知
    const unlistenAppsUpdated = await listen("apps_updated", (event) => {
      console.log(
        "Received apps_updated event from backend. Refetching list...",
      );
      fetchApps();
    });

    const unlistenCommandsReady = await listen("commands_ready", (event) => {
      console.log(
        "Received commands_ready event from backend. Refetching list...",
      );
      fetchApps();
    });

    // Listen for plugin inline display events
    const unlistenPluginInline = await listen<{
      plugin_id: string;
      plugin_name: string;
      plugin_url: string;
    }>("show_plugin_inline", async (event) => {
      console.log(
        "Received show_plugin_inline event for:",
        event.payload.plugin_id,
      );
      showPluginInline = true;
      currentPluginUrl = event.payload.plugin_url;
      currentPluginId = event.payload.plugin_id;

      // Fetch plugin auto_detach state
      try {
        const plugin = await invoke<any>("get_plugin_with_schema", {
          pluginId: event.payload.plugin_id,
        });
        console.log("Plugin data received:", plugin);
        // LoadedPlugin uses #[serde(flatten)] so manifest fields are at top level
        currentPluginAutoDetach = plugin?.auto_detach ?? false;
        console.log(
          `Plugin ${event.payload.plugin_id} auto_detach state:`,
          currentPluginAutoDetach,
        );
      } catch (error) {
        console.error("Failed to get plugin auto_detach state:", error);
        currentPluginAutoDetach = false;
      }
    });

    // Listen for detach window shortcut event
    const unlistenDetachWindow = await listen("detach_window_shortcut", () => {
      console.log("Detach window shortcut triggered");
      handleDetachPlugin();
    });

    // Listen for clear app clipboard event
    const unlistenClearClipboard = await listen("clear_app_clipboard", () => {
      console.log("Clearing app clipboard content");
      attachedText = "";
      attachedFiles = [];
      showAllFiles = false;
    });

    // Listen for refresh events for progress indicator
    const unlistenRefreshStarted = await listen("refresh_started", () => {
      console.log("Refresh started");
      isRefreshing = true;
    });

    const unlistenRefreshEnded = await listen<{
      previous_count: number;
      current_count: number;
      added: number;
    }>("commands_refreshed", async (event) => {
      console.log("Refresh ended", event.payload);
      isRefreshing = false;
      fetchApps(); // Refetch the updated list

      // Show notification with count information
      const { current_count, added } = event.payload;
      let message = `共 ${current_count} 项`;
      if (added > 0) {
        message += `，新增 ${added} 项`;
      } else if (added < 0) {
        message += `，减少 ${Math.abs(added)} 项`;
      }
      await invoke("show_notification", {
        options: {
          title: "刷新成功",
          body: message,
        },
      });
    });

    unlisten = () => {
      unlistenAppsUpdated();
      unlistenCommandsReady();
      unlistenPluginInline();
      unlistenDetachWindow();
      unlistenWindowShow();
      unlistenClearClipboard();
      unlistenRefreshStarted();
      unlistenRefreshEnded();
    };
  });

  const fetchApps = async () => {
    try {
      console.log("Fetching all launchable items...");
      const res = await invoke<LaunchableItem[]>("get_all_launchable_items");
      console.log("本机软件列表: ", res);
      if (res) {
        originAppList = res;
        // 后端已经排序了，直接使用
        appList = res;
      }
      console.log(`Got ${appList.length} apps.`);
    } catch (error) {
      console.error("Failed to get all launchable items:", error);
    }
  };

  const unsubscribe = theme.subscribe((value) => {
    currentTheme = value;
  });

  // MIME 类型映射表
  const MIME_TYPES: Record<string, string> = {
    // 图片
    jpg: "image/jpeg",
    jpeg: "image/jpeg",
    png: "image/png",
    gif: "image/gif",
    webp: "image/webp",
    svg: "image/svg+xml",
    bmp: "image/bmp",
    ico: "image/x-icon",
    // 视频
    mp4: "video/mp4",
    webm: "video/webm",
    avi: "video/x-msvideo",
    mov: "video/quicktime",
    // 音频
    mp3: "audio/mpeg",
    wav: "audio/wav",
    ogg: "audio/ogg",
    oga: "audio/ogg",
    m4a: "audio/mp4",
    // 文档
    pdf: "application/pdf",
    doc: "application/msword",
    docx: "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
    xls: "application/vnd.ms-excel",
    xlsx: "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
    ppt: "application/vnd.ms-powerpoint",
    pptx: "application/vnd.openxmlformats-officedocument.presentationml.presentation",
    // 压缩包
    zip: "application/zip",
    rar: "application/x-rar-compressed",
    "7z": "application/x-7z-compressed",
    tar: "application/x-tar",
    gz: "application/gzip",
    // 代码
    js: "text/javascript",
    ts: "text/typescript",
    json: "application/json",
    html: "text/html",
    css: "text/css",
    xml: "text/xml",
    py: "text/x-python",
    java: "text/x-java",
    cpp: "text/x-c++src",
    c: "text/x-csrc",
    rs: "text/x-rust",
    go: "text/x-go",
    // 文本
    txt: "text/plain",
    md: "text/markdown",
  };

  const handleInput = (
    e: Event & { currentTarget: EventTarget & HTMLInputElement },
  ) => {
    const value = e.currentTarget.value;
    let apps = fuzzyMatch(value, originAppList);
    // 应用使用频率排序
    apps = sortByUsage(
      apps,
      usageStats,
      appConfig.sort_mode,
      appConfig.enable_usage_tracking,
    );
    inputValue = value;
    appList = apps;
    selectedIndex = 0;
    updateMatchedCommands();
  };

  // 更新匹配的命令
  const updateMatchedCommands = () => {
    matchedCommands = getMatchedCommands(
      originAppList,
      attachedText,
      attachedFiles,
    );
  };

  const handlePaste = async (e: ClipboardEvent) => {
    const items = e.clipboardData?.items;
    if (!items) return;

    const files: File[] = [];
    for (let i = 0; i < items.length; i++) {
      const item = items[i];
      if (item.kind === "file") {
        const file = item.getAsFile();
        if (file) {
          files.push(file);
        }
      }
    }

    if (files.length > 0) {
      e.preventDefault();
      attachedFiles = [...attachedFiles, ...files];
      // Debug: console.log("Attached files count:", files.length);
      updateMatchedCommands();
    }
  };

  // 根据文件扩展名获取 MIME 类型
  const getMimeType = (fileName: string): string => {
    const ext = fileName.split(".").pop()?.toLowerCase() || "";
    return MIME_TYPES[ext] || "application/octet-stream";
  };

  // 自动粘贴剪贴板内容
  const autoPasteClipboard = async () => {
    try {
      const clipboardContent = await invoke<{
        text?: string;
        files?: Array<{ path: string; name: string; is_directory: boolean }>;
        timestamp?: number;
      }>("get_clipboard_content");

      // 获取配置的时间限制
      const config = await invoke<{ auto_paste_time_limit: number }>(
        "get_app_config",
      );
      const timeLimit = config.auto_paste_time_limit;

      console.log("Auto paste config:", {
        timeLimit,
        timestamp: clipboardContent.timestamp,
        fullConfig: config,
      });

      // 如果设置了时间限制（不为0），检查剪贴板内容的时间
      if (timeLimit > 0) {
        // 如果没有时间戳，说明内容已被清空或无效，不自动粘贴
        if (!clipboardContent.timestamp) {
          console.log("No timestamp available, skipping auto-paste");
          return;
        }

        const clipboardTimestamp = clipboardContent.timestamp;
        const currentTime = Math.floor(Date.now() / 1000); // 当前时间（秒）
        const timeDiff = currentTime - clipboardTimestamp;

        console.log("Time check:", {
          clipboardTimestamp,
          currentTime,
          timeDiff,
          timeLimit,
          shouldPaste: timeDiff <= timeLimit,
        });

        // 如果时间差超过限制，不自动粘贴
        if (timeDiff > timeLimit) {
          console.log(
            `Clipboard content is too old (${timeDiff}s > ${timeLimit}s), skipping auto-paste`,
          );
          return;
        }

        console.log(
          `Clipboard content is recent (${timeDiff}s <= ${timeLimit}s), auto-pasting`,
        );
      }

      // 处理文件路径
      if (clipboardContent.files && clipboardContent.files.length > 0) {
        const files: File[] = [];

        for (const fileInfo of clipboardContent.files) {
          // 文件夹不需要读取内容
          if (fileInfo.is_directory) {
            const placeholderBlob = new Blob([]);
            const file = new File([placeholderBlob], fileInfo.name, {
              type: "application/x-directory",
            });
            Object.defineProperty(file, "path", {
              value: fileInfo.path,
              writable: false,
            });
            files.push(file);
            continue;
          }

          // 主应用不需要读取文件内容，只需要文件路径和元信息
          // 根据文件扩展名获取正确的 MIME 类型
          const mimeType = getMimeType(fileInfo.name);

          // 创建一个占位符文件对象（只包含元信息，不包含实际内容）
          const placeholderBlob = new Blob([], { type: mimeType });
          const file = new File([placeholderBlob], fileInfo.name, {
            type: mimeType,
          });

          // 添加路径属性，插件可以通过这个路径读取文件
          Object.defineProperty(file, "path", {
            value: fileInfo.path,
            writable: false,
          });

          files.push(file);
        }

        if (files.length > 0) {
          // 粘贴文件时清空文本
          inputValue = "";
          attachedText = "";
          attachedFiles = files;
          updateMatchedCommands();
        }
      }
      // 处理文本内容
      else if (clipboardContent.text) {
        const text = clipboardContent.text.trim();
        // 粘贴文本时清空附件和输入框
        attachedFiles = [];
        showAllFiles = false;
        inputValue = "";
        attachedText = text;
        updateMatchedCommands();
      }

      // 确保输入框获得焦点
      queueMicrotask(() => inputElement?.focus());
    } catch (error) {
      console.error("Failed to auto-paste clipboard:", error);
    }
  };

  const handleDrop = (e: DragEvent) => {
    e.preventDefault();
    const files = Array.from(e.dataTransfer?.files || []);
    if (files.length > 0) {
      attachedFiles = [...attachedFiles, ...files];
      console.log("Dropped files:", attachedFiles);
      updateMatchedCommands();
    }
  };

  const handleDragOver = (e: DragEvent) => {
    e.preventDefault();
  };

  // 清理附件和相关状态
  const clearAttachments = () => {
    attachedFiles = [];
    showAllFiles = false;
    attachedText = "";
    matchedCommands = [];
  };

  const removeFile = (index: number) => {
    attachedFiles = attachedFiles.filter((_, i) => i !== index);
    // 如果删除后只剩1个或0个文件，自动收起
    if (attachedFiles.length <= 1) {
      showAllFiles = false;
    }
    updateMatchedCommands();
  };

  const openApp = async (app: LaunchableItem) => {
    try {
      if (app.action) {
        // 准备附件数据
        const args: any = {};

        // 添加输入框内容
        if (inputValue) {
          args.input = inputValue;
        }

        // 添加粘贴的文本内容
        if (attachedText) {
          args.text = attachedText;
        }

        // 分类并添加文件
        if (attachedFiles.length > 0) {
          const images: any[] = [];
          const textFiles: any[] = [];
          const otherFiles: any[] = [];
          const folders: any[] = [];

          attachedFiles.forEach((file) => {
            const filePath = (file as any).path;
            const fileInfo = {
              name: file.name,
              path: filePath || "",
              type: file.type,
              size: file.size,
            };

            if (file.type === "application/x-directory") {
              folders.push(fileInfo);
            } else if (file.type.startsWith("image/")) {
              images.push(fileInfo);
            } else if (
              file.type === "text/plain" ||
              file.type === "text/markdown" ||
              file.name.endsWith(".txt") ||
              file.name.endsWith(".md")
            ) {
              textFiles.push(fileInfo);
            } else {
              otherFiles.push(fileInfo);
            }
          });

          // 添加分类后的文件
          if (images.length > 0) args.images = images;
          if (textFiles.length > 0) args.textFiles = textFiles;
          if (otherFiles.length > 0) args.files = otherFiles;
          if (folders.length > 0) args.folders = folders;
        }

        await invoke("execute_command", {
          name: app.action,
          window: await WebviewWindow.getCurrent(),
          args: Object.keys(args).length > 0 ? args : null,
        });

        // 刷新使用统计和列表（异步，不阻塞）
        Promise.all([
          invoke<CommandUsageStats[]>("get_usage_stats"),
          invoke<LaunchableItem[]>("get_all_launchable_items"),
        ])
          .then(([stats, items]) => {
            usageStats = stats;
            originAppList = items;
            // 如果当前没有搜索，更新显示列表
            if (!inputValue) {
              appList = items;
            }
          })
          .catch((err) => console.error("Failed to refresh data:", err));
      } else if (app.source === "FileCommand") {
        // Handle custom items that might not have an action
        await invoke("open_app", {
          path: app.path,
          window: await WebviewWindow.getCurrent(),
        });
      }
      inputValue = "";
      attachedText = "";
      clearAttachments();
      appList = originAppList;
      selectedIndex = 0;
    } catch (error) {
      console.error("Failed to open app:", error);
    }
  };

  // 编辑文本附件
  const editTextAttachment = () => {
    inputValue = attachedText;
    attachedText = "";
    matchedCommands = [];
    queueMicrotask(() => {
      inputElement?.focus();
      inputElement?.select();
    });
  };

  const handleKeyDown = (e: KeyboardEvent) => {
    // 优先显示匹配命令
    const displayList = matchedCommands.length > 0 ? matchedCommands : appList;

    if (e.key === "ArrowDown" || (e.key === "Tab" && !e.shiftKey)) {
      e.preventDefault();
      selectedIndex =
        selectedIndex === displayList.length - 1 ? 0 : selectedIndex + 1;
    } else if (e.key === "ArrowUp" || (e.key === "Tab" && e.shiftKey)) {
      e.preventDefault();
      selectedIndex =
        selectedIndex === 0 ? displayList.length - 1 : selectedIndex - 1;
    } else if (e.key === "Enter" && displayList.length > 0) {
      e.preventDefault();
      openApp(displayList[selectedIndex]);
    }

    // 保持选中项在可见范围内
    const container = document.querySelector(".app-list");
    const selectedItem = container?.children[selectedIndex];
    if (selectedItem) {
      selectedItem.scrollIntoView({
        behavior: "auto",
        block: "nearest",
      });
    }
  };

  const handleToSettings = () => {
    goto("/settings");
  };

  const handleClosePlugin = () => {
    showPluginInline = false;
    currentPluginUrl = "";
    currentPluginId = "";
    currentPluginAutoDetach = false;
  };

  const handleToggleAutoDetach = async (
    event: boolean | { detail: { checked: boolean } },
  ) => {
    let checked: boolean;

    // 检查是否是事件对象还是直接的布尔值
    if (typeof event === "object" && "detail" in event) {
      checked = event.detail.checked;
    } else {
      checked = event as boolean;
    }

    if (!currentPluginId) {
      console.error("No current plugin ID");
      return;
    }

    // 保存之前的状态以备回滚
    const previousState = currentPluginAutoDetach;

    try {
      // 先更新本地状态以提供即时反馈
      currentPluginAutoDetach = checked;

      await invoke("toggle_plugin_auto_detach", {
        pluginId: currentPluginId,
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
      // 如果保存失败，恢复原状态
      currentPluginAutoDetach = previousState;
      await invoke("show_notification", {
        options: {
          title: "操作失败",
          body: "无法切换自动分离设置",
        },
      });
    }
  };

  const handleDetachPlugin = async () => {
    if (!currentPluginId) return;

    try {
      // Open plugin in separate window
      await invoke("open_plugin_in_window", { pluginId: currentPluginId });
      // Close inline display
      handleClosePlugin();
    } catch (error) {
      console.error("Failed to detach plugin:", error);
    }
  };

  let pluginMessageHandler: ((event: MessageEvent) => void) | null = null;

  onDestroy(() => {
    // Clean up the theme subscription
    unsubscribe && unsubscribe();
    // As a safeguard, reset the escape handler if it's still ours
    if (get(escapeHandler) === handleEsc) {
      escapeHandler.set(() => {});
    }
    // 组件销毁时，清理监听器
    if (unlisten) {
      unlisten();
    }
    // Clean up plugin message handler
    if (pluginMessageHandler) {
      window.removeEventListener("message", pluginMessageHandler);
    }
  });
</script>

<main
  class="h-[100vh] w-full overflow-hidden rounded-xl bg-neutral-100 p-4 text-neutral-900 dark:bg-neutral-800 dark:text-neutral-100"
  data-tauri-drag-region
>
  <div
    class="flex h-full w-full flex-col"
    role="listbox"
    tabindex="0"
    onkeydown={handleKeyDown}
  >
    <div class="flex items-center gap-2 pb-2">
      <button class="flex-shrink-0 cursor-pointer" onclick={handleToSettings}>
        <img
          src="/ff_logo_{getTheme(currentTheme) === Theme.DARK
            ? Theme.LIGHT
            : Theme.DARK}.svg"
          class="h-10 w-10"
          alt="Tauri logo"
        />
      </button>
      <div
        class="flex w-full {showAllFiles
          ? 'flex-col gap-2'
          : 'flex-row items-center gap-2'} rounded-lg border border-neutral-300 bg-white px-2 py-2 dark:border-neutral-600 dark:bg-neutral-800"
        ondrop={handleDrop}
        ondragover={handleDragOver}
        role="region"
        aria-label="输入区域"
      >
        {#if attachedText}
          <div use:animate class="flex flex-wrap items-center gap-1.5">
            <TextAttachment
              text={attachedText}
              onEdit={editTextAttachment}
              onRemove={() => {
                attachedText = "";
              }}
            />
          </div>
        {:else if attachedFiles.length > 0}
          <div use:animate class="flex flex-wrap items-center gap-1.5">
            {#if showAllFiles}
              <!-- 展开模式：显示所有文件 -->
              {#each attachedFiles as file, index (file.name + index)}
                <FileAttachment {file} onRemove={() => removeFile(index)} />
              {/each}
            {:else}
              <!-- 折叠模式：只显示第一个文件 -->
              <FileAttachment
                file={attachedFiles[0]}
                onRemove={() => removeFile(0)}
              />
            {/if}
            {#if attachedFiles.length > 1}
              <button
                class="inline-flex h-[34px] items-center gap-1 rounded-md border px-2 text-sm font-medium transition-colors {showAllFiles
                  ? 'border-blue-300 bg-blue-50 text-blue-700 hover:bg-blue-100 dark:border-blue-600 dark:bg-blue-900/30 dark:text-blue-300 dark:hover:bg-blue-900/50'
                  : 'border-orange-300 bg-orange-50 text-orange-700 hover:bg-orange-100 dark:border-orange-600 dark:bg-orange-900/30 dark:text-orange-300 dark:hover:bg-orange-900/50'}"
                onclick={() => {
                  showAllFiles = !showAllFiles;
                }}
                aria-label={showAllFiles ? "收起文件" : "展开所有文件"}
              >
                {#if showAllFiles}
                  <CaretLeft class="size-4" weight="bold" />
                  <span>收起</span>
                {:else}
                  <span>+{attachedFiles.length - 1}</span>
                  <CaretRight class="size-4" weight="bold" />
                {/if}
              </button>
            {/if}
          </div>
        {/if}
        <input
          bind:this={inputElement}
          class="{showAllFiles
            ? 'w-full'
            : 'min-w-0 flex-1'} h-[34px] bg-transparent text-2xl focus:ring-0 focus:outline-none active:ring-0 active:outline-none"
          type="text"
          placeholder="Hi Onin!"
          bind:value={inputValue}
          oninput={handleInput}
          onpaste={handlePaste}
          onkeydown={(e) => {
            if (e.key === "Backspace" && inputValue === "") {
              e.preventDefault();
              if (attachedText) {
                // 文本附件：进入编辑模式并全选
                editTextAttachment();
              } else if (attachedFiles.length > 0) {
                // 文件附件：展开时删除最后一个，折叠时删除所有
                if (showAllFiles) {
                  removeFile(attachedFiles.length - 1);
                } else {
                  attachedFiles = [];
                  showAllFiles = false;
                }
              }
            }
          }}
        />
      </div>
      <div class="flex-shrink-0">
        {#if showPluginInline}
          <DropdownMenu.Root>
            <DropdownMenu.Trigger class="ml-2 cursor-pointer">
              <DotsThreeVertical class="size-8" />
            </DropdownMenu.Trigger>
            <DropdownMenu.Portal>
              <DropdownMenu.Content
                class="border-muted bg-background shadow-popover rounded-xl border px-1 py-1.5 outline-hidden focus-visible:outline-hidden"
                sideOffset={8}
              >
                <DropdownMenu.Item
                  class="rounded-button data-highlighted:bg-muted flex h-10 items-center py-3 pr-1.5 pl-3 text-sm font-medium ring-0! ring-transparent! select-none focus-visible:outline-none"
                >
                  <button
                    class="flex w-full cursor-pointer items-center"
                    onclick={handleDetachPlugin}
                  >
                    <AppWindow class="text-foreground-alt mr-2 size-5" />
                    <span class="mr-2">分离窗口</span>
                    {#if $detachWindowShortcut}
                      <kbd
                        class=" rounded-button border-dark-10 bg-background-alt text-muted-foreground shadow-kbd ml-auto inline-flex items-center justify-center border px-1 text-xs uppercase"
                      >
                        {$detachWindowShortcut}
                      </kbd>
                    {/if}
                  </button>
                </DropdownMenu.Item>
                <DropdownMenu.Item
                  class="rounded-button data-highlighted:bg-muted flex h-10 items-center py-3 pr-1.5 pl-3 text-sm font-medium ring-0! ring-transparent! select-none focus-visible:outline-none"
                >
                  <button
                    class="flex w-full cursor-pointer items-center"
                    onclick={handleClosePlugin}
                  >
                    <X class="text-foreground-alt mr-2 size-5" />
                    <span class="mr-2">关闭插件</span>
                    <kbd
                      class=" rounded-button border-dark-10 bg-background-alt text-muted-foreground shadow-kbd ml-auto inline-flex items-center justify-center border px-1 text-xs"
                    >
                      ESC
                    </kbd>
                  </button>
                </DropdownMenu.Item>
                <DropdownMenu.CheckboxItem
                  bind:checked={currentPluginAutoDetach}
                  class="rounded-button data-highlighted:bg-muted flex h-10 cursor-pointer items-center py-3 pr-1.5 pl-3 text-sm font-medium ring-0! ring-transparent! select-none focus-visible:outline-none"
                  onCheckedChange={handleToggleAutoDetach}
                >
                  {#snippet children({ checked })}
                    <div class="flex items-center pr-4">
                      <PuzzlePiece class="text-foreground-alt mr-2 size-5" />
                      自动分离为独立窗口
                    </div>
                    <div class="ml-auto flex items-center gap-px">
                      {#if checked}
                        <Check class="size-4" />
                      {/if}
                    </div>
                  {/snippet}
                </DropdownMenu.CheckboxItem>
              </DropdownMenu.Content>
            </DropdownMenu.Portal>
          </DropdownMenu.Root>
        {/if}
      </div>
    </div>

    <div class="relative flex-1 overflow-hidden">
      <!-- Refresh progress bar (absolute positioned, no layout shift) -->
      {#if isRefreshing}
        <div
          class="absolute top-0 left-0 z-10 h-0.5 w-full overflow-hidden bg-neutral-200 dark:bg-neutral-700"
        >
          <div
            class="refresh-progress-bar h-full w-1/3 bg-gradient-to-r from-blue-400 via-blue-500 to-blue-400"
          ></div>
        </div>
      {/if}
      {#if showPluginInline}
        <!-- Plugin inline display area -->
        <!-- 使用本地 HTTP 服务器加载插件，支持所有资源和动态导入 -->
        <iframe
          bind:this={pluginIframeElement}
          src={currentPluginUrl}
          class="h-full w-full border-0"
          title="Plugin"
          allow="clipboard-read; clipboard-write"
          onload={() => {
            // Send show event to plugin iframe after it loads
            if (pluginIframeElement && pluginIframeElement.contentWindow) {
              pluginIframeElement.contentWindow.postMessage(
                {
                  type: "plugin-lifecycle-event",
                  event: "show",
                },
                "*",
              );
            }
          }}
        ></iframe>
      {:else}
        <ScrollArea.Root
          class="h-full w-full rounded-[10px] border px-2 py-2"
          type="hover"
        >
          <ScrollArea.Viewport class="h-full w-full">
            <!-- App list display area -->
            <div class="app-list">
              <div use:animate>
                {#each matchedCommands.length > 0 ? matchedCommands : appList as app, index (app.path + app.name)}
                  <button
                    role="option"
                    aria-selected={selectedIndex === index}
                    class="flex w-full rounded p-2 text-left text-2xl transition-all duration-200 {selectedIndex !==
                    index
                      ? 'hover:bg-neutral-200 dark:hover:bg-neutral-700'
                      : ''} {selectedIndex === index
                      ? 'bg-neutral-300 dark:bg-neutral-600'
                      : ''}"
                    onclick={() => openApp(app)}
                  >
                    {#if app.icon}
                      {#if app.icon_type === "Base64"}
                        <img
                          src={`data:image/png;base64,${app.icon}`}
                          class="mr-2 inline-block h-8 w-8"
                          alt=""
                        />
                      {:else}
                        <!-- 所有其他情况使用 Phosphor 图标 -->
                        <div
                          class="mr-2 flex h-8 w-8 items-center justify-center rounded-md bg-gray-200 dark:bg-gray-700"
                        >
                          <PhosphorIcon icon={app.icon} class="h-6 w-6" />
                        </div>
                      {/if}
                    {/if}
                    <div class="flex flex-1 flex-col">
                      <div class="flex items-center justify-between">
                        <span>
                          {app.name}
                        </span>
                        <span
                          class="rounded-md bg-neutral-200 px-1.5 py-0.5 text-xs text-neutral-700 dark:bg-neutral-700 dark:text-neutral-300"
                        >
                          {app.source_display || app.source}
                        </span>
                      </div>
                      {#if app.source !== "Command"}
                        <span
                          class="text-neutral-399 text-xs dark:text-neutral-500"
                        >
                          {app.path}
                        </span>
                      {/if}
                    </div>
                  </button>
                {/each}
              </div>
            </div>
          </ScrollArea.Viewport>
          <ScrollArea.Scrollbar
            orientation="vertical"
            class="bg-muted hover:bg-dark-10 data-[state=visible]:animate-in data-[state=hidden]:animate-out data-[state=hidden]:fade-out-0 data-[state=visible]:fade-in-0 flex w-1.5 touch-none rounded-full border-l border-l-transparent p-px transition-all duration-200 select-none hover:w-3"
          >
            <ScrollArea.Thumb class="bg-muted-foreground flex-1 rounded-full" />
          </ScrollArea.Scrollbar>
          <ScrollArea.Scrollbar
            orientation="horizontal"
            class="bg-muted hover:bg-dark-10 flex h-1.5 touch-none rounded-full border-t border-t-transparent p-px transition-all duration-200 select-none hover:h-3"
          >
            <ScrollArea.Thumb class="bg-muted-foreground rounded-full" />
          </ScrollArea.Scrollbar>
          <ScrollArea.Corner />
        </ScrollArea.Root>
      {/if}
    </div>
  </div>
</main>
