<script lang="ts">
  import { onDestroy, onMount } from "svelte";
  import { Popover, Button, ScrollArea } from "bits-ui";
  import { invoke } from "@tauri-apps/api/core";
  import { open } from "@tauri-apps/plugin-dialog";
  import type { LaunchableItem } from "$lib/type";
  import PhosphorIcon from "$lib/components/PhosphorIcon.svelte";
  // import { Webview } from "@tauri-apps/api/webview";
  import { getCurrentWebview } from "@tauri-apps/api/webview";
  import { TauriEvent } from "@tauri-apps/api/event";
  import { Trash, Folder, File, AppWindow } from "phosphor-svelte";

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
    const text = event.clipboardData?.getData("text/plain");
    if (!text) return;
    const paths = text
      .split("\n")
      .map((p) => p.trim())
      .filter(Boolean);
    await addItems(paths);
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

  const handleOpenFolder = async () => {
    // 解决问题 #2：在打开文件对话框前，请求后端锁定窗口
    await invoke("acquire_window_close_lock");
    isProcessing = true;
    try {
      const selected = await open({
        multiple: true,
        directory: false,
      });

      if (!selected || selected.length === 0) {
        // 用户取消了选择，直接返回。finally 块会确保锁被释放。
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
      // 确保操作结束后（无论成功、失败还是取消），都释放锁
      await invoke("release_window_close_lock");
    }
  };

  // 监听拖放文件到指定区域
  const listenDragDrop = async () => {
    // 修复：正确赋值给 state 变量，以便组件销毁时可以注销监听器
    unlistenDragDrop = await getCurrentWebview().onDragDropEvent(async (e) => {
      console.log("拖放移动", e);
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

  // 使用 $effect 可以安全地在元素绑定后添加/移除事件监听器
  $effect(() => {
    const el = listContainerEl;
    if (el) {
      el.addEventListener("paste", handlePaste);
      return () => {
        el.removeEventListener("paste", handlePaste);
      };
    }
  });

  onMount(() => {
    fetchFileCommands();
    listenDragDrop();
  });

  onDestroy(() => {
    unlistenDragDrop?.();
  });
</script>

<main class="flex h-full w-full flex-col">
  <h2 class="mb-2 text-xl font-bold">文件启动设置</h2>
  {#if fileCommands.length > 0}
    <div class="mb-2 flex items-center text-sm text-neutral-500">
      可以通过拖放或者粘贴文件/文件夹路径来添加文件，也可以点击
      <Button.Root
        class="rounded-input bg-dark text-background shadow-mini hover:bg-dark/95 mx-1
	inline-flex h-5 items-center justify-center px-[6px]
	text-[10px] font-semibold active:scale-[0.98] active:transition-all"
        onclick={handleOpenFolder}
      >
        按钮
      </Button.Root>
      添加
    </div>
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
        tabindex="-1"
      >
        {#if isLoading}
          <p class="text-neutral-500">正在加载...</p>
        {:else if fileCommands.length === 0}
          <div class="mb-2 flex items-center text-sm text-neutral-500">
            可以通过拖放或者粘贴文件/文件夹路径来添加文件指令，也可以点击
            <Button.Root
              class="rounded-input bg-dark text-background shadow-mini hover:bg-dark/95 mx-1
	inline-flex h-5 items-center justify-center px-[6px]
	text-[10px] font-semibold active:scale-[0.98] active:transition-all"
              onclick={handleOpenFolder}
            >
              按钮
            </Button.Root>
            添加
          </div>
        {:else}
          <ul class="h-full w-full overflow-y-auto text-left">
            {#each fileCommands as item, index (item.path)}
              <li
                class="group flex items-center rounded-lg p-2 hover:bg-neutral-200 dark:hover:bg-neutral-700"
              >
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
                <div class="flex-1 overflow-hidden">
                  <p class="truncate font-semibold">{item.name}</p>
                  <p class="truncate text-sm text-neutral-500">{item.path}</p>
                </div>

                <Popover.Root>
                  <Popover.Trigger>
                    <button
                      class="ml-2 rounded-full p-1 text-neutral-400 opacity-0 transition-opacity group-hover:opacity-100 hover:text-red-500"
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
