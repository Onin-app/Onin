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
    class="border-border/70 bg-background text-foreground relative flex h-full w-full overflow-hidden rounded-2xl border shadow-2xl ring-1 ring-white/10 dark:ring-white/5"
    data-tauri-drag-region
  >
    <TooltipProvider delayDuration={150}>
      <aside
        class="border-border/50 bg-muted/30 flex flex-col border-r pt-4 pb-3 transition-[width,padding] duration-180 ease-[cubic-bezier(0.32,0.72,0,1)] {isCollapsed
          ? 'w-16 px-2'
          : 'w-52 p-3'}"
        data-tauri-drag-region
      >
        <div
          class="mb-3 flex items-center {isCollapsed
            ? 'justify-center'
            : 'justify-between px-2'} h-8"
          data-tauri-drag-region
        >
          {#if !isCollapsed}
            <span
              class="text-muted-foreground text-xs font-semibold tracking-wider uppercase select-none"
              data-tauri-drag-region
            >
              设置
            </span>
          {/if}
          <Tooltip>
            <TooltipTrigger
              class="text-muted-foreground hover:bg-muted/80 hover:text-foreground flex h-7 w-7 cursor-pointer items-center justify-center rounded-lg transition-[transform,background-color,color] duration-140 ease-[cubic-bezier(0.23,1,0.32,1)] outline-none active:scale-95"
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
          {#each settings as setting (setting.id)}
            {@const Icon = setting.icon}
            {@const isActive = activeSetting.id === setting.id}
            {#if isCollapsed}
              <Tooltip>
                <TooltipTrigger
                  class="flex h-9 w-full cursor-pointer items-center justify-center rounded-xl transition-[transform,background-color,color,box-shadow] duration-140 ease-[cubic-bezier(0.23,1,0.32,1)] outline-none active:scale-[0.96] {isActive
                    ? 'bg-card text-foreground border-border/50 border font-medium shadow-xs'
                    : 'text-foreground/75 hover:bg-muted/60 hover:text-foreground'}"
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
                class="flex w-full cursor-pointer items-center gap-3 rounded-xl px-3 py-2 text-sm font-medium transition-[transform,background-color,color,box-shadow] duration-140 ease-[cubic-bezier(0.23,1,0.32,1)] outline-none active:scale-[0.97] {isActive
                  ? 'bg-card text-foreground border-border/50 border font-medium shadow-xs'
                  : 'text-foreground/75 hover:bg-muted/60 hover:text-foreground'}"
                onclick={() => handleClickSetting(setting)}
              >
                <Icon size={18} class="shrink-0" />
                <span class="truncate">{setting.name}</span>
              </Button.Root>
            {/if}
          {/each}
        </nav>

        <div class="border-border/40 mt-auto border-t pt-3">
          {#if isCollapsed}
            <Tooltip>
              <TooltipTrigger
                class="text-foreground/75 hover:bg-muted/60 hover:text-foreground flex h-9 w-full cursor-pointer items-center justify-center rounded-xl transition-[transform,background-color,color] duration-140 ease-[cubic-bezier(0.23,1,0.32,1)] outline-none active:scale-[0.96]"
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
              class="text-foreground/75 hover:bg-muted/60 hover:text-foreground flex w-full cursor-pointer items-center gap-3 rounded-xl px-3 py-2 text-sm font-medium transition-[transform,background-color,color] duration-140 ease-[cubic-bezier(0.23,1,0.32,1)] outline-none active:scale-[0.97]"
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
      class="flex-1 overflow-hidden bg-transparent p-6"
      data-tauri-drag-region
    >
      <div class="mx-auto flex h-full max-w-3xl flex-col">
        {#key activeSetting.id}
          <div
            class="h-full w-full animate-[tab-enter_140ms_cubic-bezier(0.23,1,0.32,1)_forwards]"
          >
            <ActiveComponent />
          </div>
        {/key}
      </div>
    </div>
  </main>
</div>

<style>
  @keyframes tab-enter {
    from {
      opacity: 0;
      transform: translateY(3px);
    }
    to {
      opacity: 1;
      transform: translateY(0);
    }
  }
</style>
