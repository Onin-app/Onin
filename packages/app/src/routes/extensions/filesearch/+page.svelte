<script lang="ts">
  import { onDestroy, onMount } from "svelte";
  import { goto } from "$app/navigation";
  import { convertFileSrc, invoke } from "@tauri-apps/api/core";
  import { toast } from "svelte-sonner";
  import AppScrollArea from "$lib/components/AppScrollArea.svelte";
  import ConfirmDialog from "$lib/components/ConfirmDialog.svelte";
  import ExtensionHeader from "$lib/components/ExtensionHeader.svelte";
  import PhosphorIcon from "$lib/components/PhosphorIcon.svelte";
  import type { LaunchableItem } from "$lib/type";

  interface FileSearchStatus {
    is_searching: boolean;
    last_result_count: number;
    backend: string;
    everything_installed: boolean;
    everything_ipc_available: boolean;
    everything_install_required: boolean;
    available: boolean;
    last_error?: string | null;
  }

  let searchQuery = $state("");
  let results = $state<LaunchableItem[]>([]);
  let selectedIndex = $state(0);
  let status = $state<FileSearchStatus>({
    is_searching: false,
    last_result_count: 0,
    backend: "",
    everything_installed: false,
    everything_ipc_available: false,
    everything_install_required: false,
    available: true,
    last_error: null,
  });
  let isSearching = $state(false);
  let isInstallingEverything = $state(false);
  let installEverythingDialogOpen = $state(false);
  let dismissedEverythingInstallPrompt = $state(false);
  let headerRef = $state<ExtensionHeader>(null!);
  let searchVersion = 0;
  let searchInFlight = false;
  let queuedSearch: { query: string; version: number } | null = null;
  let searchTimer: ReturnType<typeof setTimeout> | null = null;
  let loadingTimer: ReturnType<typeof setTimeout> | null = null;
  let showSearchingIndicator = $state(false);
  let listPaneWidth = $state(40);
  let isResizingPane = $state(false);

  const selectedItem = $derived(results[selectedIndex] ?? null);
  const detailQueryExamples = [
    "logseq",
    "project ext:md",
    "image ext:png",
    "notes type:folder",
    '"project plan"',
  ];

  const handleBack = () => {
    goto("/");
  };

  const clearSearchTimers = () => {
    if (searchTimer) {
      clearTimeout(searchTimer);
      searchTimer = null;
    }
    if (loadingTimer) {
      clearTimeout(loadingTimer);
      loadingTimer = null;
    }
  };

  const handleSearch = (value: string) => {
    searchQuery = value;
    selectedIndex = 0;
    searchVersion += 1;
    const version = searchVersion;
    clearSearchTimers();

    const query = value.trim();
    if (query.length < 2) {
      queuedSearch = null;
      results = [];
      isSearching = false;
      showSearchingIndicator = false;
      return;
    }

    isSearching = true;
    showSearchingIndicator = false;
    const searchDelay = status.everything_ipc_available ? 25 : 180;
    searchTimer = setTimeout(() => {
      enqueueSearch(query, version);
    }, searchDelay);
    loadingTimer = setTimeout(() => {
      if (isSearching && results.length === 0) {
        showSearchingIndicator = true;
      }
    }, 140);
  };

  const applyQueryExample = (query: string) => {
    handleSearch(query);
    headerRef?.focus();
  };

  const enqueueSearch = (query: string, version: number) => {
    if (query.length < 2) {
      return;
    }

    queuedSearch = { query, version };
    if (!searchInFlight) {
      void runQueuedSearch();
    }
  };

  const runQueuedSearch = async () => {
    if (searchInFlight || !queuedSearch) {
      return;
    }

    const currentSearch = queuedSearch;
    queuedSearch = null;
    searchInFlight = true;

    try {
      const nextResults = await invoke<LaunchableItem[]>("search_files", {
        query: currentSearch.query,
        limit: 60,
      });

      if (
        currentSearch.version === searchVersion &&
        currentSearch.query === searchQuery.trim()
      ) {
        results = nextResults;
        isSearching = false;
        showSearchingIndicator = false;
      }
    } catch (error) {
      console.error("[FileSearch] Failed to search files:", error);
      if (
        currentSearch.version === searchVersion &&
        currentSearch.query === searchQuery.trim()
      ) {
        results = [];
        isSearching = false;
        showSearchingIndicator = false;
      }
    } finally {
      searchInFlight = false;
      if (queuedSearch) {
        void runQueuedSearch();
      }
    }
  };

  const refreshStatus = async () => {
    try {
      status = await invoke<FileSearchStatus>("get_file_search_status");
      if (
        status.everything_install_required &&
        !dismissedEverythingInstallPrompt &&
        !isInstallingEverything
      ) {
        installEverythingDialogOpen = true;
      }
    } catch (error) {
      console.error("[FileSearch] Failed to get status:", error);
    }
  };

  const installEverything = async () => {
    if (isInstallingEverything) return;

    isInstallingEverything = true;
    const toastId = toast.loading("正在安装 Everything...");
    try {
      await invoke("install_file_search_everything");
      await refreshStatus();
      toast.success("Everything 已安装，文件搜索将优先使用 Everything", {
        id: toastId,
      });
      const query = searchQuery.trim();
      if (query.length >= 2) {
        searchVersion += 1;
        enqueueSearch(query, searchVersion);
      }
    } catch (error) {
      console.error("[FileSearch] Failed to install Everything:", error);
      dismissedEverythingInstallPrompt = true;
      toast.error(String(error), { id: toastId });
    } finally {
      isInstallingEverything = false;
    }
  };

  const cancelEverythingInstall = () => {
    dismissedEverythingInstallPrompt = true;
  };

  const handleKeyDown = (e: KeyboardEvent) => {
    if (e.key === "Backspace" && searchQuery === "") {
      handleBack();
      return;
    }

    if (results.length === 0) return;

    if (e.key === "Enter") {
      e.preventDefault();
      openItem(results[selectedIndex]);
    } else if (e.key === "ArrowDown") {
      e.preventDefault();
      selectedIndex = Math.min(selectedIndex + 1, results.length - 1);
      scrollToSelected();
    } else if (e.key === "ArrowUp") {
      e.preventDefault();
      selectedIndex = Math.max(selectedIndex - 1, 0);
      scrollToSelected();
    }
  };

  const scrollToSelected = () => {
    setTimeout(() => {
      document
        .getElementById(`file-search-item-${selectedIndex}`)
        ?.scrollIntoView({ block: "nearest" });
    }, 0);
  };

  const openItem = async (item: LaunchableItem) => {
    try {
      await invoke("open_file_search_result", { path: item.path });
      invoke("close_main_window");
    } catch (error) {
      console.error("[FileSearch] Failed to open file:", error);
    }
  };

  const isImageFile = (path: string) => {
    const lower = path.toLowerCase();
    return [
      ".png",
      ".jpg",
      ".jpeg",
      ".gif",
      ".bmp",
      ".webp",
      ".ico",
      ".svg",
    ].some((ext) => lower.endsWith(ext));
  };

  const getKindLabel = (item: LaunchableItem) =>
    item.item_type === "Folder" ? "Folder" : "File";

  const clampListPaneWidth = (value: number) =>
    Math.min(64, Math.max(28, value));

  const startPaneResize = (event: MouseEvent) => {
    event.preventDefault();
    isResizingPane = true;
    document.body.classList.add("select-none", "cursor-col-resize");
  };

  const handlePaneResize = (event: MouseEvent) => {
    if (!isResizingPane) return;

    const shell = document.getElementById("file-search-split-shell");
    if (!shell) return;

    const rect = shell.getBoundingClientRect();
    const nextWidth = ((event.clientX - rect.left) / rect.width) * 100;
    listPaneWidth = clampListPaneWidth(nextWidth);
  };

  const stopPaneResize = () => {
    if (!isResizingPane) return;
    isResizingPane = false;
    document.body.classList.remove("select-none", "cursor-col-resize");
  };

  const getParentPath = (path: string) => {
    const normalized = path.replace(/[\\/]+$/, "");
    const separatorIndex = Math.max(
      normalized.lastIndexOf("\\"),
      normalized.lastIndexOf("/"),
    );

    if (separatorIndex <= 0) {
      return path;
    }

    return normalized.slice(0, separatorIndex);
  };

  const getFileExtension = (item: LaunchableItem) => {
    if (item.item_type === "Folder") return "";

    const dotIndex = item.name.lastIndexOf(".");
    if (dotIndex <= 0 || dotIndex === item.name.length - 1) {
      return "";
    }

    return item.name.slice(dotIndex + 1).toUpperCase();
  };

  const getPreviewKind = (item: LaunchableItem) => {
    if (item.item_type === "Folder") return "文件夹";

    const extension = getFileExtension(item);
    return extension ? `${extension} 文件` : "文件";
  };

  const getPreviewMeta = (item: LaunchableItem) => [
    { label: "Name", value: item.name },
    { label: "Where", value: getParentPath(item.path) },
    { label: "Type", value: getPreviewKind(item) },
    { label: "Path", value: item.path },
  ];

  onMount(async () => {
    await refreshStatus();
    headerRef?.focus();
    window.addEventListener("mousemove", handlePaneResize);
    window.addEventListener("mouseup", stopPaneResize);
  });

  onDestroy(() => {
    searchVersion += 1;
    queuedSearch = null;
    clearSearchTimers();
    window.removeEventListener("mousemove", handlePaneResize);
    window.removeEventListener("mouseup", stopPaneResize);
    document.body.classList.remove("select-none", "cursor-col-resize");
  });
