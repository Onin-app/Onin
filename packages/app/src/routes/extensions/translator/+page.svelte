<script lang="ts">
  import { WebviewWindow } from "@tauri-apps/api/webviewWindow";
  import { emit } from "@tauri-apps/api/event";
  import { Tabs, ScrollArea } from "bits-ui";
  import "../../../index.css";

  let engine = $state("sougou");

  const engines = [
    { id: "sougou", label: "搜狗" },
    { id: "google", label: "Google" },
    { id: "baidu", label: "百度" },
  ];

  const switchEngine = async (newEngine: string) => {
    if (engine === newEngine) return;
    engine = newEngine;
    await emit("translator_switch", { engine: newEngine });
  };
</script>

<div
  class="flex h-full w-full items-center justify-center border-b border-gray-200 bg-gray-50 dark:border-gray-700 dark:bg-gray-900"
>
  <Tabs.Root
    value={engine}
    onValueChange={(val) => switchEngine(val)}
    class="w-full"
  >
    <ScrollArea.Root class="w-full" type="hover">
      <ScrollArea.Viewport class="w-full overflow-y-hidden">
        <Tabs.List
          class="flex w-max min-w-full gap-1 rounded-lg bg-gray-200 p-1 text-xs font-medium text-gray-500 hover:bg-gray-200/80 dark:bg-gray-800 dark:text-gray-400"
        >
          {#each engines as e}
            <Tabs.Trigger
              value={e.id}
              class="flex flex-shrink-0 items-center justify-center rounded-md px-3 py-1 transition-all focus-visible:ring-2 focus-visible:ring-blue-500 focus-visible:ring-offset-2 focus-visible:outline-none disabled:pointer-events-none disabled:opacity-50 data-[state=active]:bg-white data-[state=active]:text-gray-900 data-[state=active]:shadow-sm dark:data-[state=active]:bg-gray-950 dark:data-[state=active]:text-gray-50"
            >
              {e.label}
            </Tabs.Trigger>
          {/each}
        </Tabs.List>
      </ScrollArea.Viewport>
      <ScrollArea.Scrollbar
        orientation="horizontal"
        class="bg-muted hover:bg-dark-10 data-[state=visible]:animate-in data-[state=hidden]:animate-out data-[state=hidden]:fade-out-0 data-[state=visible]:fade-in-0 flex h-1.5 touch-none rounded-full border-t border-t-transparent p-px transition-all duration-200 select-none hover:h-3"
      >
        <ScrollArea.Thumb class="bg-muted-foreground rounded-full" />
      </ScrollArea.Scrollbar>
      <ScrollArea.Corner />
    </ScrollArea.Root>
  </Tabs.Root>
</div>

<style>
  :global(body) {
    margin: 0;
    padding: 0;
    overflow: hidden;
  }
</style>
