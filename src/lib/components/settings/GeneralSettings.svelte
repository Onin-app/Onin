<script lang="ts">
  import { onDestroy, onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { Label, RadioGroup, Switch, Button } from "bits-ui";

  import { theme, toggleTheme } from "$lib/utils/theme";
  import { Theme } from "$lib/type";
  import { detachWindowShortcut } from "$lib/stores/shortcuts";

  import SetItem from "./SetItem.svelte";
  import ShortcutInput from "./ShortcutInput.svelte";

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
  let shortcut = $state<string>("");

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
      shortcut = await invoke("get_toggle_shortcut");
    } catch (e) {
      console.error("Failed to get shortcut:", e);
    }
    try {
      trayIconEnabled = await invoke("is_tray_visible");
    } catch (e) {
      console.error("Failed to get tray visibility state:", e);
    }
  });

  onDestroy(unsubscribe);
</script>

<main class="h-full w-full p-4">
  <h2 class="text-xl font-bold">主题设置</h2>
  <SetItem title="主题">
    {#snippet content()}
      <RadioGroup.Root
        class="flex gap-4 text-sm font-medium"
        bind:value={getTheme, setTheme}
      >
        {#each themeList as theme}
          <div
            class="text-foreground group flex items-center transition-all select-none"
          >
            <RadioGroup.Item
              id={theme.value}
              value={theme.value}
              class="border-border-input bg-background hover:border-dark-40 data-[state=checked]:border-foreground size-5 shrink-0 cursor-default cursor-pointer rounded-full border transition-all duration-100 ease-in-out data-[state=checked]:border-6"
            />
            <Label.Root for={theme.value} class="cursor-pointer pl-3">
              {theme.label}
            </Label.Root>
          </div>
        {/each}
      </RadioGroup.Root>
    {/snippet}
  </SetItem>

  <h2 class="mt-4 text-xl font-bold">系统设置</h2>
  <SetItem title="开机自启">
    {#snippet content()}
      <Switch.Root
        bind:checked={autostartEnabled}
        onCheckedChange={handleAutostartToggle}
        name="autoStart"
        class="focus-visible:ring-foreground focus-visible:ring-offset-background data-[state=checked]:bg-foreground data-[state=unchecked]:bg-dark-10 data-[state=unchecked]:shadow-mini-inset dark:data-[state=checked]:bg-foreground peer inline-flex h-[24px] min-h-[24px] w-[40px] shrink-0 cursor-pointer items-center rounded-full px-[3px] transition-colors focus-visible:ring-2 focus-visible:ring-offset-2 focus-visible:outline-hidden disabled:cursor-not-allowed disabled:opacity-50"
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
        class="focus-visible:ring-foreground focus-visible:ring-offset-background data-[state=checked]:bg-foreground data-[state=unchecked]:bg-dark-10 data-[state=unchecked]:shadow-mini-inset dark:data-[state=checked]:bg-foreground peer inline-flex h-[24px] min-h-[24px] w-[40px] shrink-0 cursor-pointer items-center rounded-full px-[3px] transition-colors focus-visible:ring-2 focus-visible:ring-offset-2 focus-visible:outline-hidden disabled:cursor-not-allowed disabled:opacity-50"
      >
        <Switch.Thumb
          class="bg-background data-[state=unchecked]:shadow-mini dark:border-background/30 dark:bg-foreground dark:shadow-popover pointer-events-none block size-[20px] shrink-0 rounded-full transition-transform data-[state=checked]:translate-x-[14px] data-[state=unchecked]:translate-x-0 dark:border dark:data-[state=unchecked]:border"
        />
      </Switch.Root>
    {/snippet}
  </SetItem>
  <SetItem title="显示/隐藏窗口快捷键">
    {#snippet content()}
      <ShortcutInput
        bind:value={shortcut}
        onSave={() => invoke("set_toggle_shortcut", { shortcutStr: shortcut })}
        showPresets={true}
      />
    {/snippet}
  </SetItem>
  <SetItem title="分离窗口快捷键">
    {#snippet content()}
      <ShortcutInput
        bind:value={$detachWindowShortcut}
        onSave={() => detachWindowShortcut.setShortcut($detachWindowShortcut)}
        showPresets={false}
      />
    {/snippet}
  </SetItem>

  <h2 class="mt-4 text-xl font-bold">数据存储</h2>
  <SetItem title="应用数据">
    {#snippet content()}
      <Button.Root
        class="rounded-input bg-dark text-background shadow-mini hover:bg-dark/95 inline-flex h-8 items-center justify-center px-[14px] text-[12px] font-semibold active:scale-[0.98] active:transition-all"
        onclick={() => invoke("open_app_data_dir")}
      >
        打开数据目录
      </Button.Root>
    {/snippet}
  </SetItem>
</main>
