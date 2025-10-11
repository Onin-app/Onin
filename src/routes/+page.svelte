<script lang="ts">
  import { onDestroy, onMount } from "svelte";
  import { get } from "svelte/store";
  import { ScrollArea } from "bits-ui";
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
  import { WebviewWindow } from "@tauri-apps/api/webviewWindow";

  import { goto } from "$app/navigation";
  import { fuzzyMatch } from "$lib/utils/fuzzyMatch";
  import { Theme, type LaunchableItem } from "$lib/type";
  import { theme, getTheme } from "$lib/utils/theme";
  import { escapeHandler } from "$lib/stores/escapeHandler";
  import Icon from "$lib/components/Icon.svelte";

  import "../index.css";

  let inputValue = $state<string>("");
  let originAppList = $state<LaunchableItem[]>([]);
  let appList = $state<LaunchableItem[]>([]);
  let selectedIndex = $state<number>(0);
  let currentTheme = $state<Theme>(Theme.DARK);
  let unlisten = $state<null | (() => void)>(null);

  const handleEsc = () => {
    console.log("Main page ESC handler executing");
    inputValue = "";
    appList = originAppList;
    selectedIndex = 0;
    invoke("close_main_window");
  };

  onMount(async () => {
    console.log("Main page component has mounted");
    // Register this page's ESC handler with the global store
    escapeHandler.set(handleEsc);

    // 1. 立即获取一次数据
    await fetchApps();

    // Fetch initial data. The visibility listener is now handled in the layout.
    // (async () => {
    //   const res = await invoke<AppInfo[]>("get_installed_apps");
    //   if (res) {
    //     originAppList = res;
    //     appList = res;
    //   }
    // })();

    // 2. 监听后端的更新通知
    const unlistenAppsUpdated = await listen("apps_updated", (event) => {
      console.log(
        "Received apps_updated event from backend. Refetching list...",
      );
      fetchApps();
    });

    const unlistenCommandsReady = await listen("commands_ready", (event) => {
      console.log(
        "Received commands_ready event from backend. Refetching list...",
      );
      fetchApps();
    });

    unlisten = () => {
      unlistenAppsUpdated();
      unlistenCommandsReady();
    };
  });

  const fetchApps = async () => {
    try {
      console.log("Fetching all launchable items...");
      const res = await invoke<LaunchableItem[]>("get_all_launchable_items");
      console.log("本机软件列表: ", res);
      if (res) {
        originAppList = res;
        appList = res;
      }
      console.log(`Got ${appList.length} apps.`);
    } catch (error) {
      console.error("Failed to get all launchable items:", error);
    }
  };

  const unsubscribe = theme.subscribe((value) => {
    currentTheme = value;
  });

  const handleInput = (
    e: Event & { currentTarget: EventTarget & HTMLInputElement },
  ) => {
    const value = e.currentTarget.value;
    const apps = fuzzyMatch(value, originAppList);
    inputValue = value;
    appList = apps;
    selectedIndex = 0;
  };

  const openApp = async (app: LaunchableItem) => {
    try {
      if (app.action) {
        await invoke("execute_command", {
          name: app.action,
          window: await WebviewWindow.getCurrent(),
        });
      } else if (app.source === "FileCommand") {
        // Handle custom items that might not have an action
        await invoke("open_app", {
          path: app.path,
          window: await WebviewWindow.getCurrent(),
        });
      }
      inputValue = "";
      appList = originAppList;
      selectedIndex = 0;
    } catch (error) {
      console.error("Failed to open app:", error);
    }
  };

  const handleKeyDown = (e: KeyboardEvent) => {
    if (e.key === "ArrowDown" || (e.key === "Tab" && !e.shiftKey)) {
      e.preventDefault();
      selectedIndex =
        selectedIndex === appList.length - 1 ? 0 : selectedIndex + 1;
    } else if (e.key === "ArrowUp" || (e.key === "Tab" && e.shiftKey)) {
      e.preventDefault();
      selectedIndex =
        selectedIndex === 0 ? appList.length - 1 : selectedIndex - 1;
    } else if (e.key === "Enter" && appList.length > 0) {
      e.preventDefault();
      openApp(appList[selectedIndex]);
    }

    // 保持选中项在可见范围内
    const container = document.querySelector(".app-list");
    const selectedItem = container?.children[selectedIndex];
    if (selectedItem) {
      selectedItem.scrollIntoView({
        behavior: "auto",
        block: "nearest",
      });
    }
  };

  const handleToSettings = () => {
    goto("/settings");
  };

  onDestroy(() => {
    // Clean up the theme subscription
    unsubscribe && unsubscribe();
    // As a safeguard, reset the escape handler if it's still ours
    if (get(escapeHandler) === handleEsc) {
      escapeHandler.set(() => {});
    }
    // 组件销毁时，清理监听器
    if (unlisten) {
      unlisten();
    }
  });
