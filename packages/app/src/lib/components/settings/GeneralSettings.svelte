<script lang="ts">
  import { onDestroy, onMount } from "svelte";
  import { getVersion } from "@tauri-apps/api/app";
  import { invoke } from "@tauri-apps/api/core";
  import { Button } from "$lib/components/ui/button";
  import { Input } from "$lib/components/ui/input";
  import { Switch } from "$lib/components/ui/switch";
  import { Slider } from "$lib/components/ui/slider";
  import { Badge } from "$lib/components/ui/badge";
  import { Card } from "$lib/components/ui/card";
  import { Tabs, TabsList, TabsTrigger } from "$lib/components/ui/tabs";
  import { ScrollArea } from "$lib/components/ui/scroll-area";
  import { toast } from "svelte-sonner";

  import { theme, toggleTheme } from "$lib/utils/theme";
  import { windowOpacity, setWindowOpacity } from "$lib/stores/opacity";
  import { Theme, type SortMode, type AppConfig } from "$lib/type";
  import {
    detachWindowShortcut,
    toggleWindowShortcut,
  } from "$lib/stores/shortcuts";

  import ConfirmDialog from "$lib/components/ConfirmDialog.svelte";
  import SetItem from "./SetItem.svelte";
  import ShortcutInput from "./ShortcutInput.svelte";
  import {
    checkingUpdate,
    checkUpdate,
    hasNewVersion,
    latestVersion,
  } from "$lib/stores/update";

  const themeList: { value: Theme; label: string }[] = [
    { value: Theme.SYSTEM, label: "跟随系统" },
    { value: Theme.LIGHT, label: "明亮" },
    { value: Theme.DARK, label: "暗黑" },
  ];

  let currentTheme = $state<Theme>(Theme.DARK);
  let windowOpacityVal = $state<number>(100);
  let autostartEnabled = $state<boolean>(false);
  let trayIconEnabled = $state<boolean>(false);
  let autoCheckUpdate = $state<boolean>(true);
  let shortcut = $state<string>("");
  let autoPasteTimeLimit = $state<number>(5);
  let autoClearTimeLimit = $state<number>(0);
  let sortMode = $state<SortMode>("smart");
  let enableUsageTracking = $state<boolean>(true);
  let marketplaceApiUrl = $state<string>("");
  let disabledExtensionIds = $state<string[]>([]);
  let appVersion = $state<string>(import.meta.env.PACKAGE_VERSION || "未知");
  let clearUsageStatsDialogOpen = $state<boolean>(false);
  let clearingUsageStats = $state<boolean>(false);

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

  const setTheme = (value: Theme) => {
    currentTheme = value;
    toggleTheme(value);
  };

  const handleAutostartToggle = async () => {
    try {
      if (autostartEnabled) {
        await invoke("plugin:autostart|enable");
      } else {
        await invoke("plugin:autostart|disable");
      }
      autostartEnabled = await invoke("plugin:autostart|is_enabled");
      toast.success(autostartEnabled ? "已启用开机自启" : "已禁用开机自启");
    } catch (error) {
      console.error("Failed to toggle autostart:", error);
      autostartEnabled = !autostartEnabled;
      toast.error("设置开机自启失败");
    }
  };

  const handleTrayIconToggle = async () => {
    try {
      await invoke("set_tray_visibility", { visible: trayIconEnabled });
      trayIconEnabled = await invoke("is_tray_visible");
      toast.success(trayIconEnabled ? "已显示托盘图标" : "已隐藏托盘图标");
    } catch (error) {
      console.error("Failed to toggle tray icon visibility:", error);
      trayIconEnabled = !trayIconEnabled;
      toast.error("设置托盘图标失败");
    }
  };

  const handleOpacityChange = (value: number) => {
    windowOpacityVal = value;
    setWindowOpacity(value);
  };

  const updateConfig = async () => {
    try {
      await invoke("update_app_config", {
        config: {
          auto_paste_time_limit: autoPasteTimeLimit,
          auto_clear_time_limit: autoClearTimeLimit,
          sort_mode: sortMode,
          enable_usage_tracking: enableUsageTracking,
          marketplace_api_url: marketplaceApiUrl || undefined,
          disabled_extension_ids: disabledExtensionIds,
          auto_check_update: autoCheckUpdate,
          window_opacity: windowOpacityVal,
        },
      });
      toast.success("配置已保存");
    } catch (error) {
      console.error("Failed to update config:", error);
      toast.error("保存配置失败，请重试");
    }
  };

  const saveToggleShortcut = async () => {
    try {
      await toggleWindowShortcut.setShortcut(shortcut);
      toast.success("快捷键已保存");
    } catch (error) {
      console.error("Failed to set toggle shortcut:", error);
      toast.error("保存快捷键失败");
    }
  };

  const saveDetachWindowShortcut = async () => {
    try {
      await detachWindowShortcut.setShortcut($detachWindowShortcut);
      toast.success("快捷键已保存");
    } catch (error) {
      console.error("Failed to set detach window shortcut:", error);
      toast.error("保存快捷键失败");
    }
  };

  const handleClearUsageStats = async () => {
    clearUsageStatsDialogOpen = true;
  };

  const confirmClearUsageStats = async () => {
    if (clearingUsageStats) return;

    clearingUsageStats = true;
    try {
      await invoke("clear_usage_stats");
      clearUsageStatsDialogOpen = false;
      toast.success("使用记录已清除");
    } catch (error) {
      console.error("Failed to clear usage stats:", error);
      toast.error("清除失败：" + String(error));
    } finally {
      clearingUsageStats = false;
    }
  };

  const unsubscribeTheme = theme.subscribe((value) => {
    currentTheme = value;
  });

  const unsubscribeOpacity = windowOpacity.subscribe((value) => {
    windowOpacityVal = value;
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
      disabledExtensionIds = config.disabled_extension_ids || [];
      autoCheckUpdate = config.auto_check_update ?? true;
      if (config.window_opacity !== undefined) {
        windowOpacityVal = config.window_opacity;
        setWindowOpacity(config.window_opacity);
      }
    } catch (e) {
      console.error("Failed to get app config:", e);
      toast.error("加载应用配置失败，请重启应用");
    }

    try {
      appVersion = await getVersion();
    } catch (e) {
      console.error("Failed to get app version:", e);
    }
  });

  onDestroy(() => {
    unsubscribeTheme();
    unsubscribeOpacity();
  });
