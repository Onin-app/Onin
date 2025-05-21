<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";

  import "../index.css";

  let originAppList = $state<string[]>([]);
  let appList = $state<string[]>([]);

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
    console.log("e", e.target.value);
    const value = e.target.value.toLowerCase();
    const apps = originAppList.filter((app) =>
      app.toLowerCase().includes(value),
    );
    appList = apps;
  };
</script>

<main
  class="w-full h-[100vh] p-4 rounded-xl bg-[aquamarine] overflow-hidden flex flex-col"
>
  <input
    class="w-full p-2 text-2xl h-[60px]"
    type="text"
    placeholder="Hi Baize!"
    oninput={handleInput}
  />
  <div class="flex-1 py-2 overflow-auto">
    {#each appList as app}
      <div class="w-full p-2 text-2xl">{app}</div>
    {/each}
  </div>
</main>
