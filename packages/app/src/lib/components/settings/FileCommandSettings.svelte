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
  <header class="mb-4 flex items-center justify-between px-1">
    <div>
      <h2 class="text-foreground text-sm font-semibold tracking-tight">
        文件启动设置
      </h2>
      <p class="text-muted-foreground/75 text-xs">
        将常用文件、文件夹或应用添加为快捷启动指令
      </p>
    </div>
    <Button
      variant="outline"
      size="sm"
      class="h-8 cursor-pointer gap-1.5 rounded-xl text-xs font-medium transition-[transform,background-color] duration-120 active:scale-95"
      onclick={handleOpenFileOrFolder}
    >
      <Folder size={14} />
      添加文件/文件夹
    </Button>
  </header>

  {#if fileCommands.length > 0}
    <Accordion.Root class="mb-3" type="single">
      <Accordion.Item value="help" class="border-none">
        <Accordion.Header>
          <Accordion.Trigger
            class="text-muted-foreground hover:bg-muted/50 hover:text-foreground border-border/50 bg-muted/20 flex w-full cursor-pointer items-center justify-between rounded-xl border px-3 py-2 text-xs font-medium transition-[background-color,color] duration-120"
          >
            <span>如何添加文件/文件夹？</span>
            <CaretDown
              class="transition-transform duration-140 ease-[cubic-bezier(0.23,1,0.32,1)] [[data-state=open]_&]:rotate-180"
              size={14}
            />
          </Accordion.Trigger>
        </Accordion.Header>
        <Accordion.Content
          class="data-[state=closed]:animate-accordion-up data-[state=open]:animate-accordion-down overflow-hidden"
        >
          <div class="text-muted-foreground/80 px-3 pt-2 pb-1 text-xs">
            <ul class="ml-4 list-disc space-y-1">
              <li>
                <strong>拖放</strong>：从文件管理器拖放文件/文件夹到下方区域
              </li>
              <li>
                <strong>粘贴路径</strong>：复制文件/文件夹完整路径后按
                Cmd/Ctrl+V 粘贴
              </li>
              <li>
                <strong>点击右上角</strong>：点击「添加文件/文件夹」按钮浏览选择
              </li>
            </ul>
          </div>
        </Accordion.Content>
      </Accordion.Item>
    </Accordion.Root>
  {/if}

  <ScrollArea
    class="border-border/60 bg-card/40 relative flex-1 overflow-hidden rounded-2xl border-2 border-dashed p-2 transition-[border-color,background-color] duration-140 {isDraggingOver
      ? 'border-primary bg-primary/5'
      : ''}"
    orientation="both"
    viewportClass="h-full w-full"
  >
    <div
      bind:this={listContainerEl}
      class="list-container relative rounded-xl text-center transition-colors"
      role="group"
    >
      {#if isLoading}
        <p class="text-muted-foreground py-8 text-xs">正在加载...</p>
      {:else if fileCommands.length === 0}
        <div
          class="flex flex-col items-center justify-center gap-3 py-12 text-center"
        >
          <div
            class="border-border/50 bg-muted/60 text-muted-foreground flex h-12 w-12 items-center justify-center rounded-2xl border shadow-xs"
          >
            <Folder size={24} weight="duotone" />
          </div>
          <div class="space-y-1">
            <p class="text-foreground text-sm font-semibold tracking-tight">
              暂无已添加的文件快捷指令
            </p>
            <p class="text-muted-foreground/75 max-w-sm text-xs leading-normal">
              你可以直接将文件或文件夹拖拽至此区域，或通过粘贴路径与浏览添加。
            </p>
          </div>
          <Button
            variant="outline"
            size="sm"
            class="mt-2 h-8 cursor-pointer gap-1.5 rounded-xl text-xs font-medium transition-[transform,background-color] duration-120 active:scale-95"
            onclick={handleOpenFileOrFolder}
          >
            <Folder size={14} />
            选择文件或文件夹
          </Button>
        </div>
      {:else}
        <ul class="flex h-full w-full flex-col gap-1.5 text-left">
          {#each fileCommands as item, index (item.path)}
            <li
              class="group hover:bg-muted/60 border-border/40 hover:border-border bg-card/60 flex items-center justify-between gap-3 rounded-xl border p-2 shadow-2xs transition-[transform,background-color,border-color] duration-120 active:scale-[0.99]"
            >
              <div class="flex min-w-0 flex-1 items-center overflow-hidden">
                {#if item.icon && item.icon_type === "Base64"}
                  <img
                    src={`data:image/png;base64,${item.icon}`}
                    alt="{item.name} icon"
                    class="mr-2.5 h-8 w-8 flex-shrink-0 rounded-lg"
                  />
                {:else if item.icon}
                  <div
                    class="border-border/40 bg-muted/80 mr-2.5 flex h-8 w-8 flex-shrink-0 items-center justify-center rounded-lg border shadow-2xs"
                  >
                    <PhosphorIcon icon={item.icon} class="h-4.5 w-4.5" />
                  </div>
                {:else}
                  <div
                    class="border-border/40 bg-muted/80 text-muted-foreground mr-2.5 flex h-8 w-8 flex-shrink-0 items-center justify-center rounded-lg border shadow-2xs"
                  >
                    {#if item.item_type === "Folder"}
                      <Folder size={18} />
                    {:else if item.item_type === "File"}
                      <File size={18} />
                    {:else}
                      <AppWindow size={18} />
                    {/if}
                  </div>
                {/if}
                <div class="min-w-0 flex-1 overflow-hidden">
                  <p
                    class="text-foreground truncate text-xs font-semibold tracking-tight"
                  >
                    {item.name}
                  </p>
                  <p
                    class="text-muted-foreground/75 truncate font-mono text-[11px]"
                  >
                    {item.path}
                  </p>
                </div>
              </div>

              <div class="flex shrink-0 items-center pr-1">
                <Popover>
                  <PopoverTrigger>
                    <Button
                      variant="ghost"
                      size="icon"
                      class="text-muted-foreground hover:text-destructive h-7 w-7 cursor-pointer rounded-lg opacity-0 transition-[opacity,transform] duration-120 group-hover:opacity-100 active:scale-90"
                      aria-label="删除 {item.name}"
                    >
                      <Trash size={15} />
                    </Button>
                  </PopoverTrigger>
                  <PopoverContent
                    class="border-border/60 bg-popover text-popover-foreground w-64 rounded-xl border p-3 shadow-xl"
                  >
                    <h3 class="text-foreground mb-1 text-xs font-semibold">
                      确认删除？
                    </h3>
                    <p class="text-muted-foreground mb-3 text-[11px]">
                      将从文件启动列表中移除此项目。
                    </p>
                    <div class="flex justify-end gap-2">
                      <PopoverClose>
                        <Button
                          variant="outline"
                          size="sm"
                          class="h-7 cursor-pointer rounded-lg px-2.5 text-xs active:scale-95"
                        >
                          取消
                        </Button>
                      </PopoverClose>
                      <Button
                        variant="destructive"
                        size="sm"
                        class="h-7 cursor-pointer rounded-lg px-2.5 text-xs active:scale-95"
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
          class="bg-background/60 absolute inset-0 flex items-center justify-center rounded-xl backdrop-blur-xs"
        >
          <p class="text-foreground text-xs font-medium">正在处理...</p>
        </div>
      {/if}
    </div>
  </ScrollArea>
</main>
