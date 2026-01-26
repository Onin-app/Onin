<script lang="ts">
  /**
   * Clipboard Extension Page
   */
  import { onMount, onDestroy } from "svelte";
  import { goto } from "$app/navigation";
  import { invoke, convertFileSrc } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
  import { ScrollArea } from "bits-ui";
  import ExtensionHeader from "$lib/components/ExtensionHeader.svelte";

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
    try {
      // 1. Set to clipboard
      await invoke("set_clipboard_item", { text: item.text });
      console.log("[Clipboard] Copied item:", item.id);

      // 2. Hide window
      await invoke("close_main_window");

      // 3. Simulate paste
      setTimeout(async () => {
        try {
          await invoke("simulate_paste");
        } catch (e) {
          console.error("Failed to paste:", e);
        }
      }, 100);
    } catch (e) {
      console.error("Failed to select item:", e);
    }
  };

  onMount(async () => {
    fetchHistory();
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

  function isImageFile(path: string) {
    const lower = path.trim().toLowerCase();
    return (
      lower.endsWith(".png") ||
      lower.endsWith(".jpg") ||
      lower.endsWith(".jpeg") ||
      lower.endsWith(".gif") ||
      lower.endsWith(".bmp") ||
      lower.endsWith(".webp") ||
      lower.endsWith(".ico") ||
      lower.endsWith(".svg")
    );
  }

  function getDisplayName(item: ClipboardItem) {
    if (item.item_type === "File") {
      // Split by both / and \ to handle cross-platform paths, though mainly Windows here
      const parts = item.text.split(/[/\\]/);
      return parts[parts.length - 1] || item.text;
    }
    return item.text.replace(/\n/g, ' ');
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

  <div class="flex flex-1 overflow-hidden">
    <!-- Left List Pane -->
    <div 
      class="flex w-1/3 flex-col border-r border-neutral-200 dark:border-neutral-700"
    >
      <ScrollArea.Root class="h-full w-full" type="hover">
        <ScrollArea.Viewport class="h-full w-full p-2">
            {#if filteredItems.length === 0}
            <div class="flex h-full items-center justify-center text-sm text-neutral-500">
                {#if searchQuery}
                No matches
                {:else}
                Empty
                {/if}
            </div>
            {:else}
            <div class="flex flex-col gap-1">
                {#each filteredItems as item, index (item.id)}
                <button
                    id="item-{index}"
                    class="group flex w-full flex-col items-start gap-1 rounded-md p-2 text-left text-sm transition-colors border border-transparent
                    {selectedIndex === index
                    ? 'bg-blue-50 dark:bg-blue-900/20 border-blue-200 dark:border-blue-800'
                    : 'hover:bg-neutral-100 dark:hover:bg-neutral-800 border-transparent'}"
                    onclick={() => (selectedIndex = index)}
                    ondblclick={() => handleItemSelect(item)}
                >
                    <div class="flex w-full items-center justify-between gap-2">
                    <span
                        class="rounded px-1.5 py-0.5 text-[10px] uppercase tracking-wider font-semibold
                        {selectedIndex === index 
                        ? 'bg-blue-500 text-white' 
                        : 'bg-neutral-200 text-neutral-600 dark:bg-neutral-700 dark:text-neutral-300'}"
                    >
                        {item.item_type}
                    </span>
                    <span class="text-[10px] text-neutral-400">
                        {formatTime(item.timestamp)}
                    </span>
                    </div>
                    
                    <div class="flex w-full gap-2 items-center mt-1">
                    {#if item.item_type === "Image" && item.thumbnail}
                        <img 
                        src={item.thumbnail} 
                        alt="Thumbnail"
                        class="h-12 w-12 rounded object-cover border border-neutral-200 dark:border-neutral-700 flex-shrink-0 bg-neutral-100 dark:bg-neutral-800"
                        />
                        <span class="text-xs text-neutral-500 italic">Image Bitmap</span>
                    {:else}
                        <!-- Display filename for Files, plain text with fallback for others -->
                        <div class="w-full truncate font-medium text-neutral-900 dark:text-neutral-100 leading-tight" title={item.text}>
                        {getDisplayName(item)}
                        </div>
                    {/if}
                    </div>
                </button>
                {/each}
            </div>
            {/if}
        </ScrollArea.Viewport>
        <ScrollArea.Scrollbar
            orientation="vertical"
            class="bg-muted hover:bg-dark-10 data-[state=visible]:animate-in data-[state=hidden]:animate-out data-[state=hidden]:fade-out-0 data-[state=visible]:fade-in-0 flex w-1.5 touch-none select-none rounded-full border-l border-l-transparent p-px transition-all duration-200 hover:w-3"
        >
            <ScrollArea.Thumb class="bg-muted-foreground flex-1 rounded-full" />
        </ScrollArea.Scrollbar>
        <ScrollArea.Corner />
      </ScrollArea.Root>
    </div>

    <!-- Right Preview Pane -->
    <div class="flex w-2/3 flex-col overflow-hidden bg-white dark:bg-neutral-900">
      {#if filteredItems[selectedIndex]}
        {@const selectedItem = filteredItems[selectedIndex]}
        <div class="flex h-full flex-col">
           <!-- Preview Header -->
           <div class="flex items-center justify-between border-b border-neutral-200 px-4 py-3 dark:border-neutral-800 bg-neutral-50 dark:bg-neutral-900/50 flex-shrink-0">
              <div class="flex items-center gap-2">
                 <span class="font-medium text-neutral-900 dark:text-neutral-100 text-sm">Preview</span>
                 <span class="text-xs text-neutral-400">|</span>
                 <span class="text-xs text-neutral-500">{selectedItem.item_type}</span>
              </div>
              <button 
                class="rounded-md bg-blue-500 px-3 py-1 text-xs font-medium text-white hover:bg-blue-600 transition-colors shadow-sm"
                onclick={() => handleItemSelect(selectedItem)}
              >
                Paste
              </button>
           </div>
           
           <!-- Fixed File Path Info (if applicable) -->
           {#if selectedItem.item_type === "File"}
              <div class="flex-shrink-0 border-b border-neutral-100 bg-neutral-50/50 p-2 px-4 text-xs dark:border-neutral-800 dark:bg-neutral-900/30">
                 <div class="font-mono text-neutral-500 break-all select-text cursor-text">
                    {selectedItem.text}
                 </div>
              </div>
           {/if}

           <!-- Preview Content -->
           <div class="flex-1 overflow-hidden relative">
              <ScrollArea.Root class="h-full w-full" type="hover">
                <ScrollArea.Viewport class="h-full w-full p-6">
                    {#if selectedItem.item_type === "Image" && selectedItem.thumbnail}
                    <div class="flex min-h-full items-center justify-center bg-[url('/checker-board.svg')] bg-repeat">
                        <img
                        src={selectedItem.thumbnail}
                        class="max-w-full rounded shadow-lg border border-neutral-200 dark:border-neutral-700"
                        alt="Preview"
                        />
                    </div>
                    {:else if selectedItem.item_type === "File"}
                    {#if isImageFile(selectedItem.text)}
                        <div class="flex min-h-full items-center justify-center bg-[url('/checker-board.svg')] bg-repeat">
                            <img
                            src={convertFileSrc(selectedItem.text)}
                            class="max-w-full rounded shadow-lg border border-neutral-200 dark:border-neutral-700 max-h-[80vh]"
                            alt="Preview"
                            />
                        </div>
                    {:else}
                        <!-- For non-image files, we might want to show icon or details -->
                        <div class="flex h-full items-center justify-center text-neutral-400 flex-col gap-2">
                            <div class="i-lucide-file-text text-6xl opacity-20"></div>
                            <span>File Preview</span>
                        </div>
                    {/if}
                    {:else}
                    <!-- Text Content -->
                    <div class="font-mono text-sm text-neutral-800 dark:text-neutral-200 break-words whitespace-pre-wrap leading-relaxed select-text cursor-text">
                        {selectedItem.text}
                    </div>
                    {/if}
                </ScrollArea.Viewport>
                <ScrollArea.Scrollbar
                    orientation="vertical"
                    class="bg-muted hover:bg-dark-10 data-[state=visible]:animate-in data-[state=hidden]:animate-out data-[state=hidden]:fade-out-0 data-[state=visible]:fade-in-0 flex w-1.5 touch-none select-none rounded-full border-l border-l-transparent p-px transition-all duration-200 hover:w-3"
                >
                    <ScrollArea.Thumb class="bg-muted-foreground flex-1 rounded-full" />
                </ScrollArea.Scrollbar>
                <ScrollArea.Corner />
              </ScrollArea.Root>
           </div>
           
           <!-- Footer Info -->
           <div class="flex justify-between items-center border-t border-neutral-200 px-4 py-2 text-[10px] text-neutral-400 dark:border-neutral-800 bg-neutral-50 dark:bg-neutral-900/50">
              <span class="font-mono">{selectedItem.id}</span>
              <span>{selectedItem.text.length} chars</span>
           </div>
        </div>
      {:else}
        <div class="flex h-full items-center justify-center text-neutral-400 flex-col gap-2">
           <div class="i-lucide-clipboard text-4xl opacity-20"></div>
           <span>Select an item to view details</span>
        </div>
      {/if}
    </div>
  </div>
</div>
