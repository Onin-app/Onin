<script lang="ts">
  import { onDestroy, onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { Label, RadioGroup, Switch, Button } from "bits-ui";
  import { toast } from "svelte-sonner";

  import { theme, toggleTheme } from "$lib/utils/theme";
  import { Theme, type SortMode, type AppConfig } from "$lib/type";
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
  let autoPasteTimeLimit = $state<number>(5);
  let autoClearTimeLimit = $state<number>(0);
  let sortMode = $state<SortMode>("smart");
  let enableUsageTracking = $state<boolean>(true);
  let marketplaceApiUrl = $state<string>("");

  const sortModeOptions: {
    value: SortMode;
    label: string;
    description: string;
  }[] = [
    {
      value: "smart",
      label: "智能排序",
      description: "综合使用频率和最近使用时间",
    },
    { value: "frequency", label: "频率优先", description: "按使用次数排序" },
    { value: "recent", label: "最近使用", description: "按最后使用时间排序" },
    { value: "default", label: "默认排序", description: "不使用频率数据" },
  ];

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
      toast.success(autostartEnabled ? "已启用开机自启" : "已禁用开机自启");
    } catch (error) {
      console.error("Failed to toggle autostart:", error);
      // 如果设置失败，将UI状态回滚
      autostartEnabled = !autostartEnabled;
      toast.error("设置开机自启失败");
    }
  };

  const handleTrayIconToggle = async () => {
    try {
      // `bind:checked` 会提前更新 trayIconEnabled 的值
      await invoke("set_tray_visibility", { visible: trayIconEnabled });
      // 从后端重新获取状态以确保UI同步
      trayIconEnabled = await invoke("is_tray_visible");
      toast.success(trayIconEnabled ? "已显示托盘图标" : "已隐藏托盘图标");
    } catch (error) {
      console.error("Failed to toggle tray icon visibility:", error);
      // 如果设置失败，将UI状态回滚
      trayIconEnabled = !trayIconEnabled;
      toast.error("设置托盘图标失败");
    }
  };

  const updateConfig = async () => {
    try {
      console.log("Updating config with:", {
        autoPasteTimeLimit,
        autoClearTimeLimit,
        sortMode,
        enableUsageTracking,
        marketplaceApiUrl,
      });
      await invoke("update_app_config", {
        config: {
          auto_paste_time_limit: autoPasteTimeLimit,
          auto_clear_time_limit: autoClearTimeLimit,
          sort_mode: sortMode,
          enable_usage_tracking: enableUsageTracking,
          marketplace_api_url: marketplaceApiUrl || undefined,
        },
      });
      console.log("Config updated successfully");
      toast.success("配置已保存");
    } catch (error) {
      console.error("Failed to update config:", error);
      toast.error("保存配置失败，请重试");
    }
  };

  const handleClearUsageStats = async () => {
    if (!confirm("确定要清除所有使用记录吗？此操作不可恢复。")) {
      return;
    }
    try {
      await invoke("clear_usage_stats");
      toast.success("使用记录已清除");
    } catch (error) {
      console.error("Failed to clear usage stats:", error);
      toast.error("清除失败：" + String(error));
    }
  };

  const unsubscribe = theme.subscribe((value) => {
    currentTheme = value;
  });

  onMount(async () => {
    try {
      autostartEnabled = await invoke("plugin:autostart|is_enabled");
    } catch (e) {
      console.error("Failed to get autostart state:", e);
      toast.error("获取开机自启状态失败");
    }

    try {
      shortcut = await invoke("get_toggle_shortcut");
    } catch (e) {
      console.error("Failed to get shortcut:", e);
      toast.error("获取快捷键配置失败");
    }

    try {
      trayIconEnabled = await invoke("is_tray_visible");
    } catch (e) {
      console.error("Failed to get tray visibility state:", e);
      toast.error("获取托盘图标状态失败");
    }

    try {
      const config = await invoke<AppConfig>("get_app_config");
      autoPasteTimeLimit = config.auto_paste_time_limit;
      autoClearTimeLimit = config.auto_clear_time_limit;
      sortMode = config.sort_mode;
      enableUsageTracking = config.enable_usage_tracking;
      marketplaceApiUrl = config.marketplace_api_url || "";
      console.log("Loaded config:", config);
    } catch (e) {
      console.error("Failed to get app config:", e);
      toast.error("加载应用配置失败，请重启应用");
    }
  });

  onDestroy(unsubscribe);
</script>

