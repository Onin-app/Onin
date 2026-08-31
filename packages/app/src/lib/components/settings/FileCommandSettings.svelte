<script lang="ts">
  import { onDestroy, onMount } from "svelte";
  import {
    Popover,
    PopoverTrigger,
    PopoverContent,
    PopoverClose,
  } from "$lib/components/ui/popover";
  import { Button } from "$lib/components/ui/button";
  import { ScrollArea } from "$lib/components/ui/scroll-area";
  import { Accordion } from "bits-ui";
  import { invoke } from "@tauri-apps/api/core";
  import { open } from "@tauri-apps/plugin-dialog";
  import { platform } from "@tauri-apps/plugin-os";
  import type { LaunchableItem } from "$lib/type";
  import PhosphorIcon from "$lib/components/PhosphorIcon.svelte";
  import { getCurrentWebview } from "@tauri-apps/api/webview";
  import { TauriEvent } from "@tauri-apps/api/event";
  import { Trash, Folder, File, AppWindow, CaretDown } from "phosphor-svelte";

  let fileCommands = $state<LaunchableItem[]>([]);
  let listContainerEl: HTMLDivElement | undefined = $state();
  let isLoading = $state(true);
  let isProcessing = $state(false);
  let isDraggingOver = $state(false);
  let unlistenDragDrop = $state<() => void>();
  let currentPlatform = $state("");

  const normalizePathKey = (path: string) => {
    const trimmedPath = path.trim();
    return currentPlatform === "windows"
      ? trimmedPath.replaceAll("/", "\\").toLowerCase()
      : trimmedPath;
  };

  const getFileCommandKey = (item: LaunchableItem) =>
    normalizePathKey(item.path);

  const uniqueFileCommands = (items: LaunchableItem[]) => {
    const seen = new Set<string>();
    return items.filter((item) => {
      const key = getFileCommandKey(item);
      if (seen.has(key)) {
        return false;
      }

      seen.add(key);
      return true;
    });
  };

  async function fetchFileCommands() {
    isLoading = true;
    try {
      const items = await invoke<LaunchableItem[]>("get_all_launchable_items");
      fileCommands = uniqueFileCommands(
        items.filter((item) => item.source === "FileCommand"),
      );
    } catch (e) {
      console.error("Failed to get file commands:", e);
    } finally {
      isLoading = false;
    }
  }

  function loadPlatform() {
    try {
      currentPlatform = platform();
    } catch (error) {
      console.error("Failed to detect platform:", error);
      currentPlatform = "";
    }
  }

  const addItems = async (paths: string[]) => {
    isProcessing = true;
    try {
      await invoke("add_file_command_items", { paths });
      await fetchFileCommands();
    } catch (error) {
      console.error("Failed to add file command items:", error);
    } finally {
      isProcessing = false;
    }
  };

  const deleteItem = async (path: string) => {
    try {
      await invoke("delete_file_command_item", { path });
      await fetchFileCommands();
    } catch (error) {
      console.error("Failed to delete file command item:", error);
    }
  };

  const handleOpenFileOrFolder = async () => {
    try {
      const selected = await open({
        multiple: true,
        directory: false,
      });

      if (!selected) return;

      const paths = Array.isArray(selected) ? selected : [selected];
      if (paths.length > 0) {
        await addItems(paths);
      }
    } catch (error) {
      console.error("Failed to open file dialog:", error);
    }
  };

  const handlePaste = async (event: ClipboardEvent) => {
    const target = event.target as HTMLElement | null;
    if (
      target &&
      (target.tagName === "INPUT" ||
        target.tagName === "TEXTAREA" ||
        target.isContentEditable)
    ) {
      return;
    }

    const text = event.clipboardData?.getData("text")?.trim();
    if (!text) return;

    const paths = text
      .split(/\r?\n/)
      .map((p) => p.trim())
      .filter(Boolean);

    if (paths.length > 0) {
      await addItems(paths);
    }
  };

  const listenDragDrop = async () => {
    unlistenDragDrop = await getCurrentWebview().onDragDropEvent(async (e) => {
      const event = e.event;
      const payload = e.payload as {
        paths: string[];
        position: { type: "Physical"; x: number; y: number };
        type: "enter" | "over" | "drop" | "leave";
      };
      if (!listContainerEl) return;
      const rect = listContainerEl.getBoundingClientRect();
      const isOverTarget =
        payload.position.x >= rect.left &&
        payload.position.x <= rect.right &&
        payload.position.y >= rect.top &&
        payload.position.y <= rect.bottom;

      if (event === TauriEvent.DRAG_OVER && isOverTarget) {
        isDraggingOver = isOverTarget;
      } else if (event === TauriEvent.DRAG_DROP && isOverTarget) {
        if (payload?.paths?.length > 0) {
          await addItems(payload.paths);
          isDraggingOver = false;
        }
      } else {
        isDraggingOver = false;
      }
    });
  };

  $effect(() => {
    document.addEventListener("paste", handlePaste);
    return () => {
      document.removeEventListener("paste", handlePaste);
    };
  });

  onMount(() => {
    loadPlatform();
    fetchFileCommands();
    listenDragDrop();
  });

  onDestroy(() => {
    unlistenDragDrop?.();
  });
