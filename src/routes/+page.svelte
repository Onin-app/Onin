<script lang="ts">
  import { onDestroy, onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
  import { WebviewWindow } from "@tauri-apps/api/webviewWindow";

  import { goto } from "$app/navigation";
  import { get } from "svelte/store";
  import { fuzzyMatch } from "$lib/utils/fuzzyMatch";
  import { Theme, type AppInfo } from "$lib/type";
  import { theme, getTheme } from "$lib/utils/theme";

  import "../index.css";
  import { escapeHandler } from "$lib/stores/escapeHandler";

  let inputValue = $state<string>("");
  let originAppList = $state<AppInfo[]>([]);
  let appList = $state<AppInfo[]>([]);
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
    unlisten = await listen("apps_updated", (event) => {
      console.log(
        "Received apps_updated event from backend. Refetching list..."
      );
      fetchApps();
    });
  });

  const fetchApps = async () => {
    try {
      console.log("Fetching apps from cache...");
      const res = await invoke<AppInfo[]>("get_installed_apps");
      if (res) {
        originAppList = res;
        appList = res;
      }
      console.log(`Got ${appList.length} apps.`);
    } catch (error) {
      console.error("Failed to get installed apps:", error);
    }
  };

  const unsubscribe = theme.subscribe((value) => {
    currentTheme = value;
  });

  const handleInput = (e) => {
    const value = e.target.value;
    const apps = fuzzyMatch(value, originAppList);
    inputValue = value;
    appList = apps;
    selectedIndex = 0;
  };

  const openApp = async (app: AppInfo) => {
    try {
      await invoke("open_app", {
        path: app.path,
        window: await WebviewWindow.getCurrent(),
      });
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
  class="w-full h-[100vh] p-4 rounded-xl text-neutral-900 dark:text-neutral-100 bg-neutral-100 dark:bg-neutral-800 overflow-hidden"
  data-tauri-drag-region
>
  <div
    class="w-full h-full flex flex-col"
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
          class="w-10 h-10"
          alt="Tauri logo"
        />
      </button>
      <input
        class="w-full p-2 ml-2 text-2xl h-[60px] focus:outline-none focus:ring-0 active:outline-none active:ring-0"
        type="text"
        placeholder="Hi Baize!"
        bind:value={inputValue}
        oninput={handleInput}
      />
    </div>
    <div class="custom-scrollbar app-list flex-1 py-2 overflow-auto">
      {#each appList as app, index}
        <button
          role="option"
          aria-selected={selectedIndex === index}
          class="flex w-full p-2 text-2xl text-left {selectedIndex !== index
            ? 'hover:bg-neutral-200 dark:hover:bg-neutral-700'
            : ''} {selectedIndex === index
            ? 'bg-neutral-300 dark:bg-neutral-600'
            : ''}"
          onclick={() => openApp(app)}
        >
          {#if app.icon}
            <img
              src={`data:image/png;base64,${app.icon}`}
              class="w-8 h-8 mr-2 inline-block"
              alt=""
            />
          {/if}
          <div class="flex flex-col">
            {app.name}
            <span class="text-xs text-neutral-400 dark:text-neutral-500">
              {app.path}
            </span>
          </div>
        </button>
      {/each}
    </div>
  </div>
</main>
