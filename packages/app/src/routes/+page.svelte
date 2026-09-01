<script lang="ts">
  /**
   * Main Page Component
   *
   * 应用主页面 - 重构后版本
   * 使用 composables 和提取的组件实现关注点分离
   *
   * 职责：
   * - 组合各个 composables
   * - 协调组件之间的交互
   * - 处理页面级别的生命周期
   */
  import { onDestroy, onMount } from "svelte";
  import { get } from "svelte/store";
  import autoAnimate from "@formkit/auto-animate";
  import type { Action } from "svelte/action";
  import { ScrollArea } from "$lib/components/ui/scroll-area";
  import {
    Tooltip,
    TooltipTrigger,
    TooltipContent,
    TooltipProvider,
  } from "$lib/components/ui/tooltip";
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { goto } from "$app/navigation";
  import { page } from "$app/state";

  // Stores
  import { Theme, type LaunchableItem } from "$lib/type";
  import { theme, getTheme } from "$lib/utils/theme";
  import { startColorPickerFlow } from "$lib/utils/colorPicker";
  import { takeScreenshot } from "$lib/utils/screenshot";
  import {
    resolveExtensionAction,
    buildNavigateRoute,
    type ExtensionContext,
  } from "$lib/utils/extensionActions";
  import { escapeHandler } from "$lib/stores/escapeHandler";
  import { focusInputTrigger, requestInputFocus } from "$lib/stores/focusInput";
  import { detachWindowShortcut } from "$lib/stores/shortcuts";
  import { hasNewVersion, latestVersion, appVersion } from "$lib/stores/update";
  import { accumulateCommandStat } from "$lib/tracking";

  // Composables
  import { usePluginManager } from "$lib/composables/usePluginManager.svelte";
  import { useClipboardManager } from "$lib/composables/useClipboardManager.svelte";
  import { useAppList } from "$lib/composables/useAppList.svelte";
  import { useExtensionManager } from "$lib/composables/useExtensionManager.svelte";

  // Components
  import SearchInput from "$lib/components/SearchInput.svelte";
  import AppListItem from "$lib/components/AppListItem.svelte";
  import PluginMenu from "$lib/components/PluginMenu.svelte";
  import RefreshProgressBar from "$lib/components/RefreshProgressBar.svelte";
  import PluginInlineView from "$lib/components/PluginInlineView.svelte";
  import ExtensionResultItem from "$lib/components/ExtensionResultItem.svelte";
  import ConfirmDialog from "$lib/components/ConfirmDialog.svelte";

  import "../index.css";

  // ===== Composables =====
  const plugin = usePluginManager();
  const clipboard = useClipboardManager();
  const appListManager = useAppList();
  const extensionManager = useExtensionManager();

  // ===== Local State =====
  let inputValue = $state<string>("");
  let matchedCommands = $state<LaunchableItem[]>([]);
  let extensionPreviewItem = $state<LaunchableItem | null>(null);
  let currentTheme = $state<Theme>(Theme.DARK);
  let unlisten = $state<null | (() => void)>(null);
  let removeWindowEscapeListener = $state<null | (() => void)>(null);

  let lastMouseX = $state<number>(0);
  let lastMouseY = $state<number>(0);

  const handleMouseMove = (e: MouseEvent) => {
    lastMouseX = e.clientX;
    lastMouseY = e.clientY;
  };

  const handleItemHover = (index: number, e: MouseEvent) => {
    if (e.clientX === lastMouseX && e.clientY === lastMouseY) {
      return;
    }
    appListManager.state.selectedIndex = index;
  };

  // Component references
  let searchInputRef: SearchInput;
  let pluginInlineViewRef = $state<PluginInlineView | null>(null);

  // Confirm dialog state
  let confirmDialogOpen = $state(false);
  let confirmDialogTitle = $state("");
  let confirmDialogDescription = $state("");
  let pendingAction = $state<(() => void | Promise<void>) | null>(null);

  // AutoAnimate action
  const animate: Action<HTMLElement> = (node) => {
    autoAnimate(node, {
      duration: 200,
      easing: "ease-in-out",
    });
  };

  // ===== Computed =====
  // 合并匹配命令和搜索结果并去重
  // 优先级：Extension 预览 -> 精确/模糊匹配 -> 匹配指令
  // 若同一命令既被模糊匹配也满足匹配指令规则，优先保留支持传参执行的“匹配指令”版本
  const displayList = $derived.by(() => {
    const result: LaunchableItem[] = [];

    // Extension 预览优先显示在最顶部（如计算器结果）
    if (extensionPreviewItem) {
      result.push(extensionPreviewItem);
    }

    // 判断是否处于纯粘贴状态：有粘贴的附件（文本或文件），且输入框中没有手动输入
    const isPurePasteState =
      (clipboard.state.attachedText ||
        clipboard.state.attachedFiles.length > 0) &&
      !inputValue.trim();

    const rawList = isPurePasteState
      ? [...matchedCommands]
      : [...appListManager.state.appList, ...matchedCommands];

    const itemMap = new Map<string, LaunchableItem>();
    const order: string[] = [];

    for (const item of rawList) {
      const key = `${item.source}:${item.name}`;
      if (!itemMap.has(key)) {
        itemMap.set(key, item);
        order.push(key);
      } else {
        // 遇到重复的项，若新的项有更具体的匹配触发模式，或者新的项包含 action 而旧 of 没有 action，则覆盖保留
        const existing = itemMap.get(key)!;
        if (
          item.trigger_mode === "matched" ||
          (!existing.action && item.action)
        ) {
          itemMap.set(key, item);
        }
      }
    }

    const uniqueApps = order.map((key) => itemMap.get(key) as LaunchableItem);
    return [...result, ...uniqueApps];
  });

  // ===== Effects =====
  // 监听 focus 请求
  $effect(() => {
    $focusInputTrigger;
    queueMicrotask(() => searchInputRef?.focus());
  });

  // 插件关闭时聚焦输入框
  $effect(() => {
    if (!plugin.state.showPluginInline) {
      queueMicrotask(() => searchInputRef?.focus());
    }
  });

  // ===== Event Handlers =====

  const handleEsc = async () => {
    // Only handle ESC on main page
    if (page.route.id !== "/") {
      return;
    }

    if (plugin.state.showPluginInline) {
      invoke("acquire_window_close_lock").catch(console.error);
      await plugin.closePlugin();
      requestInputFocus();
      setTimeout(() => {
        invoke("release_window_close_lock").catch(console.error);
      }, 200);
      return;
    }

    inputValue = "";
    clipboard.clearAttachments();
    matchedCommands = [];
    appListManager.resetToOriginList();

    // 隐藏窗口前主动释放焦点，重置 activeElement 状态，防止混淆下次打开时的焦点判定
    if (typeof document !== "undefined" && document.activeElement) {
      try {
        (document.activeElement as HTMLElement).blur();
      } catch (e) {
        console.error(e);
      }
    }

    invoke("close_main_window");
  };

  let extensionPreviewTimer: any = null;

  const handleInput = async (value: string) => {
    inputValue = value;
    appListManager.handleInput(value);
    updateMatchedCommands();
    updateExtensionManagerPreviewDebounced();
  };

  // 更新 Extension 预览（带防抖，适用于高频打字）
  const updateExtensionManagerPreviewDebounced = () => {
    if (extensionPreviewTimer) {
      clearTimeout(extensionPreviewTimer);
    }
    extensionPreviewTimer = setTimeout(async () => {
      await updateExtensionManagerPreview();
      extensionPreviewTimer = null;
    }, 50);
  };

  // 更新 Extension 预览（计算器等）
  const updateExtensionManagerPreview = async () => {
    // 优先使用粘贴的文本，其次使用输入框的值
    const effectiveText = clipboard.state.attachedText || inputValue;
    await extensionManager.getPreview(effectiveText);
    extensionPreviewItem = extensionManager.getPreviewAsItem();
  };

  const updateMatchedCommands = () => {
    matchedCommands = clipboard
      .getMatchedCommands(appListManager.state.originAppList, inputValue)
      .map((cmd) => ({
        ...cmd,
        trigger_mode: "matched" as const,
      }));
    appListManager.resetSelection();
  };

  const handlePaste = async (e: ClipboardEvent) => {
    await clipboard.handlePaste(e);
    updateMatchedCommands();
    await updateExtensionManagerPreview();
  };

  const handleDrop = (e: DragEvent) => {
    clipboard.handleDrop(e);
    updateMatchedCommands();
  };

  const handleRemoveFile = (index: number) => {
    clipboard.removeFile(index);
    updateMatchedCommands();
  };

  const handleBackspace = () => {
    if (clipboard.state.attachedText) {
      clipboard.editTextAttachment((text) => {
        inputValue = text;
        queueMicrotask(() => {
          searchInputRef?.focus();
          searchInputRef?.select();
        });
      });
      matchedCommands = [];
    } else if (clipboard.state.attachedFiles.length > 0) {
      if (clipboard.state.showAllFiles) {
        clipboard.removeFile(clipboard.state.attachedFiles.length - 1);
      } else {
        clipboard.clearAttachments();
      }
      updateMatchedCommands();
    }
  };

  const handleEditText = () => {
    clipboard.editTextAttachment((text) => {
      inputValue = text;
      queueMicrotask(() => {
        searchInputRef?.focus();
        searchInputRef?.select();
      });
    });
    matchedCommands = [];
  };

  const resetLauncherState = () => {
    inputValue = "";
    clipboard.clearAttachments();
    extensionPreviewItem = null;
    extensionManager.clearPreview();
    matchedCommands = [];
    appListManager.resetToOriginList();
  };

  const startColorPickCommand = async () => {
    await startColorPickerFlow({
      beforeStart: resetLauncherState,
      onCancel: requestInputFocus,
      closeOnSuccess: false,
      restoreMainWindow: false,
      useToastOverlay: true,
    });
  };

  // 解析 Extension Action 字符串（格式: "extension:id:code"）
  const parseExtensionAction = (
    action: string | undefined,
  ): { extensionId: string; commandCode: string } | null => {
    if (!action || !action.startsWith("extension:")) return null;
    const parts = action.split(":");
    if (parts.length >= 3) {
      return { extensionId: parts[1], commandCode: parts[2] };
    }
    return null;
  };

  // ===== Extension 执行辅助函数 =====

  /** 清除所有启动器临时状态（不含窗口操作） */
  const clearLauncherState = () => {
    inputValue = "";
    clipboard.clearAttachments();
    extensionPreviewItem = null;
    extensionManager.clearPreview();
    matchedCommands = [];
    appListManager.resetToOriginList();
  };

  /** 重置启动器状态并关闭主窗口 */
  const resetLauncherAndClose = () => {
    clearLauncherState();
    invoke("close_main_window");
  };

  /** 重置启动器状态并跳转路由 */
  const resetAndGoto = (route: string) => {
    clearLauncherState();
    goto(route);
  };

  /** 执行 Extension 命令并关闭（结果可复制） */
  const runExtensionExecute = async (
    extensionId: string,
    commandCode: string,
    text: string = "",
  ) => {
    const effectiveText = text || clipboard.state.attachedText || inputValue;
    const result = await extensionManager.execute(
      extensionId,
      commandCode,
      effectiveText,
    );
    if (result) {
      try {
        await navigator.clipboard.writeText(result);
      } catch (e) {
        console.error("[Extension] Failed to copy result:", e);
      }
    }
    resetLauncherAndClose();
  };

  /**
   * 统一处理 Extension 动作分发
   * 查表 extensionActions.ts，根据策略类型执行对应操作
   */
  const handleExtensionAction = async (
    extensionId: string,
    commandCode: string,
    triggerMode?: string,
  ) => {
    const action = resolveExtensionAction(extensionId, commandCode);
    const effectiveText = clipboard.state.attachedText || inputValue;
    const ctx: ExtensionContext = {
      effectiveText,
      triggerMode: triggerMode as ExtensionContext["triggerMode"],
    };

    if (!action) {
      // 注册表中无配置：matched 模式走 execute，否则忽略
      if (triggerMode === "matched") {
        await runExtensionExecute(extensionId, commandCode);
      }
      return;
    }

    switch (action.type) {
      case "navigate":
        resetAndGoto(buildNavigateRoute(action, ctx));
        break;
      case "execute":
        await runExtensionExecute(extensionId, commandCode);
        break;
      case "color-pick":
        await startColorPickCommand();
        break;
      case "screenshot":
        if (await takeScreenshot()) {
          resetLauncherAndClose();
        }
        break;
    }
  };

  const handleOpenApp = async (app: LaunchableItem) => {
    // 检查是否需要确认
    if (app.requires_confirmation) {
      confirmDialogTitle = `确认${app.name}`;
      confirmDialogDescription = `确定要${app.name}吗?此操作无法撤销。`;
      pendingAction = () => executeApp(app);
      confirmDialogOpen = true;
      return;
    }

    // 不需要确认,直接执行
    await executeApp(app);
  };

  // 实际执行应用/命令的函数
  const executeApp = async (app: LaunchableItem) => {
    // 拦截内部页面跳转
    if (app.source === "Internal") {
      await appListManager.openApp(app, {}, () => {
        resetLauncherState();
        if (app.action === "open_settings") {
          goto("/settings");
        } else if (app.action === "open_plugins_manager") {
          goto("/plugins");
        } else if (app.action?.startsWith("open_settings_")) {
          const tab = app.action.replace("open_settings_", "");
          goto(`/settings?tab=${tab}`);
        }
      });
      return;
    }

    // 1. 优先处理 Extension 命令（查表分发，无需逐个 if-else）
    if (app.source === "Extension") {
      const extensionInfo = parseExtensionAction(app.action);
      if (extensionInfo) {
        if (app.action) {
          invoke("record_command_usage", {
            commandName: app.action,
          }).catch((err) =>
            console.error("Failed to record command usage:", err),
          );
        }
        accumulateCommandStat(app.source);
        await handleExtensionAction(
          extensionInfo.extensionId,
          extensionInfo.commandCode,
          app.trigger_mode,
        );
        return;
      }
    }

    // 2. 检查 Preview 项目（如计算器结果）
    // 注意：Preview 项目的 path 通常以 "extension:" 开头，但不一定是 source="Extension"
    if (app.path.startsWith("extension:")) {
      await handleExtensionClick(app);
      return;
    }

    // 准备参数
    const args: any = {};

    if (inputValue) {
      args.input = inputValue;
    }
    if (clipboard.state.attachedText) {
      args.text = clipboard.state.attachedText;
    }

    // 分类文件
    if (clipboard.state.attachedFiles.length > 0) {
      const images: any[] = [];
      const textFiles: any[] = [];
      const otherFiles: any[] = [];
      const folders: any[] = [];

      clipboard.state.attachedFiles.forEach((file) => {
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

      if (images.length > 0) args.images = images;
      if (textFiles.length > 0) args.textFiles = textFiles;
      if (otherFiles.length > 0) args.files = otherFiles;
      if (folders.length > 0) args.folders = folders;
    }

    await appListManager.openApp(app, args, () => {
      inputValue = "";
      clipboard.clearAttachments();
      matchedCommands = [];
      extensionPreviewItem = null;
      appListManager.resetToOriginList();
    });
  };

  // 处理 Extension 预览项点击（如计算器结果）
  // preview 项的 path 格式为 "extension:id:code"，统一查表分发
  const handleExtensionClick = async (app: LaunchableItem) => {
    if (app.action) {
      invoke("record_command_usage", {
        commandName: app.action,
      }).catch((err) => console.error("Failed to record command usage:", err));
    }
    accumulateCommandStat(app.source);
    const parts = app.path.split(":");
    if (parts.length >= 2) {
      const extensionId = parts[1];
      const commandCode = parts[2] || "";
      // grid 和普通预览项统一走注册表分发，triggerMode 均为 "preview"
      await handleExtensionAction(extensionId, commandCode, "preview");
      return;
    }
    // path 格式不合法，降级关闭
    resetLauncherAndClose();
  };

  const handleNavigationKeyDown = (e: KeyboardEvent) => {
    appListManager.handleKeyDown(e, displayList, handleOpenApp);
  };

  const handleToSettings = () => {
    goto("/settings");
  };

  const confirmPluginModeSwitch = async (): Promise<boolean> => {
    const { confirm } = await import("@tauri-apps/plugin-dialog");
    await invoke("acquire_window_close_lock");

    try {
      return await confirm(
        "切换显示方式会重新打开插件，当前页面状态可能丢失。确定继续吗？",
        {
          title: "切换显示方式",
          kind: "warning",
        },
      );
    } finally {
      await invoke("release_window_close_lock").catch(console.error);
    }
  };

  // ===== Lifecycle =====
  const unsubscribeTheme = theme.subscribe((value) => {
    currentTheme = value;
  });

  onMount(async () => {
    escapeHandler.set(handleEsc);
    plugin.setModeSwitchConfirmHandler(confirmPluginModeSwitch);

    const handleWindowEscape = (event: KeyboardEvent) => {
      if (event.key !== "Escape" || event.defaultPrevented) {
        return;
      }

      if (page.route.id !== "/") {
        return;
      }

      event.preventDefault();
      event.stopPropagation();
      handleEsc();
    };

    window.addEventListener("keydown", handleWindowEscape, true);
    removeWindowEscapeListener = () => {
      window.removeEventListener("keydown", handleWindowEscape, true);
    };

    // 加载配置
    await appListManager.loadConfig();

    // 获取应用列表
    await appListManager.fetchApps();

    // 初始启动时主动请求一次焦点
    if (!plugin.state.showPluginInline) {
      requestInputFocus();
    }

    // 监听窗口显示事件
    const unlistenWindowShow = await listen<boolean>(
      "window_visibility",
      async (event) => {
        if (event.payload) {
          await appListManager.fetchApps();

          if (!plugin.state.showPluginInline) {
            requestInputFocus();
          } else {
            invoke("focus_inline_plugin").catch(console.error);
          }

          await clipboard.autoPasteClipboard(
            appListManager.state.appConfig.auto_paste_time_limit,
          );
          updateMatchedCommands();
          await updateExtensionManagerPreview(); // 更新 Extension 预览（如计算器）
        }

        // 转发可见性事件给插件
        plugin.sendLifecycleEvent(event.payload ? "show" : "hide");
      },
    );

    // 监听清除剪贴板事件
    const unlistenClearClipboard = await listen("clear_app_clipboard", () => {
      clipboard.clearAttachments();
    });

    // 监听窗口焦点事件并转发给插件
    const currentWindow = getCurrentWindow();
    const unlistenFocus = await currentWindow.onFocusChanged(
      ({ payload: focused }) => {
        if (plugin.state.showPluginInline) {
          plugin.sendLifecycleEvent(focused ? "focus" : "blur");
        }
      },
    );

    // 监听后端发来的 ESC 事件 (当焦点在插件窗口或全局快捷键捕获时)
    const unlistenEsc = await listen("escape_pressed", () => {
      handleEsc();
    });

    // 设置 composables 的事件监听
    const unlistenPlugin = await plugin.setupListeners();
    const unlistenAppList = await appListManager.setupListeners();

    unlisten = () => {
      unlistenWindowShow();
      unlistenClearClipboard();
      unlistenFocus();
      unlistenEsc();
      unlistenPlugin();
      unlistenAppList();
    };
  });

  onDestroy(() => {
    unsubscribeTheme?.();

    if (get(escapeHandler) === handleEsc) {
      escapeHandler.set(null);
    }

    if (unlisten) {
      unlisten();
    }

    removeWindowEscapeListener?.();

    plugin.setModeSwitchConfirmHandler(null);

    if (extensionPreviewTimer) {
      clearTimeout(extensionPreviewTimer);
    }
  });
</script>

<div
  class="h-[100vh] w-full bg-transparent p-0"
  onmousemove={handleMouseMove}
  role="presentation"
>
  <main
    class="border-border/70 bg-background text-foreground flex h-full w-full flex-col overflow-hidden rounded-2xl border p-3.5 ring-1 ring-white/10 dark:ring-white/5"
    data-tauri-drag-region
  >
    <div
      class="flex h-full w-full flex-col"
      role="listbox"
      tabindex="0"
      onkeydown={handleNavigationKeyDown}
    >
      <!-- Header: Logo + Search Input + Plugin Menu -->
      <div class="border-border/40 flex items-center gap-2.5 border-b pb-2.5">
        <TooltipProvider delayDuration={400}>
          <Tooltip>
            <TooltipTrigger
              class="relative flex-shrink-0 cursor-pointer transition-transform duration-160 ease-[cubic-bezier(0.23,1,0.32,1)] outline-none hover:scale-105 active:scale-95"
              onclick={handleToSettings}
              aria-label="打开设置"
            >
              <img
                src="/logo.png"
                class="h-9 w-9 filter transition-[filter,transform] duration-160 hover:drop-shadow-[0_0_8px_rgba(99,102,241,0.55)] dark:hover:drop-shadow-[0_0_10px_rgba(165,180,252,0.6)]"
                alt="Onin logo"
              />
              {#if $hasNewVersion}
                <!-- 精致微章：呼吸灯紫色小红点，表示有新版本 -->
                <span class="absolute -top-0.5 -right-0.5 flex h-2.5 w-2.5">
                  <span
                    class="absolute inline-flex h-full w-full animate-ping rounded-full bg-violet-400 opacity-75"
                  ></span>
                  <span
                    class="relative inline-flex h-2.5 w-2.5 rounded-full bg-violet-500 shadow-xs"
                  ></span>
                </span>
              {/if}
            </TooltipTrigger>
            <TooltipContent side="right" sideOffset={8}>
              {#if $hasNewVersion && $latestVersion}
                发现新版本 v{$latestVersion} (当前 v{$appVersion})！点击查看更新
              {:else}
                打开设置 (Settings)
              {/if}
            </TooltipContent>
          </Tooltip>
        </TooltipProvider>

        <div class="min-w-0 flex-1">
          <SearchInput
            bind:this={searchInputRef}
            bind:value={inputValue}
            attachedText={clipboard.state.attachedText}
            attachedFiles={clipboard.state.attachedFiles}
            showAllFiles={clipboard.state.showAllFiles}
            onInput={handleInput}
            onPaste={handlePaste}
            onDrop={handleDrop}
            onDragOver={clipboard.handleDragOver}
            onRemoveFile={handleRemoveFile}
            onRemoveText={() => {
              clipboard.clearAttachments();
              updateMatchedCommands();
            }}
            onEditText={handleEditText}
            onToggleShowAllFiles={clipboard.toggleShowAllFiles}
            onBackspace={handleBackspace}
          />
        </div>

        <div class="flex-shrink-0">
          {#if plugin.state.showPluginInline}
            <PluginMenu
              bind:autoDetach={plugin.state.currentPluginAutoDetach}
              bind:terminateOnBg={plugin.state.currentPluginTerminateOnBg}
              bind:runAtStartup={plugin.state.currentPluginRunAtStartup}
              detachShortcut={$detachWindowShortcut}
              onDetach={plugin.detachPlugin}
              onClose={plugin.closePlugin}
              onToggleAutoDetach={plugin.toggleAutoDetach}
              onToggleTerminateOnBg={plugin.toggleTerminateOnBg}
              onToggleRunAtStartup={plugin.toggleRunAtStartup}
              onRefresh={plugin.reloadPlugin}
              onRestart={plugin.restartPlugin}
              onOpenDevTools={plugin.openDevTools}
              onUninstall={plugin.uninstallPlugin}
            />
          {/if}
        </div>
      </div>

      <!-- Content Area -->
      <div class="relative flex-1 overflow-hidden pt-2">
        <RefreshProgressBar isRefreshing={appListManager.state.isRefreshing} />

        {#if plugin.state.showPluginInline}
          <!-- Plugin Inline View -->
          <PluginInlineView
            bind:this={pluginInlineViewRef}
            url={plugin.state.currentPluginUrl}
            pluginId={plugin.state.currentPluginId}
            version={plugin.state.currentPluginVersion}
            onLoad={() => {
              // No-op for now, logic potentially moved to component or manager
            }}
          />
        {:else if displayList.length === 0}
          <!-- Empty State -->
          <div
            class="flex h-full flex-col items-center justify-center py-10 text-center select-none"
          >
            <div
              class="bg-muted/40 border-border/50 text-muted-foreground/50 mb-3 flex h-11 w-11 items-center justify-center rounded-2xl border shadow-2xs"
            >
              <svg
                class="h-5 w-5"
                fill="none"
                viewBox="0 0 24 24"
                stroke="currentColor"
              >
                <path
                  stroke-linecap="round"
                  stroke-linejoin="round"
                  stroke-width="1.5"
                  d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z"
                />
              </svg>
            </div>
            <p class="text-foreground/85 text-xs font-medium tracking-tight">
              未找到匹配的结果
            </p>
            <p
              class="text-muted-foreground/60 mt-1 max-w-xs text-[11px] leading-normal"
            >
              {#if inputValue}
                换个关键词搜索，或按 <kbd
                  class="border-border bg-muted text-foreground/80 shadow-kbd rounded border px-1 py-0.5 font-mono text-[9px]"
                  >Esc</kbd
                > 清空
              {:else}
                请输入应用名、拼音缩写或粘贴内容
              {/if}
            </p>
          </div>
        {:else}
          <!-- App List -->
          <ScrollArea
            class="h-full w-full"
            viewportClass="h-full w-full overflow-x-hidden pr-1.5"
          >
            <div class="app-list flex flex-col gap-1 overflow-hidden py-1">
              {#each displayList as app, index ((app.action || "") + app.path + app.name + index)}
                {#if app.path.startsWith("extension:")}
                  <!-- Extension 预览项（如计算器结果） -->
                  <ExtensionResultItem
                    title={app.name}
                    description={app.description || ""}
                    icon={app.icon}
                    triggerMode={app.trigger_mode}
                    isSelected={appListManager.state.selectedIndex === index}
                    onClick={() => handleOpenApp(app)}
                    onHover={(e) => handleItemHover(index, e)}
                  />
                {:else}
                  <AppListItem
                    {app}
                    isSelected={appListManager.state.selectedIndex === index}
                    onClick={() => handleOpenApp(app)}
                    onHover={(e) => handleItemHover(index, e)}
                  />
                {/if}
              {/each}
            </div>
          </ScrollArea>
        {/if}
      </div>

      <!-- Footer: Raycast-style Action Hints -->
      {#if !plugin.state.showPluginInline}
        {@const currentSelectedItem =
          displayList[appListManager.state.selectedIndex]}
        <footer
          class="border-border/40 mt-1 flex items-center justify-between border-t pt-2 pb-0.5 text-xs select-none"
        >
          <!-- 左侧：当前选中项信息 -->
          <div
            class="text-muted-foreground/60 flex min-w-0 items-center gap-1.5 text-[11px]"
          >
            {#if currentSelectedItem}
              <span
                class="text-foreground/75 max-w-[160px] truncate font-medium"
                >{currentSelectedItem.name}</span
              >
              <span class="text-muted-foreground/30">•</span>
              <span class="text-muted-foreground/50"
                >{currentSelectedItem.source_display ||
                  (currentSelectedItem.source === "Internal"
                    ? "内置"
                    : currentSelectedItem.source)}</span
              >
            {:else}
              <span class="text-muted-foreground/50">Onin Launcher</span>
            {/if}
          </div>

          <!-- 右侧：快捷键实体键帽提示 -->
          <div class="flex shrink-0 items-center gap-2.5">
            <div
              class="text-muted-foreground/60 flex items-center gap-1 text-[11px]"
            >
              <kbd
                class="border-border/80 bg-muted text-foreground/80 shadow-kbd inline-flex h-4.5 min-w-[18px] items-center justify-center rounded border px-1 font-mono text-[10px] font-semibold"
                >↵</kbd
              >
              <span class="text-[10.5px]">打开</span>
            </div>
            <div
              class="text-muted-foreground/60 flex items-center gap-1 text-[11px]"
            >
              <kbd
                class="border-border/80 bg-muted text-foreground/80 shadow-kbd inline-flex h-4.5 min-w-[18px] items-center justify-center rounded border px-1 font-mono text-[10px] font-semibold"
                >Esc</kbd
              >
              <span class="text-[10.5px]">关闭</span>
            </div>
          </div>
        </footer>
      {/if}
    </div>
  </main>
</div>

<!-- 确认对话框 -->
<ConfirmDialog
  bind:open={confirmDialogOpen}
  title={confirmDialogTitle}
  description={confirmDialogDescription}
  onConfirm={() => {
    if (pendingAction) {
      pendingAction();
      pendingAction = null;
    }
  }}
  onCancel={() => {
    pendingAction = null;
  }}
/>
