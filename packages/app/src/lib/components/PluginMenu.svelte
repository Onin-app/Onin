<script lang="ts">
  /**
   * PluginMenu Component
   *
   * 插件操作下拉菜单组件
   * 包含分离窗口、关闭插件、自动分离开关
   * 使用 Tauri Native Menu 以防被内联原生 Webview 遮挡
   */
  import { DotsThreeVertical } from "phosphor-svelte";
  import {
    Menu,
    MenuItem,
    CheckMenuItem,
    PredefinedMenuItem,
  } from "@tauri-apps/api/menu";

  interface Props {
    autoDetach: boolean;
    terminateOnBg: boolean;
    runAtStartup: boolean;
    detachShortcut: string | null;
    onDetach: () => void;
    onClose: () => void;
    onToggleAutoDetach: (checked: boolean) => void;
    onToggleTerminateOnBg: (checked: boolean) => void;
    onToggleRunAtStartup: (checked: boolean) => void;
    onOpenDevTools: () => void;
    onUninstall: () => void;
  }

  let {
    autoDetach = $bindable(),
    terminateOnBg = $bindable(),
    runAtStartup = $bindable(),
    detachShortcut,
    onDetach,
    onClose,
    onToggleAutoDetach,
    onToggleTerminateOnBg,
    onToggleRunAtStartup,
    onOpenDevTools,
    onUninstall,
  }: Props = $props();

  async function handleMenu() {
    try {
      const detachItem = await MenuItem.new({
        text: `分离窗口 ${detachShortcut ? `(${detachShortcut})` : ""}`,
        action: () => onDetach(),
      });

      const closeItem = await MenuItem.new({
        text: "关闭插件 (ESC)",
        action: () => onClose(),
      });

      const autoDetachItem = await CheckMenuItem.new({
        text: "自动分离为独立窗口",
        checked: autoDetach,
        action: () => onToggleAutoDetach(!autoDetach),
      });

      const terminateOnBgItem = await CheckMenuItem.new({
        text: "退出到后台立即结束运行",
        checked: terminateOnBg,
        action: () => onToggleTerminateOnBg(!terminateOnBg),
      });

      const runAtStartupItem = await CheckMenuItem.new({
        text: "跟随主程序同时启动运行",
        checked: runAtStartup,
        action: () => onToggleRunAtStartup(!runAtStartup),
      });

      const separator1 = await PredefinedMenuItem.new({ item: "Separator" });
      const separator2 = await PredefinedMenuItem.new({ item: "Separator" });

      const devToolsItem = await MenuItem.new({
        text: "开发者工具",
        action: () => onOpenDevTools(),
      });

      const uninstallItem = await MenuItem.new({
        text: "卸载插件",
        action: () => onUninstall(),
      });

      const menu = await Menu.new({
        items: [
          detachItem,
          closeItem,
          separator1,
          autoDetachItem,
          terminateOnBgItem,
          runAtStartupItem,
          separator2,
          devToolsItem,
          uninstallItem,
        ],
      });

      await menu.popup();
    } catch (error) {
      console.error("Failed to show native menu:", error);
    }
  }
</script>

<button
  class="ml-2 flex h-8 w-8 flex-shrink-0 cursor-pointer items-center justify-center rounded-md transition-colors hover:bg-neutral-200 focus:outline-none dark:hover:bg-neutral-700"
  onclick={handleMenu}
  aria-label="插件选项"
>
  <DotsThreeVertical class="size-8" />
</button>
