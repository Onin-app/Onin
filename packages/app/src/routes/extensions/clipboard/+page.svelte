<script lang="ts">
  /**
   * Clipboard Extension Page
   */
  import { onMount, onDestroy } from "svelte";
  import { goto } from "$app/navigation";
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
  import { ScrollArea } from "$lib/components/ui/scroll-area";
  import ExtensionHeader from "$lib/components/ExtensionHeader.svelte";
  import FilePreview from "$lib/components/FilePreview.svelte";

  type ClipboardItem = {
    id: string;
    text: string;
    timestamp: number;
    item_type: string; // "Text" | "Image" | "File"
    thumbnail?: string;
  };

  let items = $state<ClipboardItem[]>([]);
  let searchQuery = $state("");
  let selectedIndex = $state(0);
  let headerRef: ExtensionHeader;
  let unlisten: () => void;
  // let listContainer: HTMLDivElement; // ScrollArea handles refs differently if needed, or we just look up by ID

  // Derived state for filtered items
  let filteredItems = $derived(
    items.filter((item) =>
      item.text.toLowerCase().includes(searchQuery.toLowerCase()),
    ),
  );

  async function fetchHistory() {
    try {
      items = await invoke<ClipboardItem[]>("get_clipboard_history");
      selectedIndex = 0; // Reset selection on update
    } catch (e) {
      console.error("Failed to fetch history:", e);
    }
  }

  const handleBack = () => {
    goto("/");
  };

  const handleSearch = (value: string) => {
    searchQuery = value;
    selectedIndex = 0;
  };

  const handleKeyDown = (e: KeyboardEvent) => {
    if (filteredItems.length === 0) return;

    if (e.key === "Enter") {
      e.preventDefault();
      handleItemSelect(filteredItems[selectedIndex]);
    } else if (e.key === "ArrowDown") {
      e.preventDefault();
      selectedIndex = Math.min(selectedIndex + 1, filteredItems.length - 1);
      scrollToSelected();
    } else if (e.key === "ArrowUp") {
      e.preventDefault();
      selectedIndex = Math.max(selectedIndex - 1, 0);
      scrollToSelected();
    }
  };

  const scrollToSelected = () => {
    // Simple logic to scroll element into view if needed
    // We need IDs on the elements to find them
    setTimeout(() => {
      const el = document.getElementById(`item-${selectedIndex}`);
      if (el) {
        el.scrollIntoView({ block: "nearest" });
      }
    }, 0);
  };

  const handleItemSelect = async (item: ClipboardItem) => {
    const startTime = performance.now();

    try {
      // 1. Paste (Background: set clipboard + simulate paste)
      // The backend command spawns a thread and returns immediately, so this await is fast.
      // Optimization: Send ID only to avoid large Base64 transfer.
      await invoke("paste_clipboard_item", { itemId: item.id });

      const t1 = performance.now();

      // 2. Hide window immediately
      invoke("close_main_window");

      const t2 = performance.now();
    } catch (e) {
      console.error("Failed to select item:", e);
    }
  };

  // 提前发起异步历史数据获取，与路由转场并行，以在窗口显现时能立刻呈现列表响应键盘
  fetchHistory();

  onMount(async () => {
    headerRef?.focus();

    // Listen for updates from backend
    unlisten = await listen("clipboard-update", () => {
      fetchHistory();
    });
  });

  onDestroy(() => {
    if (unlisten) unlisten();
  });

  function formatTime(ts: number) {
    return new Date(ts).toLocaleTimeString([], {
      hour: "2-digit",
      minute: "2-digit",
    });
  }

  function getDisplayName(item: ClipboardItem) {
    if (item.item_type === "File") {
      // Split by both / and \ to handle cross-platform paths, though mainly Windows here
      const parts = item.text.split(/[/\\]/);
      return parts[parts.length - 1] || item.text;
    }
    return item.text.replace(/\n/g, " ");
  }
</script>

