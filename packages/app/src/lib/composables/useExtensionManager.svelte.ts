/**
 * Extension Manager Composable
 *
 * 管理 Extension 功能的 composable
 * - 实时获取 Extension 预览（如计算器结果）
 * - 执行 Extension 命令
 */

import { invoke } from "@tauri-apps/api/core";
import type { LaunchableItem } from "$lib/type";

/**
 * Extension 预览结果（来自后端）
 */
export interface ExtensionPreview {
  extension_id: string;
  command_code: string;
  title: string;
  description: string;
  icon: string;
  copyable: string;
  view_type?: "single" | "grid";
  grid_data?: EmojiGridData;
}

/**
 * Emoji Grid 数据
 */
export interface EmojiGridData {
  groups: EmojiGroup[];
}

/**
 * Emoji 分组
 */
export interface EmojiGroup {
  name: string;
  slug: string;
  emojis: EmojiItem[];
}

/**
 * 单个 Emoji 项
 */
export interface EmojiItem {
  emoji: string;
  name: string;
}

/**
 * 将 ExtensionPreview 转换为 LaunchableItem 用于显示
 */
function previewToLaunchableItem(preview: ExtensionPreview): LaunchableItem & {
  view_type?: "single" | "grid";
  grid_data?: EmojiGridData;
} {
  return {
    name: preview.title,
    description: preview.description,
    keywords: [],
    path: `extension:${preview.extension_id}:${preview.command_code}`,
    icon: preview.icon,
    icon_type: "Iconfont",
    item_type: "App",
    source: "Command",
    action: `extension:${preview.extension_id}`,
    source_display: "Extension",
    view_type: preview.view_type,
    grid_data: preview.grid_data,
  };
}

export function useExtensionManager() {
  let currentPreview = $state<ExtensionPreview | null>(null);
  let lastInput = $state<string>("");

  /**
   * 获取输入的 Extension 预览
   */
  async function getPreview(input: string): Promise<ExtensionPreview | null> {
    if (!input || input === lastInput) {
      return currentPreview;
    }

    lastInput = input;

    try {
      const preview = await invoke<ExtensionPreview | null>(
        "get_extension_preview",
        { input },
      );
      currentPreview = preview;
      return preview;
    } catch (error) {
      console.error("[Extension] Failed to get preview:", error);
      currentPreview = null;
      return null;
    }
  }

  /**
   * 获取 Extension 预览作为 LaunchableItem（用于 displayList）
   */
  function getPreviewAsItem(): LaunchableItem | null {
    if (!currentPreview) return null;
    return previewToLaunchableItem(currentPreview);
  }

  /**
   * 执行 Extension 命令
   */
  async function execute(
    extensionId: string,
    input: string,
  ): Promise<string | null> {
    try {
      const result = await invoke<{
        success: boolean;
        copyable?: string;
        error?: string;
      }>("execute_extension", { extensionId, input });

      if (result.success && result.copyable) {
        return result.copyable;
      }
      return null;
    } catch (error) {
      console.error("[Extension] Failed to execute:", error);
      return null;
    }
  }

  /**
   * 清除当前预览
   */
  function clearPreview() {
    currentPreview = null;
    lastInput = "";
  }

  return {
    get state() {
      return {
        currentPreview,
      };
    },
    getPreview,
    getPreviewAsItem,
    execute,
    clearPreview,
  };
}