</script>

<div class="flex h-full w-full flex-col overflow-hidden">
  <ExtensionHeader
    bind:this={headerRef}
    placeholder="搜索文件或文件夹..."
    bind:value={searchQuery}
    onInput={handleSearch}
    onBack={handleBack}
    onKeyDown={handleKeyDown}
  >
    {#snippet right()}
      <div
        class="flex items-center gap-1.5 rounded-md border border-neutral-300/70 bg-neutral-200/70 px-2 py-1 text-xs text-neutral-600 dark:border-neutral-600/70 dark:bg-neutral-700/70 dark:text-neutral-300"
        title={status.last_error || "系统文件搜索后端状态"}
      >
        <span
          class="h-1.5 w-1.5 rounded-full {status.is_searching
            ? 'animate-pulse bg-amber-500'
            : status.available
              ? 'bg-emerald-500'
              : 'bg-red-500'}"
        ></span>
        <span class="whitespace-nowrap">
          {status.backend || "系统搜索"} · {status.available
            ? "可用"
            : "不可用"}
        </span>
      </div>
    {/snippet}
  </ExtensionHeader>

  <div class="h-0.5 flex-shrink-0 overflow-hidden bg-transparent">
    {#if showSearchingIndicator}
      <div
        class="h-full w-1/3 animate-[file-search-loading_900ms_ease-in-out_infinite] rounded-full bg-blue-500/80"
      ></div>
    {/if}
  </div>

  {#if results.length === 0}
    <div class="flex flex-1 items-center justify-center overflow-hidden px-8">
      {#if showSearchingIndicator && searchQuery.trim().length >= 2}
        <div
          class="flex flex-col items-center justify-center gap-3 text-center text-sm text-neutral-500"
        >
          <div
            class="h-5 w-5 animate-spin rounded-full border-2 border-neutral-300 border-t-blue-500 dark:border-neutral-700 dark:border-t-blue-400"
          ></div>
          <span>正在搜索...</span>
        </div>
      {:else}
        <div
          class="flex max-w-lg flex-col items-center justify-center gap-4 text-center text-sm text-neutral-500"
        >
          <div
            class="flex h-16 w-16 items-center justify-center rounded-xl bg-neutral-100 text-neutral-400 dark:bg-neutral-800"
          >
            <PhosphorIcon icon="folder" class="h-8 w-8" />
          </div>
          <div class="space-y-1">
            <div class="font-medium text-neutral-700 dark:text-neutral-300">
              {#if searchQuery.trim().length < 2}
                输入至少 2 个字符开始搜索
              {:else}
                没有匹配的文件或文件夹
              {/if}
            </div>
            <div class="text-xs text-neutral-400">
              可组合关键词、扩展名和类型过滤
            </div>
          </div>
          <div class="flex flex-wrap justify-center gap-1.5">
            {#each detailQueryExamples as example (example)}
              <button
                class="rounded-md border border-neutral-200 bg-white px-2 py-1 font-mono text-[11px] text-neutral-600 transition-colors hover:border-neutral-300 hover:bg-neutral-100 dark:border-neutral-700 dark:bg-neutral-900 dark:text-neutral-300 dark:hover:border-neutral-600 dark:hover:bg-neutral-800"
                title="使用 {example} 搜索"
                onclick={() => applyQueryExample(example)}
              >
                {example}
              </button>
            {/each}
          </div>
        </div>
      {/if}
    </div>
  {:else}
    <div id="file-search-split-shell" class="flex flex-1 overflow-hidden">
      <div class="flex min-w-0 flex-col" style={`width: ${listPaneWidth}%`}>
        <AppScrollArea
          class="h-full w-full"
          viewportClass="h-full w-full p-1.5"
        >
          <div class="flex flex-col gap-1">
            {#each results as item, index (item.path)}
              <button
                id="file-search-item-{index}"
                class="group flex w-full items-center gap-2.5 rounded-md border border-transparent px-2.5 py-1.5 text-left text-sm transition-colors {selectedIndex ===
                index
                  ? 'bg-neutral-200 dark:bg-neutral-700/50'
                  : 'hover:bg-neutral-200/50 dark:hover:bg-neutral-800'}"
                onclick={() => (selectedIndex = index)}
                ondblclick={() => openItem(item)}
              >
                <div
                  class="flex h-8 w-8 flex-shrink-0 items-center justify-center rounded-md bg-neutral-200 text-neutral-600 dark:bg-neutral-700 dark:text-neutral-300"
                >
                  <PhosphorIcon icon={item.icon} size={18} />
                </div>

                <div class="min-w-0 flex-1">
                  <div
                    class="truncate font-medium text-neutral-900 dark:text-neutral-100"
                    title={item.name}
                  >
                    {item.name}
                  </div>
                  <div
                    class="mt-0.5 truncate text-xs text-neutral-500 dark:text-neutral-400"
                    title={item.description || item.path}
                  >
                    {item.description || item.path}
                  </div>
                </div>

                <span
                  class="flex-shrink-0 text-[10px] font-semibold tracking-wider text-neutral-400 uppercase"
                >
                  {getKindLabel(item)}
                </span>
              </button>
            {/each}
          </div>
        </AppScrollArea>
      </div>

      <button
        class="group relative z-10 flex w-2 flex-shrink-0 cursor-col-resize items-stretch justify-center border-x border-transparent bg-neutral-200/70 transition-colors hover:bg-blue-500/15 focus:outline-none dark:bg-neutral-800/80 dark:hover:bg-blue-400/15 {isResizingPane
          ? 'bg-blue-500/20 dark:bg-blue-400/20'
          : ''}"
        title="拖动调整列表和预览宽度"
        aria-label="拖动调整列表和预览宽度"
        onmousedown={startPaneResize}
      >
        <span
          class="my-auto h-10 w-0.5 rounded-full bg-neutral-300 transition-colors group-hover:bg-blue-500 dark:bg-neutral-600 dark:group-hover:bg-blue-400"
        ></span>
      </button>

      <div
        class="flex min-w-0 flex-1 flex-col overflow-hidden bg-white dark:bg-neutral-900"
      >
        <div
          class="flex flex-shrink-0 items-center justify-between border-b border-neutral-200 bg-neutral-50 px-4 py-2.5 dark:border-neutral-800 dark:bg-neutral-900/50"
        >
          <div class="min-w-0">
            <div
              class="truncate text-sm font-medium text-neutral-900 dark:text-neutral-100"
              title={selectedItem.name}
            >
              {selectedItem.name}
            </div>
            <div
              class="mt-1 truncate font-mono text-xs text-neutral-500 dark:text-neutral-400"
              title={selectedItem.path}
            >
              {selectedItem.path}
            </div>
          </div>
        </div>

        <div class="relative flex-1 overflow-hidden">
          <AppScrollArea class="h-full w-full" viewportClass="h-full w-full">
            {#if selectedItem.item_type === "File" && isImageFile(selectedItem.path)}
              <div
                class="flex min-h-full items-center justify-center bg-[url('/checker-board.svg')] bg-repeat p-8"
              >
                <img
                  src={convertFileSrc(selectedItem.path)}
                  class="max-h-[75vh] max-w-full rounded border border-neutral-200 shadow-lg dark:border-neutral-700"
                  alt="Preview"
                />
              </div>
            {:else}
              <div class="flex min-h-full flex-col">
                <div
                  class="flex min-h-[220px] flex-1 items-center justify-center px-8 py-8"
                >
                  {#if selectedItem.item_type === "Folder"}
                    <div class="folder-preview-icon" aria-hidden="true">
                      <div class="folder-preview-icon__back"></div>
                      <div class="folder-preview-icon__front"></div>
                    </div>
                  {:else}
                    <div
                      class="relative flex h-28 w-24 items-center justify-center rounded-lg border border-neutral-200 bg-gradient-to-b from-white to-neutral-100 text-neutral-400 shadow-[0_18px_45px_rgba(15,23,42,0.12)] dark:border-neutral-700 dark:from-neutral-800 dark:to-neutral-900 dark:text-neutral-500"
                    >
                      <div
                        class="absolute top-0 right-0 h-7 w-7 rounded-bl-lg border-b border-l border-neutral-200 bg-neutral-100 dark:border-neutral-700 dark:bg-neutral-800"
                      ></div>
                      <PhosphorIcon icon={selectedItem.icon} size={40} />
                    </div>
                  {/if}
                </div>

                <div
                  class="border-t border-neutral-200 px-5 py-4 dark:border-neutral-800"
                >
                  <div
                    class="mb-3 text-xs font-semibold text-neutral-500 dark:text-neutral-400"
                  >
                    Metadata
                  </div>
                  <div
                    class="divide-y divide-neutral-200 dark:divide-neutral-800"
                  >
                    {#each getPreviewMeta(selectedItem) as row (row.label)}
                      <div
                        class="grid grid-cols-[96px_minmax(0,1fr)] gap-4 py-2 text-xs"
                      >
                        <div
                          class="font-medium text-neutral-500 dark:text-neutral-400"
                        >
                          {row.label}
                        </div>
                        <div
                          class="truncate text-right font-medium text-neutral-900 dark:text-neutral-100"
                          title={row.value}
                        >
                          {row.value}
                        </div>
                      </div>
                    {/each}
                  </div>
                </div>

                <div
                  class="flex items-center justify-between gap-3 border-t border-neutral-200 px-5 py-3 dark:border-neutral-800"
                >
                  <div
                    class="min-w-0 text-xs text-neutral-500 dark:text-neutral-400"
                  >
                    双击结果或按 Enter 打开
                  </div>
                  <button
                    class="rounded-md bg-neutral-900 px-3 py-1.5 text-xs font-medium text-white transition-colors hover:bg-neutral-700 dark:bg-neutral-100 dark:text-neutral-900 dark:hover:bg-neutral-300"
                    onclick={() => openItem(selectedItem)}
                  >
                    打开
                  </button>
                </div>
              </div>
            {/if}
          </AppScrollArea>
        </div>
      </div>
    </div>
  {/if}
</div>

<ConfirmDialog
  bind:open={installEverythingDialogOpen}
  title="安装 Everything 加速文件搜索"
  description="当前未检测到 Everything。安装后 Onin 会优先使用 Everything IPC 获取实时文件搜索结果；取消后本次将继续使用 Windows Search。"
  onConfirm={installEverything}
  onCancel={cancelEverythingInstall}
/>

<style>
  @keyframes file-search-loading {
    0% {
      transform: translateX(-120%);
    }
    50% {
      transform: translateX(160%);
    }
    100% {
      transform: translateX(320%);
    }
  }

  .folder-preview-icon {
    position: relative;
    width: 112px;
    height: 102px;
    filter: drop-shadow(0 16px 18px rgb(15 23 42 / 0.13));
  }

  .folder-preview-icon__back,
  .folder-preview-icon__front {
    position: absolute;
    border-radius: 6px;
    background: linear-gradient(135deg, #ffe99d 0%, #ffd54f 48%, #ffc83d 100%);
  }

  .folder-preview-icon__back {
    top: 10px;
    right: 7px;
    width: 76px;
    height: 80px;
  }

  .folder-preview-icon__back::before {
    position: absolute;
    top: 0;
    left: -15px;
    width: 36px;
    height: 24px;
    border-radius: 6px 6px 0 0;
    background: #ffe690;
    content: "";
  }

  .folder-preview-icon__front {
    bottom: 3px;
    left: 15px;
    width: 70px;
    height: 78px;
    transform: skewY(28deg);
    transform-origin: bottom left;
    background: linear-gradient(145deg, #fff0b1 0%, #ffd96b 58%, #ffc743 100%);
    opacity: 0.96;
  }
</style>
