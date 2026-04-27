<script lang="ts">
  import { onDestroy, onMount } from "svelte";
  import { goto } from "$app/navigation";
  import { convertFileSrc, invoke } from "@tauri-apps/api/core";
  import AppScrollArea from "$lib/components/AppScrollArea.svelte";
  import ExtensionHeader from "$lib/components/ExtensionHeader.svelte";
  import PhosphorIcon from "$lib/components/PhosphorIcon.svelte";
  import type { LaunchableItem } from "$lib/type";

  interface FileSearchStatus {
    is_indexing: boolean;
    indexed_count: number;
  }

  let searchQuery = $state("");
  let results = $state<LaunchableItem[]>([]);
  let selectedIndex = $state(0);
  let status = $state<FileSearchStatus>({
    is_indexing: false,
    indexed_count: 0,
  });
  let isSearching = $state(false);
  let headerRef = $state<ExtensionHeader>(null!);
  let requestId = 0;
  let statusTimer: ReturnType<typeof setInterval> | null = null;
  let searchTimer: ReturnType<typeof setTimeout> | null = null;

  const selectedItem = $derived(results[selectedIndex] ?? null);

  const handleBack = () => {
    goto("/");
  };

  const handleSearch = (value: string) => {
    searchQuery = value;
    selectedIndex = 0;

    if (searchTimer) {
      clearTimeout(searchTimer);
    }

    const query = value.trim();
    if (query.length < 2) {
      requestId++;
      results = [];
      isSearching = false;
      return;
    }

    isSearching = true;
    searchTimer = setTimeout(() => {
      searchFiles(value);
    }, 250);
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
      }
    } catch (error) {
      console.error("[FileSearch] Failed to search files:", error);
      if (currentRequestId === requestId) {
        results = [];
        isSearching = false;
      }
    }
  };

  const refreshStatus = async () => {
    try {
      status = await invoke<FileSearchStatus>("get_file_search_status");
    } catch (error) {
      console.error("[FileSearch] Failed to get status:", error);
    }
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

    statusTimer = setInterval(refreshStatus, 1000);
  });

  onDestroy(() => {
    if (statusTimer) {
      clearInterval(statusTimer);
    }
    if (searchTimer) {
      clearTimeout(searchTimer);
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
        title={status.is_indexing ? "正在建立本地文件索引" : "本地文件索引状态"}
      >
        <span
          class="h-1.5 w-1.5 rounded-full {status.is_indexing
            ? 'animate-pulse bg-amber-500'
            : 'bg-emerald-500'}"
        ></span>
        <span class="whitespace-nowrap">
          {status.is_indexing ? "索引中" : "已索引"} ·
          {status.indexed_count.toLocaleString()} 项
        </span>
      </div>
    {/snippet}
  </ExtensionHeader>

  <div class="h-0.5 flex-shrink-0 overflow-hidden bg-transparent">
    {#if isSearching}
      <div
        class="h-full w-1/3 animate-[file-search-loading_900ms_ease-in-out_infinite] rounded-full bg-blue-500/80"
      ></div>
    {/if}
  </div>

  <div class="flex flex-1 overflow-hidden">
    <div
      class="flex w-2/5 flex-col border-r border-neutral-200 dark:border-neutral-700"
    >
      <AppScrollArea class="h-full w-full" viewportClass="h-full w-full p-2">
        {#if isSearching && searchQuery.trim().length >= 2 && results.length === 0}
          <div
            class="flex h-full flex-col items-center justify-center gap-3 px-6 text-center text-sm text-neutral-500"
          >
            <div
              class="h-5 w-5 animate-spin rounded-full border-2 border-neutral-300 border-t-blue-500 dark:border-neutral-700 dark:border-t-blue-400"
            ></div>
            <span>正在搜索...</span>
          </div>
        {:else if results.length === 0}
          <div
            class="flex h-full items-center justify-center px-6 text-center text-sm text-neutral-500"
          >
            {#if searchQuery.trim().length < 2}
              输入至少 2 个字符开始搜索
            {:else}
              没有匹配的文件或文件夹
            {/if}
          </div>
        {:else}
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
        {/if}
      </AppScrollArea>
    </div>

    <div
      class="flex w-3/5 flex-col overflow-hidden bg-white dark:bg-neutral-900"
    >
      {#if selectedItem}
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
      {:else}
        <div
          class="flex h-full flex-col items-center justify-center gap-2 text-neutral-400"
        >
          <div
            class="flex h-16 w-16 items-center justify-center rounded-xl bg-neutral-100 dark:bg-neutral-800"
          >
            <PhosphorIcon icon="folder" class="h-8 w-8" />
          </div>
          <span>选择文件或文件夹查看详情</span>
        </div>
      {/if}
    </div>
  </div>
</div>

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
