/**
 * Plugin Icon Utility
 *
 * 生成插件图标的 URL
 */
import { invoke } from "@tauri-apps/api/core";

export interface PluginIconInfo {
  id: string;
  icon?: string;
  dir_name?: string;
}

/**
 * 生成插件图标的 URL
 *
 * @param plugin 插件信息
 * @returns 图标 URL 或 undefined
 */
export async function getPluginIconUrl(
  plugin: PluginIconInfo,
): Promise<string | undefined> {
  // 如果 manifest 中没有 icon，尝试查找默认图标文件
  if (!plugin.icon) {
    const dirName = plugin.dir_name || plugin.id;
    const iconNames = ["icon.svg", "icon.png", "icon.jpg", "icon.jpeg"];

    try {
      const port = await invoke<number>("get_plugin_server_port");

      // 尝试每个可能的图标文件
      for (const iconName of iconNames) {
        const testUrl = `http://127.0.0.1:${port}/plugin/${dirName}/${iconName}`;
        try {
          const response = await fetch(testUrl, { method: "HEAD" });
          if (response.ok) {
            return testUrl;
          }
        } catch {
          // 继续尝试下一个
        }
      }
    } catch (e) {
      console.error("Failed to get plugin server port:", e);
    }

    return undefined;
  }

  // 如果是完整 URL（marketplace 插件），直接返回
  if (plugin.icon.startsWith("http://") || plugin.icon.startsWith("https://")) {
    return plugin.icon;
  }

  // 如果是相对路径（本地插件），通过插件服务器访问
  try {
    const port = await invoke<number>("get_plugin_server_port");
    const dirName = plugin.dir_name || plugin.id;
    return `http://127.0.0.1:${port}/plugin/${dirName}/${plugin.icon}`;
  } catch (e) {
    console.error("Failed to get plugin server port:", e);
    return undefined;
  }
}
