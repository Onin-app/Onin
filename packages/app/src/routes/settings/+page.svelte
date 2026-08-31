<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { get } from "svelte/store";
  import { Button } from "bits-ui";

  import { goto } from "$app/navigation";
  import {
    Gear,
    RocketLaunch,
    TerminalWindow,
    Keyboard,
    PlugsConnected,
    Sparkle,
    PuzzlePiece,
    Cloud,
    Database,
    SidebarSimple,
  } from "phosphor-svelte";
  import {
    Tooltip,
    TooltipContent,
    TooltipProvider,
    TooltipTrigger,
  } from "$lib/components/ui/tooltip";

  import GeneralSettings from "$lib/components/settings/GeneralSettings.svelte";
  import FileCommandSettings from "$lib/components/settings/FileCommandSettings.svelte";
  import CommandSettings from "$lib/components/settings/CommandSettings.svelte";
  import ShortcutSettings from "$lib/components/settings/ShortcutSettings.svelte";
  import AISettings from "$lib/components/settings/AISettings.svelte";
  import ExtensionSettings from "$lib/components/settings/ExtensionSettings.svelte";
  import SyncSettings from "$lib/components/settings/SyncSettings.svelte";
  import MyDataSettings from "$lib/components/settings/MyDataSettings.svelte";
  import { escapeHandler } from "$lib/stores/escapeHandler";

  interface SettingItem {
    name: string;
    id: string;
    component:
      | typeof GeneralSettings
      | typeof FileCommandSettings
      | typeof CommandSettings
      | typeof ShortcutSettings
      | typeof AISettings
      | typeof ExtensionSettings
      | typeof SyncSettings
      | typeof MyDataSettings;
    icon: any;
  }

  const settings: SettingItem[] = [
    {
      name: "通用设置",
      id: "general",
      component: GeneralSettings,
      icon: Gear,
    },
    {
      name: "文件启动",
      id: "startup",
      component: FileCommandSettings,
      icon: RocketLaunch,
    },
    {
      name: "指令设置",
      id: "commands",
      component: CommandSettings,
      icon: TerminalWindow,
    },
    {
      name: "扩展",
      id: "extensions",
      component: ExtensionSettings,
      icon: PuzzlePiece,
    },
    {
      name: "全局快捷键",
      id: "shortcuts",
      component: ShortcutSettings,
      icon: Keyboard,
    },
    {
      name: "AI",
      id: "ai",
      component: AISettings,
      icon: Sparkle,
    },
    {
      name: "数据同步",
      id: "sync",
      component: SyncSettings,
      icon: Cloud,
    },
    {
      name: "我的数据",
      id: "my_data",
      component: MyDataSettings,
      icon: Database,
    },
  ];

  let activeSetting = $state<SettingItem>(settings[0]);
  let ActiveComponent = $derived(activeSetting.component);
  let isCollapsed = $state(false);

  const handleEsc = () => {
    goto("/");
  };

  const toggleCollapse = () => {
    isCollapsed = !isCollapsed;
    try {
      localStorage.setItem("settings_sidebar_collapsed", String(isCollapsed));
    } catch {
      // ignore
    }
  };

  onMount(() => {
    // Register this page's ESC handler
    escapeHandler.set(handleEsc);

    try {
      const savedCollapsed = localStorage.getItem("settings_sidebar_collapsed");
      if (savedCollapsed !== null) {
        isCollapsed = savedCollapsed === "true";
      }
    } catch {
      // ignore
    }

    // 解析 query 参数，自动激活对应的设置面板
    const urlParams = new URLSearchParams(window.location.search);
    const tab = urlParams.get("tab");
    if (tab) {
      const matched = settings.find((s) => s.id === tab);
      if (matched) {
        activeSetting = matched;
      }
    }
  });

  onDestroy(() => {
    // On destroy, reset the handler if it's still ours
    if (get(escapeHandler) === handleEsc) {
      escapeHandler.set(null);
    }
  });

  const handleClickSetting = (setting: SettingItem) => {
    activeSetting = setting;
  };
</script>

