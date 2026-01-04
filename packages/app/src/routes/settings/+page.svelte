<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { get } from "svelte/store";
  import { Button } from "bits-ui";

  import { goto } from "$app/navigation";
  import {
    Gear,
    RocketLaunch,
    TerminalWindow,
    Keyboard,
    PlugsConnected,
  } from "phosphor-svelte";

  import GeneralSettings from "$lib/components/settings/GeneralSettings.svelte";
  import FileCommandSettings from "$lib/components/settings/FileCommandSettings.svelte";
  import CommandSettings from "$lib/components/settings/CommandSettings.svelte";
  import ShortcutSettings from "$lib/components/settings/ShortcutSettings.svelte";
  import { escapeHandler } from "$lib/stores/escapeHandler";

  interface SettingItem {
    name: string;
    id: string;
    component:
      | typeof GeneralSettings
      | typeof FileCommandSettings
      | typeof CommandSettings
      | typeof ShortcutSettings;
    icon: any;
  }

  const settings: SettingItem[] = [
    {
      name: "通用设置",
      id: "general",
      component: GeneralSettings,
      icon: Gear,
    },
    {
      name: "文件启动",
      id: "startup",
      component: FileCommandSettings,
      icon: RocketLaunch,
    },
    {
      name: "指令设置",
      id: "commands",
      component: CommandSettings,
      icon: TerminalWindow,
    },
    {
      name: "全局快捷键",
      id: "shortcuts",
      component: ShortcutSettings,
      icon: Keyboard,
    },
  ];

  let activeSetting = $state<SettingItem>(settings[0]);
  let ActiveComponent = $derived(activeSetting.component);

  const handleEsc = () => {
    console.log("Settings page ESC handler executing");
    goto("/");
  };

  onMount(() => {
    console.log("Settings component has mounted");
    // Register this page's ESC handler
    escapeHandler.set(handleEsc);
  });

  onDestroy(() => {
    // On destroy, reset the handler if it's still ours
    if (get(escapeHandler) === handleEsc) {
      escapeHandler.set(() => {});
    }
  });

  const handleClickSetting = (setting: SettingItem) => {
    activeSetting = setting;
  };
</script>

<main
  class="flex h-screen w-full overflow-hidden bg-neutral-50 text-neutral-900 selection:bg-neutral-200 dark:bg-neutral-900 dark:text-neutral-100 dark:selection:bg-neutral-700"
  data-tauri-drag-region
>
  <aside
    class="flex w-52 flex-col border-r border-neutral-200 bg-neutral-100/50 p-3 pt-6 dark:border-neutral-800 dark:bg-neutral-900/50"
    data-tauri-drag-region
  >
    <div
      class="mb-6 px-3 text-sm font-medium text-neutral-500"
      data-tauri-drag-region
    >
      设置
    </div>
    <nav class="flex flex-1 flex-col gap-1">
      {#each settings as setting}
        <Button.Root
          class="flex w-full items-center gap-3 rounded-lg px-3 py-2 text-sm font-medium transition-colors {activeSetting.id ===
          setting.id
            ? 'bg-white text-neutral-900 shadow-sm dark:bg-neutral-800 dark:text-white'
            : 'text-neutral-600 hover:bg-neutral-200/50 hover:text-neutral-900 dark:text-neutral-400 dark:hover:bg-neutral-800/50 dark:hover:text-white'}"
          onclick={() => handleClickSetting(setting)}
        >
          <svelte:component this={setting.icon} size={18} />
          {setting.name}
        </Button.Root>
      {/each}
    </nav>

    <div
      class="mt-auto border-t border-neutral-200 pt-4 dark:border-neutral-800"
    >
      <Button.Root
        class="flex w-full items-center gap-3 rounded-lg px-3 py-2 text-sm font-medium text-neutral-600 transition-colors hover:bg-neutral-200/50 hover:text-neutral-900 dark:text-neutral-400 dark:hover:bg-neutral-800/50 dark:hover:text-white"
        onclick={() => goto("/plugins")}
      >
        <PlugsConnected size={18} />
        插件管理
      </Button.Root>
    </div>
  </aside>

  <div
    class="flex-1 overflow-hidden bg-white p-6 dark:bg-neutral-950"
    data-tauri-drag-region
  >
    <div class="mx-auto flex h-full max-w-3xl flex-col">
      <ActiveComponent />
    </div>
  </div>
</main>
