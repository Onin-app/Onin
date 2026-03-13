import { listen } from "@tauri-apps/api/event";

// 监听插件console输出
export function setupPluginConsoleListener() {
  listen("plugin_console_log", (event) => {
    const { message, timestamp } = event.payload as {
      message: string;
      timestamp: number;
    };
  });
}