</script>

<main
  class="h-[100vh] w-full overflow-hidden rounded-xl bg-neutral-100 p-4 text-neutral-900 dark:bg-neutral-800 dark:text-neutral-100"
  data-tauri-drag-region
>
  <div
    class="flex h-full w-full flex-col"
    role="listbox"
    tabindex="0"
    onkeydown={handleKeyDown}
  >
    <div class="flex items-center pb-2">
      <button class="cursor-pointer" onclick={handleToSettings}>
        <img
          src="/ff_logo_{getTheme(currentTheme) === Theme.DARK
            ? Theme.LIGHT
            : Theme.DARK}.svg"
          class="h-10 w-10"
          alt="Tauri logo"
        />
      </button>
      <input
        class="ml-2 h-[60px] w-full p-2 text-2xl focus:ring-0 focus:outline-none active:ring-0 active:outline-none"
        type="text"
        placeholder="Hi Baize!"
        bind:value={inputValue}
        oninput={handleInput}
      />
    </div>
    <ScrollArea.Root
      class="relative flex-1 overflow-hidden rounded-[10px] border px-2 py-2"
    >
      <ScrollArea.Viewport class="h-full w-full">
        <div class="ajp-list overflow-auto">
          {#each appList as app, index}
            <button
              role="option"
              aria-selected={selectedIndex === index}
              class="flex w-full rounded p-2 text-left text-2xl {selectedIndex !==
              index
                ? 'hover:bg-neutral-200 dark:hover:bg-neutral-700'
                : ''} {selectedIndex === index
                ? 'bg-neutral-300 dark:bg-neutral-600'
                : ''}"
              onclick={() => openApp(app)}
            >
              {#if app.icon}
                {#if app.icon_type === "Iconfont"}
                  <div
                    class="mr-2 flex h-8 w-8 items-center justify-center rounded-md bg-gray-200 dark:bg-gray-700"
                  >
                    <Icon icon={app.icon} class="h-6 w-6" />
                  </div>
                {:else if app.icon}
                  <img
                    src={`data:image/png;base64,${app.icon}`}
                    class="mr-2 inline-block h-8 w-8"
                    alt=""
                  />
                {/if}
              {/if}
              <div class="flex flex-1 flex-col">
                <div class="flex items-center justify-between">
                  <span>
                    {app.name}
                  </span>
                  <span
                    class="rounded-md bg-neutral-200 px-1.5 py-0.5 text-xs text-neutral-700 dark:bg-neutral-700 dark:text-neutral-300"
                  >
                    {app.source_display || app.source}
                  </span>
                </div>
                {#if app.source !== "Command"}
                  <span class="text-neutral-399 text-xs dark:text-neutral-500">
                    {app.path}
                  </span>
                {/if}
              </div>
            </button>
          {/each}
        </div>
      </ScrollArea.Viewport>
      <ScrollArea.Scrollbar
        orientation="vertical"
        class="bg-muted hover:bg-dark-10 data-[state=visible]:animate-in data-[state=hidden]:animate-out data-[state=hidden]:fade-out-0 data-[state=visible]:fade-in-0 flex w-2.5 touch-none rounded-full border-l border-l-transparent p-px transition-all duration-200 select-none hover:w-3"
      >
        <ScrollArea.Thumb class="bg-muted-foreground flex-1 rounded-full" />
      </ScrollArea.Scrollbar>
      <ScrollArea.Scrollbar
        orientation="horizontal"
        class="bg-muted hover:bg-dark-10 flex h-2.5 touch-none rounded-full border-t border-t-transparent p-px transition-all duration-200 select-none hover:h-3 "
      >
        <ScrollArea.Thumb class="bg-muted-foreground rounded-full" />
      </ScrollArea.Scrollbar>
      <ScrollArea.Corner />
    </ScrollArea.Root>
  </div>
</main>
