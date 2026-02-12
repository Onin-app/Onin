<script lang="ts">
  import { WebviewWindow } from "@tauri-apps/api/webviewWindow";
  import { emit } from "@tauri-apps/api/event";
  import { Tabs } from "bits-ui";
  import "../../../index.css";

  let engine = $state("google");

  const switchEngine = async (newEngine: string) => {
    if (engine === newEngine) return;
    engine = newEngine;
    await emit("translator_switch", { engine: newEngine });
  };
</script>

<div
  class="flex h-full w-full items-center justify-center border-b border-gray-200 bg-gray-50 dark:border-gray-700 dark:bg-gray-900"
>
  <Tabs.Root value={engine} onValueChange={(val) => switchEngine(val)} class="w-[400px]">
    <Tabs.List
      class="grid w-full grid-cols-2 rounded-lg bg-gray-200 p-1 text-sm font-medium text-gray-500 hover:bg-gray-200/80 dark:bg-gray-800 dark:text-gray-400"
    >
      <Tabs.Trigger
        value="google"
        class="flex items-center justify-center rounded-md py-1.5 transition-all focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-blue-500 focus-visible:ring-offset-2 disabled:pointer-events-none disabled:opacity-50 data-[state=active]:bg-white data-[state=active]:text-gray-900 data-[state=active]:shadow-sm dark:data-[state=active]:bg-gray-950 dark:data-[state=active]:text-gray-50"
      >
        Google Translate
      </Tabs.Trigger>
      <Tabs.Trigger
        value="deepl"
        class="flex items-center justify-center rounded-md py-1.5 transition-all focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-blue-500 focus-visible:ring-offset-2 disabled:pointer-events-none disabled:opacity-50 data-[state=active]:bg-white data-[state=active]:text-gray-900 data-[state=active]:shadow-sm dark:data-[state=active]:bg-gray-950 dark:data-[state=active]:text-gray-50"
      >
        DeepL
      </Tabs.Trigger>
    </Tabs.List>
  </Tabs.Root>
</div>

<style>
  :global(body) {
    margin: 0;
    padding: 0;
    overflow: hidden;
  }
</style>
