<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";

  import { fuzzyMatch } from "../utils/fuzzyMatch";

  import "../index.css";

  let originAppList = $state<string[]>([]);
  let appList = $state<string[]>([]);
  let selectedIndex = $state<number>(0);

  onMount(async () => {
    console.log("the component has mounted");
    const res = await invoke<string[]>("get_installed_apps");
    console.log("res", res);
    if (res) {
      originAppList = res;
      appList = res;
    }
  });

  listen("window_visibility", (event) => {
    console.log("window_visibility", event.payload);
    // 窗口显示时 input 聚焦
    const input = document.querySelector("input");
    input?.focus();
  });

  const handleInput = (e) => {
    const value = e.target.value;
    const apps = fuzzyMatch(value, originAppList);
    appList = apps;
    selectedIndex = 0;
  };

  const handleKeyDown = (e: KeyboardEvent) => {
    if (e.key === "ArrowDown") {
      e.preventDefault();
      selectedIndex =
        selectedIndex === appList.length - 1 ? 0 : selectedIndex + 1;
    } else if (e.key === "ArrowUp") {
      e.preventDefault();
      selectedIndex =
        selectedIndex === 0 ? appList.length - 1 : selectedIndex - 1;
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
      oninput={handleInput}
    />
    <div class="app-list flex-1 py-2 overflow-auto">
      {#each appList as app, index}
        <div
          role="option"
          aria-selected={selectedIndex === index}
          class="w-full p-2 text-2xl {selectedIndex === index
            ? 'bg-[aqua]'
            : ''}"
        >
          {app}
        </div>
      {/each}
    </div>
  </div>
</main>
