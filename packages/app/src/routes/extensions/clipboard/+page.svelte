<script lang="ts">
  /**
   * Clipboard Extension Page
   */
  import { onMount, onDestroy } from "svelte";
  import { goto } from "$app/navigation";
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
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
  let listContainer: HTMLDivElement;

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
      // If it's text/file, we use set_clipboard_item (which currently only handles text)
      // TODO: Update backend command to handle files/images restoration if needed
      // detailed version: `set_clipboard_item` just calls `ctx.set_text`.
      // For images, we need a new command? Or just let user copy the "text" representation?
      // Since `monitor.rs` sets `text` to "Image 100x100" for images, pasting that is useless.
      // But for Files, `text` is the file list, which `ctx.set_text` puts as text, not true file object.
      // Limitation: Current `set_clipboard_item` only restores TEXT.
      // Improvement: Just using valid text for now.

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
</script>

<div class="flex h-full w-full flex-col">
  <ExtensionHeader
    bind:this={headerRef}
    placeholder="Search Clipboard History..."
    bind:value={searchQuery}
    onInput={handleSearch}
    onBack={handleBack}
    onKeyDown={handleKeyDown}
  />

  <div class="flex-1 overflow-y-auto p-2" bind:this={listContainer}>
    {#if filteredItems.length === 0}
      <div class="flex h-full items-center justify-center text-neutral-500">
        {#if searchQuery}
          No matches found
        {:else}
          Clipboard is empty
        {/if}
      </div>
    {:else}
      <div class="flex flex-col gap-1">
        {#each filteredItems as item, index (item.id)}
          <button
            id="item-{index}"
            class="group flex w-full flex-col items-start gap-1 rounded-lg p-3 text-left font-sans transition-colors
            {selectedIndex === index
              ? 'bg-neutral-200 dark:bg-neutral-700'
              : 'hover:bg-neutral-100 dark:hover:bg-neutral-800'}"
            onclick={() => handleItemSelect(item)}
            onmouseenter={() => (selectedIndex = index)}
          >
            {#if item.item_type === "Image" && item.thumbnail}
              <div class="flex gap-2">
                <img
                  src={item.thumbnail}
                  class="h-16 w-16 rounded border border-neutral-200 object-cover dark:border-neutral-700"
                  alt="Thumbnail"
                />
                <div class="flex flex-col">
                  <span
                    class="text-sm font-medium text-neutral-900 dark:text-neutral-100"
                    >Image</span
                  >
                  <span class="text-xs text-neutral-500">{item.text}</span>
                </div>
              </div>
            {:else}
              <div
                class="w-full truncate text-sm font-medium text-neutral-900 dark:text-neutral-100"
              >
                {item.text}
              </div>
            {/if}

            <div
              class="mt-1 flex w-full justify-between text-xs text-neutral-400"
            >
              <span
                class="rounded bg-neutral-200 px-1.5 py-0.5 text-neutral-600 dark:bg-neutral-700 dark:text-neutral-300"
              >
                {item.item_type}
              </span>
              <span>{formatTime(item.timestamp)}</span>
            </div>
          </button>
        {/each}
      </div>
    {/if}
  </div>
</div>
