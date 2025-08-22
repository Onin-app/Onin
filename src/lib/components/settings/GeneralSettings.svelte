<script lang="ts">
  import { onDestroy, onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { Label, RadioGroup, Switch, Button, Tooltip } from "bits-ui";

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
  let shortcut = $state<string>("");
  let isRecording = $state<boolean>(false);
  let previousShortcut = "";

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

  const handleShortcutInputKeydown = (e: KeyboardEvent) => {
    e.preventDefault();
    e.stopPropagation();

    const parts: string[] = [];
    if (e.ctrlKey) parts.push("Ctrl");
    if (e.altKey) parts.push("Alt");
    if (e.shiftKey) parts.push("Shift");
    if (e.metaKey) parts.push("Super");

    const key = e.key;

    // 如果只是修饰键，则更新UI以显示它们
    if (["Control", "Alt", "Shift", "Meta"].includes(key)) {
      shortcut = parts.join("+");
      return; // 等待一个非修饰键来完成快捷键
    }

    let finalKey = key;
    if (key === " ") {
      finalKey = "Space";
    } else if (key.length === 1 && /[a-zA-Z]/.test(key)) {
      finalKey = key.toUpperCase();
    }

    parts.push(finalKey);
    shortcut = parts.join("+");
  };

  const saveShortcut = async () => {
    const modifiers = ["Ctrl", "Alt", "Shift", "Super"];
    const parts = shortcut.split("+");
    const lastPart = parts[parts.length - 1];

    // 验证快捷键是否有效（非空且包含一个非修饰键）
    if (!shortcut || (parts.length > 0 && modifiers.includes(lastPart))) {
      // 无效快捷键，恢复到之前的值
      shortcut = previousShortcut;
      isRecording = false;
      return;
    }

    try {
      await invoke("set_toggle_shortcut", { shortcutStr: shortcut });
      isRecording = false;
    } catch (error) {
      console.error("Failed to set shortcut:", error);
      // 如果失败，可以重新获取旧的快捷键以回滚显示
      shortcut = previousShortcut;
      isRecording = false;
    }
  };

  const cancelShortcutEdit = () => {
    isRecording = false;
    // 恢复到修改前保存的快捷键
    shortcut = previousShortcut;
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

  const startRecording = () => {
    isRecording = true;
    previousShortcut = shortcut;
    // shortcut = "";
  };

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
  <SetItem title="快捷键">
    {#snippet content()}
      <div class="flex items-center gap-2">
        {#if isRecording}
          <div>
            <Tooltip.Provider>
              <Tooltip.Root
                delayDuration={200}
                bind:open={() => isRecording, () => {}}
              >
                <Tooltip.Trigger>
                  <input
                    type="text"
                    readonly
                    value={shortcut}
                    onkeydown={handleShortcutInputKeydown}
                    placeholder={shortcut}
                    class="bg-background text-foreground w-40 rounded border p-1"
                    autofocus
                  />
                </Tooltip.Trigger>
                <Tooltip.Content sideOffset={8}>
                  <div
                    class="rounded-input border-dark-10 bg-background shadow-popover z-0 border p-3 text-sm font-medium outline-hidden"
                  >
                    <p>
                      1. 先按功能键（Ctrl、Alt、Windows、Shift），再按其他普通键
                    </p>
                    <p>2. 按 F1 ~ F12 单键</p>
                  </div>
                </Tooltip.Content>
              </Tooltip.Root>
            </Tooltip.Provider>
          </div>
          <Button.Root
            class="rounded-input bg-dark text-background shadow-mini hover:bg-dark/95 inline-flex h-8 items-center justify-center px-[14px] text-[12px] font-semibold active:scale-[0.98] active:transition-all"
            onclick={saveShortcut}
          >
            保存
          </Button.Root>
          <Button.Root
            class="rounded-input bg-dark text-background shadow-mini hover:bg-dark/95 inline-flex h-8 items-center justify-center px-[14px] text-[12px] font-semibold active:scale-[0.98] active:transition-all"
            onclick={cancelShortcutEdit}
          >
            取消
          </Button.Root>
        {:else}
          <div
            class="bg-muted text-muted-foreground w-40 rounded border p-1 select-none"
          >
            {shortcut || "无"}
          </div>
          <Button.Root
            class="rounded-input bg-dark text-background shadow-mini hover:bg-dark/95 inline-flex h-8 items-center justify-center px-[14px] text-[12px] font-semibold active:scale-[0.98] active:transition-all"
            onclick={startRecording}
          >
            修改
          </Button.Root>
        {/if}
      </div>
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
