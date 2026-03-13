<script lang="ts">
  import { onDestroy, onMount } from "svelte";
  import { Popover, Button, ScrollArea, Accordion } from "bits-ui";
  import { invoke } from "@tauri-apps/api/core";
  import { open } from "@tauri-apps/plugin-dialog";
  import type { LaunchableItem } from "$lib/type";
  import PhosphorIcon from "$lib/components/PhosphorIcon.svelte";
  // import { Webview } from "@tauri-apps/api/webview";
  import { getCurrentWebview } from "@tauri-apps/api/webview";
  import { TauriEvent } from "@tauri-apps/api/event";
  import { Trash, Folder, File, AppWindow, CaretDown } from "phosphor-svelte";

  let fileCommands = $state<LaunchableItem[]>([]);
  let listContainerEl: HTMLDivElement | undefined = $state();
  let isLoading = $state(true);
  let isProcessing = $state(false);
  let isDraggingOver = $state(false);
  let unlistenDragDrop = $state<() => void>();

  async function fetchFileCommands() {
    isLoading = true;
    try {
      const items = await invoke<LaunchableItem[]>("get_all_launchable_items");
      fileCommands = items.filter((item) => item.source === "FileCommand");
    } catch (e) {
      console.error("Failed to get file commands:", e);
    } finally {
      isLoading = false;
    }
  }

  async function addItems(paths: string[]) {
    if (paths.length === 0 || isProcessing) return;

    isProcessing = true;
    try {
      const newItems: LaunchableItem[] = await invoke("add_file_commands", {
        paths,
      });
      fileCommands = newItems;
    } catch (e) {
      console.error("Failed to add file commands:", e);
    } finally {
      isProcessing = false;
    }
  }

  async function handlePaste(event: ClipboardEvent) {
    event.preventDefault();

    const clipboardData = event.clipboardData;
    if (!clipboardData) return;

    // 只支持文本路径，不支持从 Finder 复制的文件对象
    // 因为浏览器安全限制，无法读取文件对象的完整路径
    const text = clipboardData.getData("text/plain");

    if (!text) {
      return;
    }

    const paths = text
      .split("\n")
      .map((p) => p.trim())
      .filter(Boolean);

    if (paths.length > 0) {
      await addItems(paths);
    }
  }

  async function deleteItem(path: string) {
    try {
      // 乐观更新UI，立即移除项以获得更好的用户体验
      const originalItems = fileCommands;
      fileCommands = fileCommands.filter((item) => item.path !== path);

      // 调用后端并同步最终状态
      const newItems: LaunchableItem[] = await invoke("remove_file_command", {
        path,
      });
      fileCommands = newItems;
    } catch (e) {
      console.error("Failed to remove file command:", e);
      // 如果出错，则从后端重新获取列表以回滚状态
      fetchFileCommands();
    }
  }

  const handleOpenFileOrFolder = async () => {
    await invoke("acquire_window_close_lock");
    isProcessing = true;
    try {
      // 先尝试使用自定义的原生对话框（macOS 支持同时选择文件和文件夹）
      let selected = await invoke<string[]>("open_file_or_folder_dialog");

      // 如果返回空数组（可能是非 macOS 平台或用户取消），回退到 Tauri 对话框
      if (selected.length === 0) {
        const tauriSelected = await open({
          multiple: true,
          directory: false, // 非 macOS 平台只能选择文件
        });
        if (tauriSelected && tauriSelected.length > 0) {
          selected = tauriSelected;
        }
      }

      if (selected.length === 0) {
        return;
      }

      const newItems: LaunchableItem[] = await invoke("add_file_commands", {
        paths: selected,
      });
      fileCommands = newItems;
    } catch (e) {
      console.error("Failed to add file commands:", e);
    } finally {
      isProcessing = false;
      await invoke("release_window_close_lock");
    }
  };

  // 监听拖放文件到指定区域
  const listenDragDrop = async () => {
    // 修复：正确赋值给 state 变量，以便组件销毁时可以注销监听器
    unlistenDragDrop = await getCurrentWebview().onDragDropEvent(async (e) => {
      const event = e.event;
      const payload = e.payload as {
        paths: string[];
        position: { type: "Physical"; x: number; y: number };
        type: "enter" | "over" | "drop" | "leave";
      };
      if (!listContainerEl) return;
      // 获取目标元素的位置和尺寸
      const rect = listContainerEl.getBoundingClientRect();
      // 判断鼠标坐标是否在目标元素内部
      const isOverTarget =
        payload.position.x >= rect.left &&
        payload.position.x <= rect.right &&
        payload.position.y >= rect.top &&
        payload.position.y <= rect.bottom;

      if (event === TauriEvent.DRAG_OVER && isOverTarget) {
        // 当拖动文件悬浮在目标区域上时，设置状态以提供视觉反馈
        isDraggingOver = isOverTarget;
      } else if (event === TauriEvent.DRAG_DROP && isOverTarget) {
        if (payload?.paths?.length > 0) {
          // 如果在目标区域内释放，则添加文件
          await addItems(payload.paths);
          isDraggingOver = false;
        }
      } else {
        // 当拖放操作被取消或离开窗口时，重置视觉反馈
        isDraggingOver = false;
      }
    });
  };

  // 将粘贴事件绑定到 document，避免 main 元素需要 tabindex
  $effect(() => {
    document.addEventListener("paste", handlePaste);
    return () => {
      document.removeEventListener("paste", handlePaste);
    };
  });

  onMount(() => {
    fetchFileCommands();
    listenDragDrop();
  });

  onDestroy(() => {
    unlistenDragDrop?.();
  });