</script>

<ScrollArea class="h-full w-full" viewportClass="h-full w-full">
  <main class="h-full w-full pr-2 pb-8">
    <!-- 主题设置 -->
    <section class="mb-6">
      <h2
        class="text-muted-foreground mb-3 px-1 text-xs font-semibold tracking-wider uppercase"
      >
        主题设置
      </h2>
      <Card class="border-border/60 bg-card rounded-2xl px-4 py-0.5 shadow-2xs">
        <SetItem title="主题">
          {#snippet content()}
            <Tabs
              value={currentTheme}
              onValueChange={(v) => v && setTheme(v as Theme)}
            >
              <TabsList>
                {#each themeList as themeItem}
                  <TabsTrigger value={themeItem.value}>
                    {themeItem.label}
                  </TabsTrigger>
                {/each}
              </TabsList>
            </Tabs>
          {/snippet}
        </SetItem>
        <SetItem title="窗口透明度" description="调节主窗口及界面的不透明度">
          {#snippet content()}
            <div class="flex w-48 items-center gap-3">
              <Slider
                type="single"
                value={windowOpacityVal}
                min={30}
                max={100}
                step={1}
                onValueChange={(v) => handleOpacityChange(v as number)}
                onValueCommit={updateConfig}
              />
              <span
                class="text-muted-foreground w-12 shrink-0 text-right text-xs"
              >
                {windowOpacityVal}%
              </span>
            </div>
          {/snippet}
        </SetItem>
      </Card>
    </section>

    <!-- 系统设置 -->
    <section class="mb-6">
      <h2
        class="text-muted-foreground mb-3 px-1 text-xs font-semibold tracking-wider uppercase"
      >
        系统设置
      </h2>
      <Card class="border-border/60 bg-card rounded-2xl px-4 py-0.5 shadow-2xs">
        <SetItem title="开机自启">
          {#snippet content()}
            <Switch
              bind:checked={autostartEnabled}
              onCheckedChange={handleAutostartToggle}
            />
          {/snippet}
        </SetItem>
        <SetItem title="任务栏中显示图标">
          {#snippet content()}
            <Switch
              bind:checked={trayIconEnabled}
              onCheckedChange={handleTrayIconToggle}
            />
          {/snippet}
        </SetItem>
        <SetItem
          title="自动检查更新"
          description="启动应用时及后台自动检测最新版本"
        >
          {#snippet content()}
            <Switch
              bind:checked={autoCheckUpdate}
              onCheckedChange={updateConfig}
            />
          {/snippet}
        </SetItem>
        <SetItem title="显示/隐藏窗口快捷键">
          {#snippet content()}
            <ShortcutInput
              bind:value={shortcut}
              onSave={saveToggleShortcut}
              showPresets={true}
            />
          {/snippet}
        </SetItem>
        <SetItem title="分离窗口快捷键">
          {#snippet content()}
            <ShortcutInput
              bind:value={$detachWindowShortcut}
              onSave={saveDetachWindowShortcut}
              showPresets={false}
            />
          {/snippet}
        </SetItem>
      </Card>
    </section>

    <!-- 剪贴板设置 -->
    <section class="mb-6">
      <h2
        class="text-muted-foreground mb-3 px-1 text-xs font-semibold tracking-wider uppercase"
      >
        剪贴板设置
      </h2>
      <Card class="border-border/60 bg-card rounded-2xl px-4 py-0.5 shadow-2xs">
        <SetItem
          title="自动粘贴时间限制（秒）"
          description="复制内容后在此时间内自动粘贴"
        >
          {#snippet content()}
            <div class="flex w-48 items-center gap-3">
              <Slider
                type="single"
                value={autoPasteTimeLimit}
                min={0}
                max={60}
                step={1}
                onValueChange={(v) => (autoPasteTimeLimit = v as number)}
                onValueCommit={updateConfig}
              />
              <span
                class="text-muted-foreground w-16 shrink-0 text-right text-xs"
              >
                {autoPasteTimeLimit === 0
                  ? "不限制"
                  : `${autoPasteTimeLimit}秒`}
              </span>
            </div>
          {/snippet}
        </SetItem>
        <SetItem
          title="自动清空剪贴板时间限制（秒）"
          description="在此时间后自动清空剪贴板内容，保护隐私"
        >
          {#snippet content()}
            <div class="flex w-48 items-center gap-3">
              <Slider
                type="single"
                value={autoClearTimeLimit}
                min={0}
                max={300}
                step={5}
                onValueChange={(v) => (autoClearTimeLimit = v as number)}
                onValueCommit={updateConfig}
              />
              <span
                class="text-muted-foreground w-16 shrink-0 text-right text-xs"
              >
                {autoClearTimeLimit === 0
                  ? "不自动清空"
                  : `${autoClearTimeLimit}秒`}
              </span>
            </div>
          {/snippet}
        </SetItem>
      </Card>
    </section>

    <!-- 指令排序 -->
    <section class="mb-6">
      <h2
        class="text-muted-foreground mb-3 px-1 text-xs font-semibold tracking-wider uppercase"
      >
        指令排序
      </h2>
      <Card class="border-border/60 bg-card rounded-2xl px-4 py-0.5 shadow-2xs">
        <SetItem
          title="启用使用频率追踪"
          description="根据使用习惯优化指令排序"
        >
          {#snippet content()}
            <Switch
              bind:checked={enableUsageTracking}
              onCheckedChange={updateConfig}
            />
          {/snippet}
        </SetItem>
        <SetItem title="排序模式">
          {#snippet content()}
            <div class="flex flex-col gap-1 text-right">
              <select
                bind:value={sortMode}
                onchange={updateConfig}
                disabled={!enableUsageTracking}
                class="border-input bg-background h-8 rounded-md border px-2 py-1 text-sm disabled:cursor-not-allowed disabled:opacity-50"
              >
                {#each sortModeOptions as option}
                  <option value={option.value}>{option.label}</option>
                {/each}
              </select>
              <span class="text-muted-foreground text-[10px]">
                {sortModeOptions.find((o) => o.value === sortMode)
                  ?.description || ""}
              </span>
            </div>
          {/snippet}
        </SetItem>
        <SetItem title="使用记录">
          {#snippet content()}
            <Button variant="outline" size="sm" onclick={handleClearUsageStats}>
              清除使用记录
            </Button>
          {/snippet}
        </SetItem>
      </Card>
    </section>

    <!-- 插件市场 -->
    <section class="mb-6">
      <h2
        class="text-muted-foreground mb-3 px-1 text-xs font-semibold tracking-wider uppercase"
      >
        插件市场
      </h2>
      <Card class="border-border/60 bg-card rounded-2xl px-4 py-0.5 shadow-2xs">
        <SetItem title="API 地址">
          {#snippet content()}
            <Input
              type="text"
              bind:value={marketplaceApiUrl}
              onchange={updateConfig}
              placeholder="https://..."
              class="h-8 w-64 text-sm"
            />
          {/snippet}
        </SetItem>
      </Card>
    </section>

    <!-- 数据存储 -->
    <section class="mb-6">
      <h2
        class="text-muted-foreground mb-3 px-1 text-xs font-semibold tracking-wider uppercase"
      >
        数据存储
      </h2>
      <Card class="border-border/60 bg-card rounded-2xl px-4 py-0.5 shadow-2xs">
        <SetItem title="应用数据">
          {#snippet content()}
            <Button
              variant="outline"
              size="sm"
              onclick={() => invoke("open_app_data_dir")}
            >
              打开数据目录
            </Button>
          {/snippet}
        </SetItem>
      </Card>
    </section>

    <!-- 关于 -->
    <section class="mb-6">
      <h2
        class="text-muted-foreground mb-3 px-1 text-xs font-semibold tracking-wider uppercase"
      >
        关于
      </h2>
      <Card class="border-border/60 bg-card rounded-2xl px-4 py-0.5 shadow-2xs">
        <SetItem title="当前版本">
          {#snippet content()}
            <div class="flex items-center gap-3">
              <Badge variant="secondary" class="font-mono text-xs">
                v{appVersion}
              </Badge>
              {#if $hasNewVersion}
                <Badge
                  variant="default"
                  class="animate-pulse font-sans text-[10px]"
                >
                  新版可升: v{$latestVersion}
                </Badge>
              {/if}
              <Button
                variant={$hasNewVersion ? "default" : "outline"}
                size="sm"
                onclick={() => checkUpdate(false)}
                disabled={$checkingUpdate}
              >
                {$checkingUpdate
                  ? "检查中..."
                  : $hasNewVersion
                    ? "立即升级"
                    : "检查更新"}
              </Button>
            </div>
          {/snippet}
        </SetItem>
      </Card>
    </section>
  </main>
</ScrollArea>

<ConfirmDialog
  bind:open={clearUsageStatsDialogOpen}
  title="清除使用记录"
  description="清除后会重置指令使用频率和最近使用数据，此操作不可恢复。"
  confirmLabel="清除"
  cancelLabel="取消"
  loading={clearingUsageStats}
  closeOnConfirm={false}
  onConfirm={confirmClearUsageStats}
  onCancel={() => {
    if (!clearingUsageStats) {
      clearUsageStatsDialogOpen = false;
    }
  }}
/>