<main class="h-full w-full overflow-auto p-4">
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

  <h2 class="mt-4 text-xl font-bold">剪贴板设置</h2>
  <SetItem title="自动粘贴时间限制（秒）">
    {#snippet content()}
      <div class="flex items-center gap-2">
        <input
          type="number"
          min="0"
          max="60"
          bind:value={autoPasteTimeLimit}
          onchange={updateConfig}
          class="w-20 rounded border border-neutral-300 bg-white px-2 py-1 text-sm dark:border-neutral-600 dark:bg-neutral-700"
        />
        <span class="text-sm text-neutral-600 dark:text-neutral-400">
          {autoPasteTimeLimit === 0
            ? "不限制"
            : `${autoPasteTimeLimit}秒内复制的内容会自动粘贴`}
        </span>
      </div>
    {/snippet}
  </SetItem>
  <SetItem title="自动清空剪贴板时间限制（秒）">
    {#snippet content()}
      <div class="flex items-center gap-2">
        <input
          type="number"
          min="0"
          max="300"
          bind:value={autoClearTimeLimit}
          onchange={updateConfig}
          class="w-20 rounded border border-neutral-300 bg-white px-2 py-1 text-sm dark:border-neutral-600 dark:bg-neutral-700"
        />
        <span class="text-sm text-neutral-600 dark:text-neutral-400">
          {autoClearTimeLimit === 0
            ? "不自动清空"
            : `${autoClearTimeLimit}秒后自动清空剪贴板内容`}
        </span>
      </div>
    {/snippet}
  </SetItem>

  <h2 class="mt-4 text-xl font-bold">指令排序</h2>
  <SetItem title="启用使用频率追踪">
    {#snippet content()}
      <Switch.Root
        bind:checked={enableUsageTracking}
        onCheckedChange={updateConfig}
        name="enableUsageTracking"
        class="focus-visible:ring-foreground focus-visible:ring-offset-background data-[state=checked]:bg-foreground data-[state=unchecked]:bg-dark-10 data-[state=unchecked]:shadow-mini-inset dark:data-[state=checked]:bg-foreground peer inline-flex h-[24px] min-h-[24px] w-[40px] shrink-0 cursor-pointer items-center rounded-full px-[3px] transition-colors focus-visible:ring-2 focus-visible:ring-offset-2 focus-visible:outline-hidden disabled:cursor-not-allowed disabled:opacity-50"
      >
        <Switch.Thumb
          class="bg-background data-[state=unchecked]:shadow-mini dark:border-background/30 dark:bg-foreground dark:shadow-popover pointer-events-none block size-[20px] shrink-0 rounded-full transition-transform data-[state=checked]:translate-x-[14px] data-[state=unchecked]:translate-x-0 dark:border dark:data-[state=unchecked]:border"
        />
      </Switch.Root>
    {/snippet}
  </SetItem>
  <SetItem title="排序模式">
    {#snippet content()}
      <div class="flex flex-col gap-2">
        <select
          bind:value={sortMode}
          onchange={updateConfig}
          disabled={!enableUsageTracking}
          class="w-48 rounded border border-neutral-300 bg-white px-2 py-1 text-sm disabled:opacity-50 dark:border-neutral-600 dark:bg-neutral-700"
        >
          {#each sortModeOptions as option}
            <option value={option.value}>{option.label}</option>
          {/each}
        </select>
        <span class="text-xs text-neutral-600 dark:text-neutral-400">
          {sortModeOptions.find((o) => o.value === sortMode)?.description || ""}
        </span>
      </div>
    {/snippet}
  </SetItem>
  <SetItem title="使用记录">
    {#snippet content()}
      <Button.Root
        class="rounded-input bg-dark text-background shadow-mini hover:bg-dark/95 inline-flex h-8 items-center justify-center px-[14px] text-[12px] font-semibold active:scale-[0.98] active:transition-all"
        onclick={handleClearUsageStats}
      >
        清除使用记录
      </Button.Root>
    {/snippet}
  </SetItem>

  <h2 class="mt-4 text-xl font-bold">插件市场</h2>
  <SetItem title="API 地址">
    {#snippet content()}
      <div class="flex items-center gap-2">
        <input
          type="text"
          bind:value={marketplaceApiUrl}
          onchange={updateConfig}
          placeholder="https://..."
          class="flex-1 rounded border border-neutral-300 bg-white px-2 py-1 text-sm dark:border-neutral-600 dark:bg-neutral-700"
        />
        <span class="text-xs text-neutral-600 dark:text-neutral-400">
          插件市场 API 服务器地址
        </span>
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
