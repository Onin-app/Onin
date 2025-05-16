<script lang="ts">
  import { onMount } from "svelte";
  import { listen } from "@tauri-apps/api/event";
  import "../index.css";

  onMount(() => {
    console.log("the component has mounted");
  });

  listen("window_visibility", (event) => {
    console.log("window_visibility", event.payload);
    // 窗口显示时 input 聚焦
    const input = document.querySelector("input");
    input?.focus();
  });

  const handleInput = (e) => {
    console.log("e", e.target.value);
  };
</script>

<main class="w-full h-[100vh] p-4 rounded-xl bg-yellow-200">
  <input
    class="w-full h-1/3 p-2 text-2xl"
    type="text"
    placeholder="Hi Baize!"
    oninput={handleInput}
  />
</main>
