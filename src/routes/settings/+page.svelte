<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { get } from "svelte/store";
  import { Button } from "bits-ui";

  import { goto } from "$app/navigation";
  import GeneralSettings from "$lib/components/settings/GeneralSettings.svelte";
  import FileCommandSettings from "$lib/components/settings/FileCommandSettings.svelte";
  import CommandSettings from "$lib/components/settings/CommandSettings.svelte";
  import { escapeHandler } from "$lib/stores/escapeHandler";

  interface SettingItem {
    name: string;
    id: string;
    component: typeof GeneralSettings | typeof FileCommandSettings | typeof CommandSettings;
    icon: string;
  }

  const settings: SettingItem[] = [
    {
      name: "通用设置",
      id: "general",
      component: GeneralSettings,
      icon: "icon-general",
    },
    {
      name: "文件启动",
      id: "startup",
      component: FileCommandSettings,
      icon: "icon-startup",
    },
    {
      name: "指令设置",
      id: "commands",
      component: CommandSettings,
      icon: "icon-commands",
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
  class="flex h-[100vh] w-full bg-neutral-100 text-neutral-900 dark:bg-neutral-800 dark:text-neutral-100"
  data-tauri-drag-region
>
  <div
    class="left relative h-full w-1/5 border-r border-neutral-200 p-4 dark:border-neutral-700"
  >
    <ul class="flex h-full w-full flex-col justify-center">
      {#each settings as setting}
        <li
          class="rounded {activeSetting.id === setting.id
            ? 'bg-neutral-300 dark:bg-neutral-600'
            : 'hover:bg-neutral-200 dark:hover:bg-neutral-700'}"
        >
          <Button.Root
            class="w-full cursor-pointer p-2 text-left"
            onclick={() => handleClickSetting(setting)}
          >
            {setting.name}
          </Button.Root>
        </li>
      {/each}
    </ul>
    <Button.Root
      class="absolute right-[0] bottom-[0] left-[0] w-full cursor-pointer p-2 hover:bg-neutral-200 dark:hover:bg-neutral-700"
      onclick={() => goto("/plugins")}
    >
      插件管理
    </Button.Root>
  </div>
  <div class="main h-full flex-1 overflow-auto">
    <ActiveComponent />
  </div>
</main>
