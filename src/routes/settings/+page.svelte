<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { get } from "svelte/store";

  import { goto } from "$app/navigation";
  import GeneralSettings from "$lib/components/settings/GeneralSettings.svelte";
  import StartupSettings from "$lib/components/settings/StartupSettings.svelte";
  import { escapeHandler } from "$lib/stores/escapeHandler";

  interface SettingItem {
    name: string;
    id: string;
    component: typeof GeneralSettings | typeof StartupSettings;
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
      name: "启动设置",
      id: "startup",
      component: StartupSettings,
      icon: "icon-startup",
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
  class="w-full h-[100vh] text-neutral-900 dark:text-neutral-100 bg-neutral-100 dark:bg-neutral-800 flex p-4"
  data-tauri-drag-region
>
  <div
    class="left w-1/5 h-full p-4 border-r border-neutral-200 dark:border-neutral-700"
  >
    <ul class="w-full h-full flex flex-col justify-center">
      {#each settings as setting}
        <li class="mb-2 mt-2">
          <button
            type="button"
            class="cursor-pointer"
            class:active={activeSetting.id === setting.id}
            onclick={() => handleClickSetting(setting)}
          >
            {setting.name}
          </button>
        </li>
      {/each}
    </ul>
  </div>
  <div class="main flex-1 h-full overflow-auto">
    <ActiveComponent />
  </div>
</main>
