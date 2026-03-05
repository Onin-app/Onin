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
  import { ScrollArea } from "bits-ui";
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
  import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
  import { goto } from "$app/navigation";
  import { page } from "$app/state";

  // Stores
  import { Theme, type LaunchableItem } from "$lib/type";
  import { theme, getTheme } from "$lib/utils/theme";
  import { escapeHandler } from "$lib/stores/escapeHandler";
  import { focusInputTrigger } from "$lib/stores/focusInput";
  import { detachWindowShortcut } from "$lib/stores/shortcuts";

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

  // Component references
  let searchInputRef: SearchInput;
  let pluginInlineViewRef = $state<PluginInlineView | null>(null);

  // Confirm dialog state
  let confirmDialogOpen = $state(false);
  let confirmDialogTitle = $state("");
  let confirmDialogDescription = $state("");
  let pendingAction = $state<(() => void) | null>(null);

  // AutoAnimate action
  const animate: Action<HTMLElement> = (node) => {
    autoAnimate(node, {
      duration: 200,
      easing: "ease-in-out",
    });
  };

  // ===== Computed =====
  // 合并匹配命令和搜索结果
  // 优先级：Extension 预览 -> 精确/模糊匹配 -> 插件通配符匹配
  const displayList = $derived.by(() => {
    const result: LaunchableItem[] = [];

    // Extension 预览优先显示在最顶部（如计算器结果）
    if (extensionPreviewItem) {
      result.push(extensionPreviewItem);
    }

    if (matchedCommands.length === 0) {
      return [...result, ...appListManager.state.appList];
    }

    // 获取匹配命令的 action 集合，用于去重
    const matchedActions = new Set(
      matchedCommands.map((cmd) => cmd.action).filter(Boolean),
    );

    // 过滤掉 appList 中已经在匹配命令中的项
    const filteredAppList = appListManager.state.appList.filter(
      (app) => !app.action || !matchedActions.has(app.action),
    );

    // 去重：当 Extension 预览已存在时，从匹配命令中过滤掉 Extension 来源的命令
    // Extension 已通过预览条目展示（如 "翻译: hello"），无需在匹配命令中重复
    const deduplicatedMatchedCommands = extensionPreviewItem
      ? matchedCommands.filter((cmd) => cmd.source !== "Extension")
      : matchedCommands;

    // Extension 预览 -> 精确/模糊匹配(appList) -> 插件通配符匹配(matchedCommands)
    return [...result, ...filteredAppList, ...deduplicatedMatchedCommands];
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

  const handleEsc = () => {
    console.log("handleEsc triggered, route:", page.route.id);
    // Only handle ESC on main page
    if (page.route.id !== "/") {
      console.log("Not on main page, ignoring ESC");
      return;
    }

    if (plugin.state.showPluginInline) {
      console.log("Closing inline plugin");
      plugin.closePlugin();
      return;
    }
    console.log("Clearing input/closing main window");
    inputValue = "";
    clipboard.clearAttachments();
    matchedCommands = [];
    appListManager.resetToOriginList();
    invoke("close_main_window");
  };

  const handleInput = async (value: string) => {
    inputValue = value;
    appListManager.handleInput(value);
    updateMatchedCommands();
    await updateExtensionPreview();
  };

  // 更新 Extension 预览（计算器等）
  const updateExtensionPreview = async () => {
    // 优先使用粘贴的文本，其次使用输入框的值
    const effectiveText = clipboard.state.attachedText || inputValue;
    await extensionManager.getPreview(effectiveText);
    extensionPreviewItem = extensionManager.getPreviewAsItem();
  };

  const updateMatchedCommands = () => {
    matchedCommands = clipboard.getMatchedCommands(
      appListManager.state.originAppList,
      inputValue,
    );
  };

  const handlePaste = async (e: ClipboardEvent) => {
    await clipboard.handlePaste(e);
    updateMatchedCommands();
    await updateExtensionPreview();
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

  // 解析 Extension Action
  const parseExtensionAction = (
    action: string | undefined,
  ): { extensionId: string; commandCode: string } | null => {
    if (!action || !action.startsWith("extension:")) return null;

    const parts = action.split(":");
    // 格式: extension:id:code
    if (parts.length >= 3) {
      return {
        extensionId: parts[1],
        commandCode: parts[2],
      };
    }
    return null;
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
    // 1. 优先处理 Extension 命令
    if (app.source === "Extension") {
      const extensionInfo = parseExtensionAction(app.action);
      if (extensionInfo) {
        const { extensionId } = extensionInfo;
        // Emoji Extension 特殊处理：导航到独立页面
        if (extensionId === "emoji") {
          inputValue = "";
          clipboard.clearAttachments();
          extensionPreviewItem = null;
          extensionManager.clearPreview();
          matchedCommands = [];
          goto("/extensions/emoji");
          return;
        }
        // Clipboard Extension
        if (extensionId === "clipboard") {
          inputValue = "";
          clipboard.clearAttachments();
          extensionPreviewItem = null;
          extensionManager.clearPreview();
          matchedCommands = [];
          goto("/extensions/clipboard");
          return;
        }
        // Translator Extension
        if (extensionId === "translator") {
          const effectiveText = clipboard.state.attachedText || inputValue;
          await extensionManager.execute(extensionId, effectiveText);

          inputValue = "";
          clipboard.clearAttachments();
          extensionPreviewItem = null;
          extensionManager.clearPreview();
          matchedCommands = [];
          appListManager.resetToOriginList();
          // We likely want to close the main window as the translator opens in a new window
          // The backend execute handler for translator opens a new window.
          invoke("close_main_window");
          return;
        }
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

  // 处理 Extension 项目点击（如计算器结果或 emoji）
  const handleExtensionClick = async (app: LaunchableItem) => {
    // 获取 Extension ID
    const parts = app.path.split(":");
    if (parts.length >= 2) {
      const extensionId = parts[1];

      // 检查是否是 grid 类型的 extension（如 emoji）
      const preview = extensionManager.state.currentPreview;
      if (preview?.view_type === "grid" && extensionId === "emoji") {
        // 导航到 emoji 页面
        inputValue = "";
        clipboard.clearAttachments();
        extensionPreviewItem = null;
        extensionManager.clearPreview();
        matchedCommands = [];
        goto("/extensions/emoji");
        return;
      }

      // 使用有效文本（粘贴文本或输入框值）
      const effectiveText = clipboard.state.attachedText || inputValue;
      const result = await extensionManager.execute(extensionId, effectiveText);

      if (result) {
        // 复制结果到剪贴板
        try {
          await navigator.clipboard.writeText(result);
          console.log("[Extension] Copied to clipboard:", result);
        } catch (e) {
          console.error("[Extension] Failed to copy:", e);
        }
      }
    }

    // 清理状态并关闭窗口
    inputValue = "";
    clipboard.clearAttachments();
    extensionPreviewItem = null;
    extensionManager.clearPreview();
    matchedCommands = [];
    appListManager.resetToOriginList();
    invoke("close_main_window");
  };

  const handleKeyDown = (e: KeyboardEvent) => {
    appListManager.handleKeyDown(e, displayList, handleOpenApp);
  };

  const handleToSettings = () => {
    goto("/settings");
  };

  // ===== Lifecycle =====
  const unsubscribeTheme = theme.subscribe((value) => {
    currentTheme = value;
  });

  onMount(async () => {
    console.log("Main page component has mounted");
    escapeHandler.set(handleEsc);

    // 设置插件消息处理 (已废弃，使用 Native Bridge)
    // window.addEventListener("message", plugin.handlePluginMessage);

    // 加载配置
    await appListManager.loadConfig();

    // 获取应用列表
    await appListManager.fetchApps();

    // 监听窗口显示事件
    const unlistenWindowShow = await listen<boolean>(
      "window_visibility",
      async (event) => {
        if (event.payload) {
          await clipboard.autoPasteClipboard();
          updateMatchedCommands();
          await updateExtensionPreview(); // 更新 Extension 预览（如计算器）
          queueMicrotask(() => searchInputRef?.focus());
        }

        // 转发可见性事件给插件
        plugin.sendLifecycleEvent(event.payload ? "show" : "hide");
      },
    );

    // 监听清除剪贴板事件
    const unlistenClearClipboard = await listen("clear_app_clipboard", () => {
      console.log("Clearing app clipboard content");
      clipboard.clearAttachments();
    });

    // 监听窗口焦点事件并转发给插件
    const currentWindow = getCurrentWebviewWindow();
    const unlistenFocus = await currentWindow.onFocusChanged(
      ({ payload: focused }) => {
        if (plugin.state.showPluginInline) {
          plugin.sendLifecycleEvent(focused ? "focus" : "blur");
        }
      },
    );

    // 监听后端发来的 ESC 事件 (当焦点在插件窗口或全局快捷键捕获时)
    const unlistenEsc = await listen("escape_pressed", () => {
      console.log("Received escape_pressed event from backend");
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

    // window.removeEventListener("message", plugin.handlePluginMessage);
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
    <!-- Header: Logo + Search Input + Plugin Menu -->
    <div class="flex items-center gap-2 pb-2">
      <button class="flex-shrink-0 cursor-pointer" onclick={handleToSettings}>
        <img src="/logo.png" class="h-10 w-10" alt="Onin logo" />
      </button>

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

      <div class="flex-shrink-0">
        {#if plugin.state.showPluginInline}
          <PluginMenu
            bind:autoDetach={plugin.state.currentPluginAutoDetach}
            detachShortcut={$detachWindowShortcut}
            onDetach={plugin.detachPlugin}
            onClose={plugin.closePlugin}
            onToggleAutoDetach={plugin.toggleAutoDetach}
          />
        {/if}
      </div>
    </div>

    <!-- Content Area -->
    <div class="relative flex-1 overflow-hidden">
      <RefreshProgressBar isRefreshing={appListManager.state.isRefreshing} />

      {#if plugin.state.showPluginInline}
        <!-- Plugin Inline View -->
        <PluginInlineView
          bind:this={pluginInlineViewRef}
          url={plugin.state.currentPluginUrl}
          pluginId={plugin.state.currentPluginId}
          onLoad={() => {
            // No-op for now, logic potentially moved to component or manager
          }}
        />
      {:else}
        <!-- App List -->
        <ScrollArea.Root
          class="h-full w-full rounded-[10px] border px-2 py-2"
          type="hover"
        >
          <ScrollArea.Viewport class="h-full w-full overflow-x-hidden">
            <div class="app-list overflow-hidden">
              <div use:animate>
                {#each displayList as app, index ((app.action || '') + app.path + app.name + index)}
                  {#if app.path.startsWith("extension:")}
                    <!-- Extension 预览项（如计算器结果） -->
                    <ExtensionResultItem
                      title={app.name}
                      description={app.description || ""}
                      icon={app.icon}
                      isSelected={appListManager.state.selectedIndex === index}
                      onClick={() => handleOpenApp(app)}
                    />
                  {:else}
                    <AppListItem
                      {app}
                      isSelected={appListManager.state.selectedIndex === index}
                      onClick={() => handleOpenApp(app)}
                    />
                  {/if}
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

          <ScrollArea.Corner />
        </ScrollArea.Root>
      {/if}
    </div>
  </div>
</main>

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
