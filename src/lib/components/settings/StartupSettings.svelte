<script lang="ts">
  import { onMount } from "svelte";
  import { Popover, Button } from "bits-ui";
  import { invoke } from "@tauri-apps/api/core";
  import { open } from "@tauri-apps/plugin-dialog";
  import type { LaunchableItem } from "$lib/type";
  import Icon from "$lib/components/Icon.svelte";

  let startupItems = $state<LaunchableItem[]>([]);
  let listContainerEl: HTMLDivElement | undefined = $state();
  let isLoading = $state(true);
  let isProcessing = $state(false);

  async function fetchStartupItems() {
    isLoading = true;
    try {
      startupItems = await invoke("get_startup_items");
    } catch (e) {
      console.error("Failed to get startup items:", e);
    } finally {
      isLoading = false;
    }
  }

  async function handlePaste(event: ClipboardEvent) {
    event.preventDefault();
    const text = event.clipboardData?.getData("text/plain");
    if (!text || isProcessing) return;

    const paths = text
      .split("\n")
      .map((p) => p.trim())
      .filter(Boolean);

    if (paths.length > 0) {
      isProcessing = true;
      try {
        const newItems: LaunchableItem[] = await invoke("add_startup_items", {
          paths,
        });
        startupItems = newItems;
      } catch (e) {
        console.error("Failed to add startup paths:", e);
      } finally {
        isProcessing = false;
      }
    }
  }

  async function deleteItem(path: string) {
    try {
      // 乐观更新UI，立即移除项以获得更好的用户体验
      const originalItems = startupItems;
      startupItems = startupItems.filter((item) => item.path !== path);

      // 调用后端并同步最终状态
      const newItems: LaunchableItem[] = await invoke("remove_startup_item", {
        path,
      });
      startupItems = newItems;
    } catch (e) {
      console.error("Failed to remove startup item:", e);
      // 如果出错，则从后端重新获取列表以回滚状态
      fetchStartupItems();
    }
  }

  function getIconForItem(item: LaunchableItem) {
    if (item.item_type === "Folder") return "folder";
    if (item.item_type === "File") return "file";
    // 'App' 或其他类型的默认图标
    return "app";
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

      const newItems: LaunchableItem[] = await invoke("add_startup_items", {
        paths: selected,
      });
      startupItems = newItems;
    } catch (e) {
      console.error("Failed to add startup paths:", e);
    } finally {
      isProcessing = false;
      // 确保操作结束后（无论成功、失败还是取消），都释放锁
      await invoke("release_window_close_lock");
    }
  };

  onMount(() => {
    fetchStartupItems();
  });

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
</script>

<main class="w-full h-full p-4 flex flex-col">
  <h2 class="text-xl font-bold mb-2">自定义启动项</h2>
  {#if startupItems.length > 0}
    <div class="mb-2 flex items-center text-neutral-500 text-sm">
      <!-- <Icon icon="warning-circle" class="mr-2" /> -->
      可以通过拖放或者粘贴文件/文件夹路径来添加启动项，也可以点击
      <Button.Root
        class="rounded-input bg-dark text-background shadow-mini hover:bg-dark/95 inline-flex
	h-5 items-center justify-center px-[6px] text-[10px]
	font-semibold active:scale-[0.98] active:transition-all mx-1"
        onclick={handleOpenFolder}
      >
        按钮
      </Button.Root>
      添加
    </div>
  {/if}
  <div
    bind:this={listContainerEl}
    class="list-container relative flex-1 border-2 border-dashed border-neutral-300 dark:border-neutral-600 rounded-lg p-4 text-center flex items-center justify-center"
    role="group"
    tabindex="-1"
  >
    {#if isLoading}
      <p class="text-neutral-500">正在加载...</p>
    {:else if startupItems.length === 0}
      <div class="mb-2 flex items-center text-neutral-500 text-sm">
        <!-- <Icon icon="warning-circle" class="mr-2" /> -->
        可以通过拖放或者粘贴文件/文件夹路径来添加启动项，也可以点击
        <Button.Root
          class="rounded-input bg-dark text-background shadow-mini hover:bg-dark/95 inline-flex
	h-5 items-center justify-center px-[6px] text-[10px]
	font-semibold active:scale-[0.98] active:transition-all mx-1"
          onclick={handleOpenFolder}
        >
          按钮
        </Button.Root>
        添加
      </div>
    {:else}
      <ul class="text-left w-full h-full overflow-y-auto custom-scrollbar">
        {#each startupItems as item, index (item.path)}
          <li
            class="flex items-center p-2 hover:bg-neutral-200 dark:hover:bg-neutral-700 rounded-lg group"
          >
            {#if item.icon}
              <img
                src={`data:image/png;base64,${item.icon}`}
                alt="{item.name} icon"
                class="w-8 h-8 mr-4 flex-shrink-0"
              />
            {:else}
              <div
                class="w-8 h-8 mr-4 flex-shrink-0 flex items-center justify-center bg-gray-200 dark:bg-gray-700 rounded"
              >
                <Icon
                  icon={getIconForItem(item)}
                  class="w-5 h-5 text-gray-500"
                />
              </div>
            {/if}
            <div class="flex-1 overflow-hidden">
              <p class="font-semibold truncate">{item.name}</p>
              <p class="text-sm text-neutral-500 truncate">{item.path}</p>
            </div>

            <Popover.Root>
              <Popover.Trigger>
                <button
                  class="ml-2 p-1 rounded-full text-neutral-400 hover:text-red-500 hover:bg-neutral-300 dark:hover:bg-neutral-600 opacity-0 group-hover:opacity-100 transition-opacity"
                  aria-label="删除 {item.name}"
                >
                  <Icon icon="delete" />
                </button>
              </Popover.Trigger>
              <Popover.Portal>
                <Popover.Content
                  class="border-dark-10 bg-background shadow-popover data-[state=open]:animate-in data-[state=closed]:animate-out data-[state=closed]:fade-out-0 data-[state=open]:fade-in-0 data-[state=closed]:zoom-out-95 data-[state=open]:zoom-in-95 data-[side=bottom]:slide-in-from-top-2 data-[side=left]:slide-in-from-right-2 data-[side=right]:slide-in-from-left-2 data-[side=top]:slide-in-from-bottom-2 origin-(--bits-popover-content-transform-origin) z-30 w-full max-w-[328px] rounded-[12px] border p-4"
                  sideOffset={8}
                >
                  <Popover.Arrow />
                  <h3
                    class="text-[14px] font-semibold leading-5 tracking-[-0.01em] mb-2"
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
        class="absolute inset-0 bg-black/30 flex items-center justify-center rounded-lg"
      >
        <p class="text-white text-lg">正在处理...</p>
      </div>
    {/if}
  </div>
</main>
