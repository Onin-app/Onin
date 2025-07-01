<script lang="ts">
  import { onDestroy, onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { Label, RadioGroup, Switch } from "bits-ui";

  import { theme, toggleTheme } from "$lib/utils/theme";
  import { Theme } from "$lib/type";
  import SetItem from "./SetItem.svelte";

  const themeList: { value: Theme; label: string }[] = [
    {
      value: Theme.SYSTEM,
      label: "跟随系统",
    },
    {
      value: Theme.LIGHT,
      label: "明亮",
    },
    {
      value: Theme.DARK,
      label: "暗黑",
    },
  ];
  let currentTheme = $state<Theme>(Theme.DARK);
  let autostartEnabled = $state<boolean>(false);
  let trayIconEnabled = $state<boolean>(false);

  const getTheme = () => currentTheme;
  const setTheme = (value: Theme) => {
    currentTheme = value;
    toggleTheme(value);
  };

  const handleAutostartToggle = async () => {
    try {
      // `bind:checked` 会在 onchange 之前更新 autostartEnabled 的值。
      // 所以，如果 autostartEnabled 为 true，意味着用户刚刚打开了开关，我们应该调用 enable。
      if (autostartEnabled) {
        await invoke("plugin:autostart|enable");
      } else {
        await invoke("plugin:autostart|disable");
      }
      // 从后端重新获取状态以确保UI同步
      autostartEnabled = await invoke("plugin:autostart|is_enabled");
    } catch (error) {
      console.error("Failed to toggle autostart:", error);
      // 如果设置失败，将UI状态回滚
      autostartEnabled = !autostartEnabled;
    }
  };

  const handleTrayIconToggle = async () => {
    try {
      // `bind:checked` 会提前更新 trayIconEnabled 的值
      await invoke("set_tray_visibility", { visible: trayIconEnabled });
      // 从后端重新获取状态以确保UI同步
      trayIconEnabled = await invoke("is_tray_visible");
    } catch (error) {
      console.error("Failed to toggle tray icon visibility:", error);
      // 如果设置失败，将UI状态回滚
      trayIconEnabled = !trayIconEnabled;
    }
  };

  const unsubscribe = theme.subscribe((value) => {
    currentTheme = value;
  });

  onMount(async () => {
    autostartEnabled = await invoke("plugin:autostart|is_enabled");
    try {
      trayIconEnabled = await invoke("is_tray_visible");
    } catch (e) {
      console.error("Failed to get tray visibility state:", e);
    }
  });

  onDestroy(unsubscribe);
</script>

<main class="w-full h-full p-4">
  <h2 class="text-xl font-bold">主题设置</h2>
  <SetItem title="主题">
    {#snippet content()}
      <RadioGroup.Root
        class="flex gap-4 text-sm font-medium"
        bind:value={getTheme, setTheme}
      >
        {#each themeList as theme}
          <div
            class="text-foreground group flex select-none items-center transition-all"
          >
            <RadioGroup.Item
              id={theme.value}
              value={theme.value}
              class="cursor-pointer border-border-input bg-background hover:border-dark-40 data-[state=checked]:border-foreground data-[state=checked]:border-6 size-5 shrink-0 cursor-default rounded-full border transition-all duration-100 ease-in-out"
            />
            <Label.Root for={theme.value} class="pl-3 cursor-pointer">
              {theme.label}
            </Label.Root>
          </div>
        {/each}
      </RadioGroup.Root>
    {/snippet}
  </SetItem>

  <h2 class="text-xl font-bold mt-4">系统设置</h2>
  <SetItem title="开机自启">
    {#snippet content()}
      <Switch.Root
        bind:checked={autostartEnabled}
        onCheckedChange={handleAutostartToggle}
        name="autoStart"
        class="focus-visible:ring-foreground focus-visible:ring-offset-background data-[state=checked]:bg-foreground data-[state=unchecked]:bg-dark-10 data-[state=unchecked]:shadow-mini-inset dark:data-[state=checked]:bg-foreground focus-visible:outline-hidden peer inline-flex h-[24px] min-h-[24px] w-[40px] shrink-0 cursor-pointer items-center rounded-full px-[3px] transition-colors focus-visible:ring-2 focus-visible:ring-offset-2 disabled:cursor-not-allowed disabled:opacity-50"
      >
        <Switch.Thumb
          class="bg-background data-[state=unchecked]:shadow-mini dark:border-background/30 dark:bg-foreground dark:shadow-popover pointer-events-none block size-[20px] shrink-0 rounded-full transition-transform data-[state=checked]:translate-x-[14px] data-[state=unchecked]:translate-x-0 dark:border dark:data-[state=unchecked]:border"
        />
      </Switch.Root>
    {/snippet}
  </SetItem>
  <SetItem title="任务栏中显示图标">
    {#snippet content()}
      <Switch.Root
        bind:checked={trayIconEnabled}
        onCheckedChange={handleTrayIconToggle}
        name="trayIcon"
        class="focus-visible:ring-foreground focus-visible:ring-offset-background data-[state=checked]:bg-foreground data-[state=unchecked]:bg-dark-10 data-[state=unchecked]:shadow-mini-inset dark:data-[state=checked]:bg-foreground focus-visible:outline-hidden peer inline-flex h-[24px] min-h-[24px] w-[40px] shrink-0 cursor-pointer items-center rounded-full px-[3px] transition-colors focus-visible:ring-2 focus-visible:ring-offset-2 disabled:cursor-not-allowed disabled:opacity-50"
      >
        <Switch.Thumb
          class="bg-background data-[state=unchecked]:shadow-mini dark:border-background/30 dark:bg-foreground dark:shadow-popover pointer-events-none block size-[20px] shrink-0 rounded-full transition-transform data-[state=checked]:translate-x-[14px] data-[state=unchecked]:translate-x-0 dark:border dark:data-[state=unchecked]:border"
        />
      </Switch.Root>
    {/snippet}
  </SetItem>
</main>
