<script lang="ts">
  import { onDestroy, onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { Tabs, Switch, Button, ScrollArea } from "bits-ui";
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

<ScrollArea.Root class="h-full w-full" type="hover">
  <ScrollArea.Viewport class="h-full w-full">
    <main class="h-full w-full pr-2 pb-8">
      <section class="mb-6">
        <h2
          class="mb-3 px-1 text-xs font-semibold tracking-wider text-neutral-500 uppercase dark:text-neutral-400"
        >
          主题设置
        </h2>
        <div
          class="overflow-hidden rounded-xl border border-neutral-200 bg-white px-4 dark:border-neutral-800 dark:bg-neutral-900"
        >
          <SetItem title="主题">
            {#snippet content()}
              <Tabs.Root
                value={currentTheme}
                onValueChange={(v) => v && setTheme(v as Theme)}
              >
                <Tabs.List
                  class="flex gap-1 rounded-lg bg-neutral-100 p-1 dark:bg-neutral-800"
                >
                  {#each themeList as theme}
                    <Tabs.Trigger
                      value={theme.value}
                      class="rounded-md px-3 py-1.5 text-xs font-medium text-neutral-600 transition-all hover:bg-white/50 data-[state=active]:bg-white data-[state=active]:text-neutral-900 data-[state=active]:shadow-sm dark:text-neutral-400 dark:hover:bg-neutral-700/50 dark:data-[state=active]:bg-neutral-700 dark:data-[state=active]:text-white"
                    >
                      {theme.label}
                    </Tabs.Trigger>
                  {/each}
                </Tabs.List>
              </Tabs.Root>
            {/snippet}
          </SetItem>
        </div>
      </section>

      <section class="mb-6">
        <h2
          class="mb-3 px-1 text-xs font-semibold tracking-wider text-neutral-500 uppercase dark:text-neutral-400"
        >
          系统设置
        </h2>
        <div
          class="overflow-hidden rounded-xl border border-neutral-200 bg-white px-4 dark:border-neutral-800 dark:bg-neutral-900"
        >
          <SetItem title="开机自启">
            {#snippet content()}
              <Switch.Root
                bind:checked={autostartEnabled}
                onCheckedChange={handleAutostartToggle}
                class="peer inline-flex h-6 w-11 shrink-0 cursor-pointer items-center rounded-full border-2 border-transparent transition-colors focus-visible:ring-2 focus-visible:ring-neutral-950 focus-visible:ring-offset-2 focus-visible:outline-hidden disabled:cursor-not-allowed disabled:opacity-50 data-[state=checked]:bg-neutral-900 data-[state=unchecked]:bg-neutral-200 dark:focus-visible:ring-neutral-300 dark:data-[state=checked]:bg-neutral-50 dark:data-[state=unchecked]:bg-neutral-700"
              >
                <Switch.Thumb
                  class="pointer-events-none block h-5 w-5 rounded-full bg-white shadow-lg ring-0 transition-transform data-[state=checked]:translate-x-5 data-[state=unchecked]:translate-x-0 dark:bg-neutral-950"
                />
              </Switch.Root>
            {/snippet}
          </SetItem>
          <SetItem title="任务栏中显示图标">
            {#snippet content()}
              <Switch.Root
                bind:checked={trayIconEnabled}
                onCheckedChange={handleTrayIconToggle}
                class="peer inline-flex h-6 w-11 shrink-0 cursor-pointer items-center rounded-full border-2 border-transparent transition-colors focus-visible:ring-2 focus-visible:ring-neutral-950 focus-visible:ring-offset-2 focus-visible:outline-hidden disabled:cursor-not-allowed disabled:opacity-50 data-[state=checked]:bg-neutral-900 data-[state=unchecked]:bg-neutral-200 dark:focus-visible:ring-neutral-300 dark:data-[state=checked]:bg-neutral-50 dark:data-[state=unchecked]:bg-neutral-700"
              >
                <Switch.Thumb
                  class="pointer-events-none block h-5 w-5 rounded-full bg-white shadow-lg ring-0 transition-transform data-[state=checked]:translate-x-5 data-[state=unchecked]:translate-x-0 dark:bg-neutral-950"
                />
              </Switch.Root>
            {/snippet}
          </SetItem>
          <SetItem title="显示/隐藏窗口快捷键">
            {#snippet content()}
              <ShortcutInput
                bind:value={shortcut}
                onSave={() =>
                  invoke("set_toggle_shortcut", { shortcutStr: shortcut })}
                showPresets={true}
              />
            {/snippet}
          </SetItem>
          <SetItem title="分离窗口快捷键">
            {#snippet content()}
              <ShortcutInput
                bind:value={$detachWindowShortcut}
                onSave={() =>
                  detachWindowShortcut.setShortcut($detachWindowShortcut)}
                showPresets={false}
              />
            {/snippet}
          </SetItem>
        </div>
      </section>

      <section class="mb-6">
        <h2
          class="mb-3 px-1 text-xs font-semibold tracking-wider text-neutral-500 uppercase dark:text-neutral-400"
        >
          剪贴板设置
        </h2>
        <div
          class="overflow-hidden rounded-xl border border-neutral-200 bg-white px-4 dark:border-neutral-800 dark:bg-neutral-900"
        >
          <SetItem
            title="自动粘贴时间限制（秒）"
            description="复制内容后在此时间内自动粘贴"
          >
            {#snippet content()}
              <div class="flex items-center gap-2">
                <input
                  type="number"
                  min="0"
                  max="60"
                  bind:value={autoPasteTimeLimit}
                  onchange={updateConfig}
                  class="h-8 w-16 rounded-md border border-neutral-200 bg-transparent px-2 text-center text-sm focus:border-neutral-900 focus:outline-hidden dark:border-neutral-700 dark:focus:border-neutral-100"
                />
                <span class="text-xs text-neutral-500">
                  {autoPasteTimeLimit === 0 ? "不限制" : "秒"}
                </span>
              </div>
            {/snippet}
          </SetItem>
          <SetItem
            title="自动清空剪贴板时间限制（秒）"
            description="在此时间后自动清空剪贴板内容，保护隐私"
          >
            {#snippet content()}
              <div class="flex items-center gap-2">
                <input
                  type="number"
                  min="0"
                  max="300"
                  bind:value={autoClearTimeLimit}
                  onchange={updateConfig}
                  class="h-8 w-16 rounded-md border border-neutral-200 bg-transparent px-2 text-center text-sm focus:border-neutral-900 focus:outline-hidden dark:border-neutral-700 dark:focus:border-neutral-100"
                />
                <span class="text-xs text-neutral-500">
                  {autoClearTimeLimit === 0 ? "不自动清空" : "秒"}
                </span>
              </div>
            {/snippet}
          </SetItem>
        </div>
      </section>

      <section class="mb-6">
        <h2
          class="mb-3 px-1 text-xs font-semibold tracking-wider text-neutral-500 uppercase dark:text-neutral-400"
        >
          指令排序
        </h2>
        <div
          class="overflow-hidden rounded-xl border border-neutral-200 bg-white px-4 dark:border-neutral-800 dark:bg-neutral-900"
        >
          <SetItem
            title="启用使用频率追踪"
            description="根据使用习惯优化指令排序"
          >
            {#snippet content()}
              <Switch.Root
                bind:checked={enableUsageTracking}
                onCheckedChange={updateConfig}
                class="peer inline-flex h-6 w-11 shrink-0 cursor-pointer items-center rounded-full border-2 border-transparent transition-colors focus-visible:ring-2 focus-visible:ring-neutral-950 focus-visible:ring-offset-2 focus-visible:outline-hidden disabled:cursor-not-allowed disabled:opacity-50 data-[state=checked]:bg-neutral-900 data-[state=unchecked]:bg-neutral-200 dark:focus-visible:ring-neutral-300 dark:data-[state=checked]:bg-neutral-50 dark:data-[state=unchecked]:bg-neutral-700"
              >
                <Switch.Thumb
                  class="pointer-events-none block h-5 w-5 rounded-full bg-white shadow-lg ring-0 transition-transform data-[state=checked]:translate-x-5 data-[state=unchecked]:translate-x-0 dark:bg-neutral-950"
                />
              </Switch.Root>
            {/snippet}
          </SetItem>
          <SetItem title="排序模式">
            {#snippet content()}
              <div class="flex flex-col gap-1 text-right">
                <select
                  bind:value={sortMode}
                  onchange={updateConfig}
                  disabled={!enableUsageTracking}
                  class="h-8 rounded-md border border-neutral-200 bg-transparent px-2 py-1 text-sm disabled:cursor-not-allowed disabled:opacity-50 dark:border-neutral-700 dark:bg-neutral-900"
                >
                  {#each sortModeOptions as option}
                    <option value={option.value}>{option.label}</option>
                  {/each}
                </select>
                <span class="text-[10px] text-neutral-400">
                  {sortModeOptions.find((o) => o.value === sortMode)
                    ?.description || ""}
                </span>
              </div>
            {/snippet}
          </SetItem>
          <SetItem title="使用记录">
            {#snippet content()}
              <Button.Root
                class="inline-flex h-8 items-center justify-center rounded-md border border-neutral-200 bg-white px-3 text-xs font-semibold text-neutral-900 shadow-sm transition-colors hover:bg-neutral-100 hover:text-neutral-900 focus-visible:ring-1 focus-visible:ring-neutral-950 focus-visible:outline-hidden disabled:pointer-events-none disabled:opacity-50 dark:border-neutral-800 dark:bg-neutral-950 dark:text-neutral-50 dark:hover:bg-neutral-800 dark:hover:text-neutral-50 dark:focus-visible:ring-neutral-300"
                onclick={handleClearUsageStats}
              >
                清除使用记录
              </Button.Root>
            {/snippet}
          </SetItem>
        </div>
      </section>

      <section class="mb-6">
        <h2
          class="mb-3 px-1 text-xs font-semibold tracking-wider text-neutral-500 uppercase dark:text-neutral-400"
        >
          插件市场
        </h2>
        <div
          class="overflow-hidden rounded-xl border border-neutral-200 bg-white px-4 dark:border-neutral-800 dark:bg-neutral-900"
        >
          <SetItem title="API 地址">
            {#snippet content()}
              <input
                type="text"
                bind:value={marketplaceApiUrl}
                onchange={updateConfig}
                placeholder="https://..."
                class="h-8 w-64 rounded-md border border-neutral-200 bg-transparent px-3 text-sm placeholder:text-neutral-400 focus:border-neutral-900 focus:outline-hidden dark:border-neutral-700 dark:focus:border-neutral-100"
              />
            {/snippet}
          </SetItem>
        </div>
      </section>

      <section class="mb-6">
        <h2
          class="mb-3 px-1 text-xs font-semibold tracking-wider text-neutral-500 uppercase dark:text-neutral-400"
        >
          数据存储
        </h2>
        <div
          class="overflow-hidden rounded-xl border border-neutral-200 bg-white px-4 dark:border-neutral-800 dark:bg-neutral-900"
        >
          <SetItem title="应用数据">
            {#snippet content()}
              <Button.Root
                class="inline-flex h-8 items-center justify-center rounded-md border border-neutral-200 bg-white px-3 text-xs font-semibold text-neutral-900 shadow-sm transition-colors hover:bg-neutral-100 hover:text-neutral-900 focus-visible:ring-1 focus-visible:ring-neutral-950 focus-visible:outline-hidden disabled:pointer-events-none disabled:opacity-50 dark:border-neutral-800 dark:bg-neutral-950 dark:text-neutral-50 dark:hover:bg-neutral-800 dark:hover:text-neutral-50 dark:focus-visible:ring-neutral-300"
                onclick={() => invoke("open_app_data_dir")}
              >
                打开数据目录
              </Button.Root>
            {/snippet}
          </SetItem>
        </div>
      </section>
    </main>
  </ScrollArea.Viewport>
  <ScrollArea.Scrollbar
    orientation="vertical"
    class="bg-muted hover:bg-dark-10 data-[state=visible]:animate-in data-[state=hidden]:animate-out data-[state=hidden]:fade-out-0 data-[state=visible]:fade-in-0 flex w-1.5 touch-none rounded-full border-l border-l-transparent p-px transition-all duration-200 select-none hover:w-3"
  >
    <ScrollArea.Thumb class="bg-muted-foreground flex-1 rounded-full" />
  </ScrollArea.Scrollbar>
  <ScrollArea.Corner />
</ScrollArea.Root>
