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
    is_indexing: boolean;
    indexed_count: number;
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
    is_indexing: false,
    indexed_count: 0,
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
  let requestId = 0;
  let searchTimer: ReturnType<typeof setTimeout> | null = null;
  let loadingTimer: ReturnType<typeof setTimeout> | null = null;
  let showSearchingIndicator = $state(false);

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

  const handleSearch = (value: string) => {
    searchQuery = value;
    selectedIndex = 0;

    if (searchTimer) {
      clearTimeout(searchTimer);
    }
    if (loadingTimer) {
      clearTimeout(loadingTimer);
    }

    const query = value.trim();
    if (query.length < 2) {
      requestId++;
      results = [];
      isSearching = false;
      showSearchingIndicator = false;
      return;
    }

    isSearching = true;
    showSearchingIndicator = false;
    const searchDelay = status.everything_ipc_available ? 25 : 180;
    searchTimer = setTimeout(() => {
      searchFiles(value);
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

  const searchFiles = async (value: string) => {
    const currentRequestId = ++requestId;
    const query = value.trim();

    if (query.length < 2) {
      results = [];
      return;
    }

    try {
      const nextResults = await invoke<LaunchableItem[]>(
        "search_indexed_files",
        {
          query,
          limit: 60,
        },
      );

      if (currentRequestId === requestId) {
        results = nextResults;
        isSearching = false;
        showSearchingIndicator = false;
      }
    } catch (error) {
      console.error("[FileSearch] Failed to search files:", error);
      if (currentRequestId === requestId) {
        results = [];
        isSearching = false;
        showSearchingIndicator = false;
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
      if (searchQuery.trim().length >= 2) {
        searchFiles(searchQuery);
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
      await invoke("open_indexed_file", { path: item.path });
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

  onMount(async () => {
    await refreshStatus();
    headerRef?.focus();
  });

  onDestroy(() => {
    if (searchTimer) {
      clearTimeout(searchTimer);
    }
    if (loadingTimer) {
      clearTimeout(loadingTimer);
    }
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
          class="h-1.5 w-1.5 rounded-full {status.is_indexing
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
    <div class="flex flex-1 overflow-hidden">
      <div
        class="flex w-2/5 flex-col border-r border-neutral-200 dark:border-neutral-700"
      >
        <AppScrollArea class="h-full w-full" viewportClass="h-full w-full p-2">
          <div class="flex flex-col gap-1">
            {#each results as item, index (item.path)}
              <button
                id="file-search-item-{index}"
                class="group flex w-full items-center gap-3 rounded-md border border-transparent px-3 py-2 text-left text-sm transition-colors {selectedIndex ===
                index
                  ? 'bg-neutral-200 dark:bg-neutral-700/50'
                  : 'hover:bg-neutral-200/50 dark:hover:bg-neutral-800'}"
                onclick={() => (selectedIndex = index)}
                ondblclick={() => openItem(item)}
              >
                <div
                  class="flex h-9 w-9 flex-shrink-0 items-center justify-center rounded-md bg-neutral-200 text-neutral-600 dark:bg-neutral-700 dark:text-neutral-300"
                >
                  <PhosphorIcon icon={item.icon} class="h-5 w-5" />
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

      <div
        class="flex w-3/5 flex-col overflow-hidden bg-white dark:bg-neutral-900"
      >
        <div
          class="flex flex-shrink-0 items-center justify-between border-b border-neutral-200 bg-neutral-50 px-4 py-3 dark:border-neutral-800 dark:bg-neutral-900/50"
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
          <AppScrollArea
            class="h-full w-full"
            viewportClass="h-full w-full p-6"
          >
            {#if selectedItem.item_type === "File" && isImageFile(selectedItem.path)}
              <div
                class="flex min-h-full items-center justify-center bg-[url('/checker-board.svg')] bg-repeat"
              >
                <img
                  src={convertFileSrc(selectedItem.path)}
                  class="max-h-[75vh] max-w-full rounded border border-neutral-200 shadow-lg dark:border-neutral-700"
                  alt="Preview"
                />
              </div>
            {:else}
              <div
                class="flex h-full min-h-[320px] flex-col items-center justify-center gap-3 text-neutral-400"
              >
                <div
                  class="flex h-20 w-20 items-center justify-center rounded-xl bg-neutral-100 dark:bg-neutral-800"
                >
                  <PhosphorIcon icon={selectedItem.icon} class="h-10 w-10" />
                </div>
                <div class="text-sm">
                  {selectedItem.item_type === "Folder"
                    ? "文件夹预览"
                    : "文件预览"}
                </div>
                <button
                  class="rounded-md bg-neutral-900 px-3 py-1.5 text-xs font-medium text-white transition-colors hover:bg-neutral-700 dark:bg-neutral-100 dark:text-neutral-900 dark:hover:bg-neutral-300"
                  onclick={() => openItem(selectedItem)}
                >
                  打开
                </button>
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
</style>
