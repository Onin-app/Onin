<script lang="ts">
  import { onMount } from "svelte";

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

  const handleClickSetting = (setting: SettingItem) => {
    activeSetting = setting;
  };

  onMount(async () => {
    console.log("the component has mounted");
  });
</script>

<main class="w-full h-[100vh] bg-[aquamarine] flex">
  <div class="left w-1/5 h-full bg-white p-4">
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
    <activeSetting.component />
  </div>
</main>
