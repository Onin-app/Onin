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
  import { goto } from "$app/navigation";

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

  // Components
  import SearchInput from "$lib/components/SearchInput.svelte";
  import AppListItem from "$lib/components/AppListItem.svelte";
  import PluginMenu from "$lib/components/PluginMenu.svelte";
  import RefreshProgressBar from "$lib/components/RefreshProgressBar.svelte";
  import PluginIframe from "$lib/components/PluginIframe.svelte";

  import "../index.css";

  // ===== Composables =====
  const plugin = usePluginManager();
  const clipboard = useClipboardManager();
  const appListManager = useAppList();

  // ===== Local State =====
  let inputValue = $state<string>("");
  let matchedCommands = $state<LaunchableItem[]>([]);
  let currentTheme = $state<Theme>(Theme.DARK);
  let unlisten = $state<null | (() => void)>(null);

  // Component references
  let searchInputRef: SearchInput;

  // AutoAnimate action
  const animate: Action<HTMLElement> = (node) => {
    autoAnimate(node, {
      duration: 200,
      easing: "ease-in-out",
    });
  };

  // ===== Computed =====
  // 合并匹配命令和搜索结果，匹配命令优先显示在顶部
  const displayList = $derived.by(() => {
    if (matchedCommands.length === 0) {
      return appListManager.state.appList;
    }

    // 获取匹配命令的 action 集合，用于去重
    const matchedActions = new Set(
      matchedCommands.map((cmd) => cmd.action).filter(Boolean),
    );

    // 过滤掉 appList 中已经在匹配命令中的项
    const filteredAppList = appListManager.state.appList.filter(
      (app) => !app.action || !matchedActions.has(app.action),
    );

    // 匹配命令排在前面，然后是过滤后的搜索结果
    return [...matchedCommands, ...filteredAppList];
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
    if (plugin.state.showPluginInline) {
      plugin.closePlugin();
      return;
    }
    inputValue = "";
    clipboard.clearAttachments();
    matchedCommands = [];
    appListManager.resetToOriginList();
    invoke("close_main_window");
  };

  const handleInput = (value: string) => {
    inputValue = value;
    appListManager.handleInput(value);
    updateMatchedCommands();
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

  const handleOpenApp = async (app: LaunchableItem) => {
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
      appListManager.resetToOriginList();
    });
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

    // 设置插件消息处理
    window.addEventListener("message", plugin.handlePluginMessage);

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

    // 设置 composables 的事件监听
    const unlistenPlugin = await plugin.setupListeners();
    const unlistenAppList = await appListManager.setupListeners();

    unlisten = () => {
      unlistenWindowShow();
      unlistenClearClipboard();
      unlistenPlugin();
      unlistenAppList();
    };
  });

  onDestroy(() => {
    unsubscribeTheme?.();

    if (get(escapeHandler) === handleEsc) {
      escapeHandler.set(() => {});
    }

    if (unlisten) {
      unlisten();
    }

    window.removeEventListener("message", plugin.handlePluginMessage);
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
        <img
          src="/ff_logo_{getTheme(currentTheme) === Theme.DARK
            ? Theme.LIGHT
            : Theme.DARK}.svg"
          class="h-10 w-10"
          alt="Tauri logo"
        />
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
        <!-- Plugin Iframe -->
        <PluginIframe url={plugin.state.currentPluginUrl} />
      {:else}
        <!-- App List -->
        <ScrollArea.Root
          class="h-full w-full rounded-[10px] border px-2 py-2"
          type="hover"
        >
          <ScrollArea.Viewport class="h-full w-full">
            <div class="app-list">
              <div use:animate>
                {#each displayList as app, index (app.path + app.name)}
                  <AppListItem
                    {app}
                    isSelected={appListManager.state.selectedIndex === index}
                    onClick={() => handleOpenApp(app)}
                  />
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