<div class="flex h-full w-full flex-col overflow-hidden">
  <ExtensionHeader
    bind:this={headerRef}
    placeholder="Search Clipboard History..."
    bind:value={searchQuery}
    onInput={handleSearch}
    onBack={handleBack}
    onKeyDown={handleKeyDown}
  />

  <div class="flex flex-1 gap-2 overflow-hidden pt-2">
    <!-- Left List Pane -->
    <div class="border-border/40 flex w-1/3 flex-col border-r pr-2">
      <ScrollArea class="h-full w-full" viewportClass="h-full w-full pr-1">
        {#if filteredItems.length === 0}
          <div
            class="text-muted-foreground flex h-full items-center justify-center text-xs"
          >
            {#if searchQuery}
              无匹配记录
            {:else}
              暂无历史记录
            {/if}
          </div>
        {:else}
          <div class="flex flex-col gap-1">
            {#each filteredItems as item, index (item.id)}
              <button
                id="item-{index}"
                class="group flex w-full cursor-pointer flex-row items-center gap-3 rounded-xl border px-3 py-2 text-left font-sans text-xs transition-[transform,background-color,border-color] duration-120 active:scale-[0.985]
                    {selectedIndex === index
                  ? 'border-border/60 bg-accent text-accent-foreground shadow-2xs'
                  : 'text-foreground/80 hover:bg-muted/60 border-transparent'}"
                onclick={() => (selectedIndex = index)}
                ondblclick={() => handleItemSelect(item)}
              >
                <!-- Left: Thumbnail or Spacer -->
                {#if item.item_type === "Image" && item.thumbnail}
                  <img
                    src={item.thumbnail}
                    alt="Thumbnail"
                    class="border-border/40 bg-muted h-9 w-9 flex-shrink-0 rounded-lg border object-cover shadow-2xs"
                  />
                {/if}

                <!-- Middle: Content -->
                <div class="flex min-w-0 flex-1 flex-col justify-center">
                  {#if item.item_type === "File"}
                    <div
                      class="text-foreground w-full truncate leading-tight font-medium"
                      title={item.text}
                    >
                      {getDisplayName(item)}
                    </div>
                  {:else if item.item_type === "Image"}
                    <span class="text-muted-foreground/70 text-xs italic"
                      >图片数据</span
                    >
                  {:else}
                    <div
                      class="text-foreground/85 line-clamp-2 w-full leading-relaxed break-all"
                    >
                      {item.text}
                    </div>
                  {/if}
                </div>

                <!-- Right: Metadata -->
                <div
                  class="flex flex-shrink-0 flex-col items-end gap-0.5 self-start pt-0.5"
                >
                  <span
                    class="text-muted-foreground/50 text-[9px] font-semibold tracking-wider uppercase"
                  >
                    {item.item_type}
                  </span>
                  <span
                    class="text-muted-foreground/50 font-mono text-[10px] tabular-nums"
                  >
                    {formatTime(item.timestamp)}
                  </span>
                </div>
              </button>
            {/each}
          </div>
        {/if}
      </ScrollArea>
    </div>

    <!-- Right Preview Pane -->
    <div
      class="bg-card border-border/50 flex w-2/3 flex-col overflow-hidden rounded-2xl border shadow-2xs"
    >
      {#if filteredItems[selectedIndex]}
        {@const selectedItem = filteredItems[selectedIndex]}
        <div class="flex h-full flex-col">
          <!-- Preview Header -->
          <div
            class="border-border/40 bg-muted/30 flex flex-shrink-0 items-center justify-between border-b px-4 py-2.5"
          >
            <div class="flex items-center gap-2">
              <span class="text-foreground text-xs font-medium">预览</span>
              <span class="text-muted-foreground/40 text-xs">•</span>
              <span class="text-muted-foreground text-xs"
                >{selectedItem.item_type}</span
              >
            </div>
          </div>

          <!-- Fixed File Path Info (if applicable) -->
          {#if selectedItem.item_type === "File"}
            <div
              class="border-border/40 bg-muted/20 flex-shrink-0 border-b p-2 px-4 text-xs"
            >
              <div
                class="text-muted-foreground/80 cursor-text font-mono text-xs break-all select-text"
              >
                {selectedItem.text}
              </div>
            </div>
          {/if}

          <!-- Preview Content -->
          <div class="relative flex-1 overflow-hidden">
            <ScrollArea class="h-full w-full" viewportClass="h-full w-full p-4">
              {#if selectedItem.item_type === "Image" && selectedItem.thumbnail}
                <FilePreview
                  imageSrc={selectedItem.thumbnail}
                  fileName="剪贴板图片"
                />
              {:else if selectedItem.item_type === "File"}
                <FilePreview
                  path={selectedItem.text}
                  fileName={getDisplayName(selectedItem)}
                  onOpen={() => handleItemSelect(selectedItem)}
                />
              {:else}
                <!-- Text Content -->
                <div
                  class="text-foreground/90 cursor-text font-mono text-xs leading-relaxed break-words whitespace-pre-wrap select-text"
                >
                  {selectedItem.text}
                </div>
              {/if}
            </ScrollArea>
          </div>

          <!-- Footer Info -->
          <div
            class="border-border/40 bg-muted/20 text-muted-foreground/60 flex items-center justify-between border-t px-4 py-2 text-[10px]"
          >
            <span class="font-mono">{selectedItem.id}</span>
            <span class="font-mono">{selectedItem.text.length} 字符</span>
          </div>
        </div>
      {:else}
        <div
          class="text-muted-foreground flex h-full flex-col items-center justify-center gap-2 text-xs"
        >
          <span>选择一项查看详情</span>
        </div>
      {/if}
    </div>
  </div>
</div>