</script>

<main
  class="flex h-full w-full flex-col"
>
  <h2 class="mb-2 text-xl font-bold">文件启动设置</h2>
  {#if fileCommands.length > 0}
    <Accordion.Root class="mb-2" type="single">
      <Accordion.Item value="help">
        <Accordion.Header>
          <Accordion.Trigger
            class="flex w-full items-center justify-between rounded-lg px-2 py-1 text-sm text-neutral-500 hover:bg-neutral-100 dark:hover:bg-neutral-800"
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
          <div class="px-2 pt-1 pb-2 text-sm text-neutral-500">
            <ul class="ml-4 list-disc space-y-1">
              <li>
                <strong>拖放</strong>：从 Finder 拖放文件/文件夹到下方区域
              </li>
              <li>
                <strong>粘贴路径</strong>：复制文件/文件夹的完整路径后按
                Cmd+V（如 /Users/xxx/file.txt）
              </li>
              <li>
                <strong>点击按钮</strong>：点击 <Button.Root
                  class="rounded-input bg-dark text-background shadow-mini hover:bg-dark/95 mx-1
	inline-flex h-5 items-center justify-center px-[6px]
	text-[10px] font-semibold active:scale-[0.98] active:transition-all"
                  onclick={handleOpenFileOrFolder}
                >
                  按钮
                </Button.Root> 选择文件/文件夹
              </li>
            </ul>
          </div>
        </Accordion.Content>
      </Accordion.Item>
    </Accordion.Root>
  {/if}

  <ScrollArea.Root
    class="bg-background-alt shadow-card relative flex-1 overflow-hidden rounded-[10px] border border-2 border-dashed px-2 py-2 {isDraggingOver
      ? 'border-blue-500'
      : 'border-neutral-300 dark:border-neutral-600'}"
    type="hover"
  >
    <ScrollArea.Viewport class="h-full w-full ">
      <div
        bind:this={listContainerEl}
        class="list-container relative rounded-lg text-center transition-colors"
        role="group"
      >
        {#if isLoading}
          <p class="text-neutral-500">正在加载...</p>
        {:else if fileCommands.length === 0}
          <Accordion.Root class="mb-2" type="single">
            <Accordion.Item value="help">
              <Accordion.Header>
                <Accordion.Trigger
                  class="flex w-full items-center justify-between rounded-lg px-2 py-1 text-sm text-neutral-500 hover:bg-neutral-100 dark:hover:bg-neutral-800"
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
                <div class="px-2 pt-1 pb-2 text-left text-sm text-neutral-500">
                  <ul class="ml-4 list-disc space-y-1">
                    <li>
                      <strong>拖放</strong>：从 Finder 拖放文件/文件夹到此区域
                    </li>
                    <li>
                      <strong>粘贴路径</strong>：复制文件/文件夹的完整路径后按
                      Cmd+V（如 /Users/xxx/file.txt）
                    </li>
                    <li>
                      <strong>点击按钮</strong>：点击 <Button.Root
                        class="rounded-input bg-dark text-background shadow-mini hover:bg-dark/95 mx-1
	inline-flex h-5 items-center justify-center px-[6px]
	text-[10px] font-semibold active:scale-[0.98] active:transition-all"
                        onclick={handleOpenFileOrFolder}
                      >
                        按钮
                      </Button.Root> 选择文件/文件夹
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
                class="group grid grid-cols-[1fr_auto] items-center gap-2 rounded-lg hover:bg-neutral-200 dark:hover:bg-neutral-700"
              >
                <div class="flex min-w-0 items-center overflow-hidden p-2">
                  {#if item.icon && item.icon_type === "Base64"}
                    <img
                      src={`data:image/png;base64,${item.icon}`}
                      alt="{item.name} icon"
                      class="mr-4 h-8 w-8 flex-shrink-0"
                    />
                  {:else if item.icon}
                    <div
                      class="mr-4 flex h-8 w-8 flex-shrink-0 items-center justify-center rounded-md bg-gray-200 dark:bg-gray-700"
                    >
                      <PhosphorIcon icon={item.icon} class="h-6 w-6" />
                    </div>
                  {:else}
                    <div
                      class="mr-4 flex h-8 w-8 flex-shrink-0 items-center justify-center rounded bg-gray-200 text-gray-500 dark:bg-gray-700 dark:text-gray-300"
                    >
                      {#if item.item_type === "Folder"}
                        <Folder size={24} />
                      {:else if item.item_type === "File"}
                        <File size={24} />
                      {:else}
                        <AppWindow size={24} />
                      {/if}
                    </div>
                  {/if}
                  <div class="min-w-0 flex-1 overflow-hidden">
                    <p class="truncate font-semibold">{item.name}</p>
                    <p class="truncate text-sm text-neutral-500">{item.path}</p>
                  </div>
                </div>

                <div class="flex items-center justify-center pr-2">
                  <Popover.Root>
                    <Popover.Trigger>
                      <button
                        class="rounded-full p-1 text-neutral-400 opacity-0 transition-opacity group-hover:opacity-100 hover:text-red-500"
                        aria-label="删除 {item.name}"
                      >
                        <Trash size={20} />
                      </button>
                    </Popover.Trigger>
                    <Popover.Portal>
                      <Popover.Content
                        class="border-dark-10 bg-background shadow-popover data-[state=open]:animate-in data-[state=closed]:animate-out data-[state=closed]:fade-out-0 data-[state=open]:fade-in-0 data-[state=closed]:zoom-out-95 data-[state=open]:zoom-in-95 data-[side=bottom]:slide-in-from-top-2 data-[side=left]:slide-in-from-right-2 data-[side=right]:slide-in-from-left-2 data-[side=top]:slide-in-from-bottom-2 z-30 w-full max-w-[328px] origin-(--bits-popover-content-transform-origin) rounded-[12px] border p-4"
                        sideOffset={8}
                      >
                        <Popover.Arrow />
                        <h3
                          class="mb-2 text-[14px] leading-5 font-semibold tracking-[-0.01em]"
                        >
                          确认删除？
                        </h3>
                        <div>
                          <Popover.Close>
                            <Button.Root
                              class="rounded-input bg-dark text-background shadow-mini hover:bg-dark/95 inline-flex
	h-6 items-center justify-center px-[12px] text-[10px]
	font-semibold active:scale-[0.98] active:transition-all"
                            >
                              取消
                            </Button.Root>
                          </Popover.Close>
                          <Button.Root
                            class="rounded-input bg-dark text-background shadow-mini hover:bg-dark/95 inline-flex
	h-6 items-center justify-center px-[12px] text-[10px]
	font-semibold active:scale-[0.98] active:transition-all"
                            onclick={() => deleteItem(item.path)}
                          >
                            确认
                          </Button.Root>
                        </div>
                      </Popover.Content>
                    </Popover.Portal>
                  </Popover.Root>
                </div>
              </li>
            {/each}
          </ul>
        {/if}
        {#if isProcessing}
          <div
            class="absolute inset-0 flex items-center justify-center rounded-lg bg-black/30"
          >
            <p class="text-lg text-white">正在处理...</p>
          </div>
        {/if}
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
</main>