<div class="h-screen w-full bg-transparent p-1">
  <main
    class="relative flex h-full w-full overflow-hidden rounded-xl bg-neutral-50 text-neutral-900 selection:bg-neutral-200 dark:bg-neutral-900 dark:text-neutral-100 dark:selection:bg-neutral-700"
    data-tauri-drag-region
  >
    <TooltipProvider delayDuration={150}>
      <aside
        class="flex flex-col border-r border-neutral-200 bg-neutral-100/50 pt-4 pb-3 transition-all duration-200 ease-in-out dark:border-neutral-800 dark:bg-neutral-900/50 {isCollapsed
          ? 'w-16 px-2'
          : 'w-52 p-3'}"
        data-tauri-drag-region
      >
        <div
          class="mb-4 flex items-center {isCollapsed
            ? 'justify-center'
            : 'justify-between px-2'} h-8"
          data-tauri-drag-region
        >
          {#if !isCollapsed}
            <span
              class="text-sm font-semibold text-neutral-500 select-none"
              data-tauri-drag-region
            >
              设置
            </span>
          {/if}
          <Tooltip>
            <TooltipTrigger
              class="flex h-7 w-7 items-center justify-center rounded-md text-neutral-500 transition-colors hover:bg-neutral-200/60 hover:text-neutral-900 dark:text-neutral-400 dark:hover:bg-neutral-800 dark:hover:text-neutral-100"
              onclick={toggleCollapse}
              aria-label={isCollapsed ? "展开侧边栏" : "折叠侧边栏"}
            >
              <SidebarSimple size={18} />
            </TooltipTrigger>
            <TooltipContent
              side={isCollapsed ? "right" : "bottom"}
              sideOffset={6}
            >
              {isCollapsed ? "展开侧边栏" : "折叠侧边栏"}
            </TooltipContent>
          </Tooltip>
        </div>

        <nav
          class="flex flex-1 flex-col gap-1 overflow-x-hidden overflow-y-auto"
        >
          {#each settings as setting}
            {@const Icon = setting.icon}
            {#if isCollapsed}
              <Tooltip>
                <TooltipTrigger
                  class="flex h-9 w-full items-center justify-center rounded-lg transition-colors {activeSetting.id ===
                  setting.id
                    ? 'bg-white text-neutral-900 shadow-sm dark:bg-neutral-800 dark:text-white'
                    : 'text-neutral-600 hover:bg-neutral-200/50 hover:text-neutral-900 dark:text-neutral-400 dark:hover:bg-neutral-800/50 dark:hover:text-white'}"
                  onclick={() => handleClickSetting(setting)}
                  aria-label={setting.name}
                >
                  <Icon size={18} />
                </TooltipTrigger>
                <TooltipContent side="right" sideOffset={10}>
                  {setting.name}
                </TooltipContent>
              </Tooltip>
            {:else}
              <Button.Root
                class="flex w-full items-center gap-3 rounded-lg px-3 py-2 text-sm font-medium transition-colors {activeSetting.id ===
                setting.id
                  ? 'bg-white text-neutral-900 shadow-sm dark:bg-neutral-800 dark:text-white'
                  : 'text-neutral-600 hover:bg-neutral-200/50 hover:text-neutral-900 dark:text-neutral-400 dark:hover:bg-neutral-800/50 dark:hover:text-white'}"
                onclick={() => handleClickSetting(setting)}
              >
                <Icon size={18} class="shrink-0" />
                <span class="truncate">{setting.name}</span>
              </Button.Root>
            {/if}
          {/each}
        </nav>

        <div
          class="mt-auto border-t border-neutral-200 pt-3 dark:border-neutral-800"
        >
          {#if isCollapsed}
            <Tooltip>
              <TooltipTrigger
                class="flex h-9 w-full items-center justify-center rounded-lg text-neutral-600 transition-colors hover:bg-neutral-200/50 hover:text-neutral-900 dark:text-neutral-400 dark:hover:bg-neutral-800/50 dark:hover:text-white"
                onclick={() => goto("/plugins")}
                aria-label="插件管理"
              >
                <PlugsConnected size={18} />
              </TooltipTrigger>
              <TooltipContent side="right" sideOffset={10}>
                插件管理
              </TooltipContent>
            </Tooltip>
          {:else}
            <Button.Root
              class="flex w-full items-center gap-3 rounded-lg px-3 py-2 text-sm font-medium text-neutral-600 transition-colors hover:bg-neutral-200/50 hover:text-neutral-900 dark:text-neutral-400 dark:hover:bg-neutral-800/50 dark:hover:text-white"
              onclick={() => goto("/plugins")}
            >
              <PlugsConnected size={18} class="shrink-0" />
              <span class="truncate">插件管理</span>
            </Button.Root>
          {/if}
        </div>
      </aside>
    </TooltipProvider>

    <div
      class="flex-1 overflow-hidden bg-white p-6 dark:bg-neutral-950"
      data-tauri-drag-region
    >
      <div class="mx-auto flex h-full max-w-3xl flex-col">
        <ActiveComponent />
      </div>
    </div>
  </main>
</div>
