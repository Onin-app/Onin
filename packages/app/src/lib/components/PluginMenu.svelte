<script lang="ts">
  /**
   * PluginMenu.svelte
   *
   * 插件内联显示时的操作菜单 - 适配最新设计规范
   * 包含分离窗口、退出、开发者工具、设置等功能
   */
  import { onMount, onDestroy } from "svelte";
  import { Menu } from "@tauri-apps/api/menu";
  import { CheckMenuItem, MenuItem, PredefinedMenuItem } from "@tauri-apps/api/menu";

  interface Props {
    autoDetach: boolean;
    terminateOnBg: boolean;
    runAtStartup: boolean;
    detachShortcut?: string;
    onDetach: () => void;
    onClose: () => void;
    onToggleAutoDetach: (checked: boolean) => void;
    onToggleTerminateOnBg: (checked: boolean) => void;
    onToggleRunAtStartup: (checked: boolean) => void;
    onRefresh: () => void;
    onRestart?: () => void;
    onOpenDevTools: () => void;
    onUninstall: () => void;
  }

  let {
    autoDetach = $bindable(),
    terminateOnBg = $bindable(),
    runAtStartup = $bindable(),
    detachShortcut = "",
    onDetach,
    onClose,
    onToggleAutoDetach,
    onToggleTerminateOnBg,
    onToggleRunAtStartup,
    onRefresh,
    onRestart,
    onOpenDevTools,
    onUninstall,
  }: Props = $props();

  let nativeMenu: Menu | null = null;

  /**
   * 初始化并显示原生菜单
   */
  async function showNativeMenu() {
    if (nativeMenu) {
      await nativeMenu.popup();
      return;
    }

    const items = [
      await MenuItem.new({
        text: `分离为独立窗口${detachShortcut ? ` (${detachShortcut})` : ""}`,
        action: onDetach,
      }),
      await PredefinedMenuItem.new({ item: "Separator" }),
      await CheckMenuItem.new({
        text: "自动分离为独立窗口",
        checked: autoDetach,
        action: (id) => {
          autoDetach = !autoDetach;
          onToggleAutoDetach(autoDetach);
        },
      }),
      await CheckMenuItem.new({
        text: "退出到后台立即结束",
        checked: terminateOnBg,
        action: (id) => {
          terminateOnBg = !terminateOnBg;
          onToggleTerminateOnBg(terminateOnBg);
        },
      }),
      await CheckMenuItem.new({
        text: "跟随主程序同时启动",
        checked: runAtStartup,
        action: (id) => {
          runAtStartup = !runAtStartup;
          onToggleRunAtStartup(runAtStartup);
        },
      }),
      await PredefinedMenuItem.new({ item: "Separator" }),
      await MenuItem.new({
          text: "刷新界面",
          action: onRefresh,
      }),
      await MenuItem.new({
          text: "重启插件",
          action: () => {
              if (onRestart) onRestart();
          },
      }),
      await MenuItem.new({
        text: "开发者工具",
        action: onOpenDevTools,
      }),
      await MenuItem.new({
        text: "卸载当前插件",
        action: onUninstall,
      }),
      await PredefinedMenuItem.new({ item: "Separator" }),
      await MenuItem.new({
        text: "关闭插件",
        action: onClose,
      }),
    ];

    nativeMenu = await Menu.new({ items });
    await nativeMenu.popup();
  }

  // 监听并更新 CheckMenuItem 的选中状态（如果菜单已创建）
  $effect(() => {
    if (nativeMenu) {
      (async () => {
        const items = await nativeMenu?.items();
        if (items) {
          for (const item of items) {
            if (item instanceof CheckMenuItem) {
              const text = await item.text();
              if (text === "自动分离为独立窗口") {
                await item.setChecked(autoDetach);
              } else if (text === "退出到后台立即结束") {
                await item.setChecked(terminateOnBg);
              } else if (text === "跟随主程序同时启动") {
                await item.setChecked(runAtStartup);
              }
            }
          }
        }
      })();
    }
  });

  onDestroy(() => {
    if (nativeMenu) {
      nativeMenu.close();
    }
  });
</script>

<div class="flex items-center">
  <button
    class="flex h-10 w-10 items-center justify-center rounded-lg text-neutral-500 transition-all hover:bg-neutral-200 hover:text-neutral-900 active:scale-95 dark:text-neutral-400 dark:hover:bg-neutral-700 dark:hover:text-neutral-100"
    onclick={showNativeMenu}
    aria-label="插件菜单"
  >
    <svg
      xmlns="http://www.w3.org/2000/svg"
      width="20"
      height="20"
      fill="currentColor"
      viewBox="0 0 256 256"
    >
      <path
        d="M128,96a32,32,0,1,0,32,32A32,32,0,0,0,128,96Zm0,112a32,32,0,1,0,32,32A32,32,0,0,0,128,208ZM128,16a32,32,0,1,0,32,32A32,32,0,0,0,128,16Z"
      ></path>
    </svg>
  </button>
</div>
