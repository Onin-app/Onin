<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { listen, type UnlistenFn } from "@tauri-apps/api/event";

  import { goto } from "$app/navigation";
  import GeneralSettings from "$lib/components/settings/GeneralSettings.svelte";

  interface SettingItem {
    name: string;
    id: string;
    component: typeof import("$lib/components/settings/GeneralSettings.svelte").default;
    icon: string;
  }

  const settings: SettingItem[] = [
    {
      name: "通用设置",
      id: "general",
      component: GeneralSettings,
      icon: "icon-general",
    },
  ];

  let activeSetting = $state<SettingItem>(settings[0]);
  let ActiveComponent = $derived(activeSetting.component);

  onMount(() => {
    console.log("the component has mounted");

    let unlistenEsc: UnlistenFn | undefined;

    const setup = async () => {
      unlistenEsc = await listen("esc_key_pressed", () => {
        console.log("settings window esc_key_pressed");
        goto("/");
      });
    };
    setup();

    return () => {
      unlistenEsc?.();
    };
  });

  const handleClickSetting = (setting: SettingItem) => {
    activeSetting = setting;
  };
</script>

<main
  class="w-full h-[100vh] text-neutral-900 dark:text-neutral-100 bg-neutral-100 dark:bg-neutral-800 flex"
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
  <div class="main flex-1 h-full">
    <ActiveComponent />
  </div>
</main>
