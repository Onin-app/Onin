<script lang="ts">
  /**
   * PluginMenu Component
   *
   * 插件操作下拉菜单组件
   * 包含分离窗口、关闭插件、自动分离开关
   * 使用 Tauri Native Menu 以防被内联原生 Webview 遮挡
   */
  import { DotsThreeVertical } from "phosphor-svelte";
  import { Menu, MenuItem, CheckMenuItem } from "@tauri-apps/api/menu";

  interface Props {
    autoDetach: boolean;
    detachShortcut: string | null;
    onDetach: () => void;
    onClose: () => void;
    onToggleAutoDetach: (checked: boolean) => void;
  }

  let {
    autoDetach = $bindable(),
    detachShortcut,
    onDetach,
    onClose,
    onToggleAutoDetach,
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
        action: () => {
          onToggleAutoDetach(!autoDetach);
        },
      });

      const menu = await Menu.new({
        items: [detachItem, closeItem, autoDetachItem],
      });
      
      await menu.popup();
    } catch (error) {
      console.error("Failed to show native menu:", error);
    }
  }
</script>

<button
  class="ml-2 flex flex-shrink-0 cursor-pointer items-center justify-center h-8 w-8 rounded-md transition-colors hover:bg-neutral-200 dark:hover:bg-neutral-700 focus:outline-none"
  onclick={handleMenu}
  aria-label="插件选项"
>
  <DotsThreeVertical class="size-8" />
</button>