</script>

<main class="flex h-full w-full flex-col">
  <h2 class="text-foreground mb-2 text-xl font-bold">文件启动设置</h2>
  {#if fileCommands.length > 0}
    <Accordion.Root class="mb-2" type="single">
      <Accordion.Item value="help">
        <Accordion.Header>
          <Accordion.Trigger
            class="text-muted-foreground hover:bg-accent flex w-full items-center justify-between rounded-lg px-2 py-1 text-sm"
          >
            <span>如何添加文件/文件夹？</span>
            <CaretDown
              class="transition-transform duration-200 [[data-state=open]_&]:rotate-180"
              size={16}
            />
          </Accordion.Trigger>
        </Accordion.Header>
        <Accordion.Content
          class="data-[state=closed]:animate-accordion-up data-[state=open]:animate-accordion-down overflow-hidden"
        >
          <div class="text-muted-foreground px-2 pt-1 pb-2 text-sm">
            <ul class="ml-4 list-disc space-y-1">
              <li>
                <strong>拖放</strong>：从 Finder 拖放文件/文件夹到下方区域
              </li>
              <li>
                <strong>粘贴路径</strong>：复制文件/文件夹的完整路径后按
                Cmd+V（如 /Users/xxx/file.txt）
              </li>
              <li>
                <strong>点击按钮</strong>：点击 <Button
                  variant="outline"
                  size="sm"
                  class="h-5 px-1.5 text-[10px]"
                  onclick={handleOpenFileOrFolder}
                >
                  按钮
                </Button> 选择文件/文件夹
              </li>
            </ul>
          </div>
        </Accordion.Content>
      </Accordion.Item>
    </Accordion.Root>
  {/if}

  <ScrollArea
    class="relative flex-1 overflow-hidden rounded-xl border-2 border-dashed px-2 py-2 {isDraggingOver
      ? 'border-primary'
      : 'border-border'}"
    orientation="both"
    viewportClass="h-full w-full"
  >
    <div
      bind:this={listContainerEl}
      class="list-container relative rounded-lg text-center transition-colors"
      role="group"
    >
      {#if isLoading}
        <p class="text-muted-foreground">正在加载...</p>
      {:else if fileCommands.length === 0}
        <Accordion.Root class="mb-2" type="single">
          <Accordion.Item value="help">
            <Accordion.Header>
              <Accordion.Trigger
                class="text-muted-foreground hover:bg-accent flex w-full items-center justify-between rounded-lg px-2 py-1 text-sm"
              >
                <span>如何添加文件/文件夹？</span>
                <CaretDown
                  class="transition-transform duration-200 [[data-state=open]_&]:rotate-180"
                  size={16}
                />
              </Accordion.Trigger>
            </Accordion.Header>
            <Accordion.Content
              class="data-[state=closed]:animate-accordion-up data-[state=open]:animate-accordion-down overflow-hidden"
            >
              <div
                class="text-muted-foreground px-2 pt-1 pb-2 text-left text-sm"
              >
                <ul class="ml-4 list-disc space-y-1">
                  <li>
                    <strong>拖放</strong>：从 Finder 拖放文件/文件夹到此区域
                  </li>
                  <li>
                    <strong>粘贴路径</strong>：复制文件/文件夹的完整路径后按
                    Cmd+V（如 /Users/xxx/file.txt）
                  </li>
                  <li>
                    <strong>点击按钮</strong>：点击 <Button
                      variant="outline"
                      size="sm"
                      class="h-5 px-1.5 text-[10px]"
                      onclick={handleOpenFileOrFolder}
                    >
                      按钮
                    </Button> 选择文件/文件夹
                  </li>
                </ul>
              </div>
            </Accordion.Content>
          </Accordion.Item>
        </Accordion.Root>
      {:else}
        <ul class="flex h-full w-full flex-col gap-1 text-left">
          {#each fileCommands as item, index (item.path)}
            <li
              class="group hover:bg-accent grid grid-cols-[1fr_auto] items-center gap-2 rounded-lg p-1 transition-colors"
            >
              <div class="flex min-w-0 items-center overflow-hidden p-1">
                {#if item.icon && item.icon_type === "Base64"}
                  <img
                    src={`data:image/png;base64,${item.icon}`}
                    alt="{item.name} icon"
                    class="mr-3 h-7 w-7 flex-shrink-0"
                  />
                {:else if item.icon}
                  <div
                    class="bg-muted mr-3 flex h-7 w-7 flex-shrink-0 items-center justify-center rounded-md"
                  >
                    <PhosphorIcon icon={item.icon} class="h-5 w-5" />
                  </div>
                {:else}
                  <div
                    class="bg-muted text-muted-foreground mr-3 flex h-7 w-7 flex-shrink-0 items-center justify-center rounded"
                  >
                    {#if item.item_type === "Folder"}
                      <Folder size={20} />
                    {:else if item.item_type === "File"}
                      <File size={20} />
                    {:else}
                      <AppWindow size={20} />
                    {/if}
                  </div>
                {/if}
                <div class="min-w-0 flex-1 overflow-hidden">
                  <p class="text-foreground truncate text-sm font-semibold">
                    {item.name}
                  </p>
                  <p class="text-muted-foreground truncate text-xs">
                    {item.path}
                  </p>
                </div>
              </div>

              <div class="flex items-center justify-center pr-2">
                <Popover>
                  <PopoverTrigger>
                    <Button
                      variant="ghost"
                      size="icon"
                      class="text-muted-foreground hover:text-destructive h-7 w-7 opacity-0 transition-opacity group-hover:opacity-100"
                      aria-label="删除 {item.name}"
                    >
                      <Trash size={18} />
                    </Button>
                  </PopoverTrigger>
                  <PopoverContent class="w-64">
                    <h3 class="mb-2 text-sm font-semibold">确认删除？</h3>
                    <div class="flex justify-end gap-2">
                      <PopoverClose>
                        <Button variant="outline" size="sm" class="h-7 text-xs">
                          取消
                        </Button>
                      </PopoverClose>
                      <Button
                        variant="destructive"
                        size="sm"
                        class="h-7 text-xs"
                        onclick={() => deleteItem(item.path)}
                      >
                        确认
                      </Button>
                    </div>
                  </PopoverContent>
                </Popover>
              </div>
            </li>
          {/each}
        </ul>
      {/if}
      {#if isProcessing}
        <div
          class="absolute inset-0 flex items-center justify-center rounded-lg bg-black/30 backdrop-blur-xs"
        >
          <p class="text-sm font-medium text-white">正在处理...</p>
        </div>
      {/if}
    </div>
  </ScrollArea>
</main>
