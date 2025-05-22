<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
  import { WebviewWindow } from "@tauri-apps/api/webviewWindow";

  import { fuzzyMatch } from "../utils/fuzzyMatch";
  import type { AppInfo } from "../type";

  import "../index.css";

  let inputValue = $state<string>("");
  let originAppList = $state<AppInfo[]>([]);
  let appList = $state<AppInfo[]>([]);
  let selectedIndex = $state<number>(0);

  onMount(async () => {
    console.log("the component has mounted");
    const res = await invoke<AppInfo[]>("get_installed_apps");
    console.log("res", res);
    if (res) {
      originAppList = res;
      appList = res;
    }
  });

  listen("window_visibility", (event) => {
    console.log("window_visibility", event.payload);
    if (event.payload) {
      // 窗口显示时 input 聚焦
      const input = document.querySelector("input");
      input?.focus();
    } else {
      // selectedIndex = 0;
    }
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
</script>

<main class="w-full h-[100vh] p-4 rounded-xl bg-[aquamarine] overflow-hidden">
  <div
    class="w-full h-full flex flex-col"
    role="listbox"
    tabindex="0"
    onkeydown={handleKeyDown}
  >
    <input
      class="w-full p-2 text-2xl h-[60px]"
      type="text"
      placeholder="Hi Baize!"
      bind:value={inputValue}
      oninput={handleInput}
    />
    <div class="app-list flex-1 py-2 overflow-auto">
      {#each appList as app, index}
        <button
          role="option"
          aria-selected={selectedIndex === index}
          class="flex w-full p-2 text-2xl text-left {selectedIndex !== index
            ? 'hover:bg-[rgba(0,255,255,0.5)]'
            : ''} {selectedIndex === index ? 'bg-[aqua]' : ''}"
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
            <span class="text-xs text-gray-400">
              {app.path}
            </span>
          </div>
        </button>
      {/each}
    </div>
  </div>
</main>
