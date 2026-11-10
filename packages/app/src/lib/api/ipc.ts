import { invoke } from "@tauri-apps/api/core";
import type {
  AppConfig,
  Command,
  LaunchableItem,
  Shortcut,
  CommandUsageStats,
} from "$lib/type";

/**
 * Tauri 后端命令与入参/返回值的类型契约映射表
 */
export interface TauriCommandMap {
  // 文件指令管理
  get_file_commands: { args?: void; return: LaunchableItem[] };
  add_file_commands: { args: { paths: string[] }; return: LaunchableItem[] };
  remove_file_command: { args: { path: string }; return: LaunchableItem[] };

  // 统一启动管理
  get_all_launchable_items: { args?: void; return: LaunchableItem[] };

  // 应用配置管理
  get_app_config: { args?: void; return: AppConfig };
  update_app_config: { args: { config: AppConfig }; return: void };

  // 快捷键管理
  get_shortcuts: { args?: void; return: Shortcut[] };
  add_shortcut: {
    args: {
      shortcut: Shortcut;
      oldShortcutStr?: string | null;
    };
    return: void;
  };
  remove_shortcut: { args: { shortcutStr: string }; return: void };
  get_toggle_shortcut: { args?: void; return: string };
  set_toggle_shortcut: { args: { shortcutStr: string }; return: void };
  get_detach_window_shortcut: { args?: void; return: string };
  set_detach_window_shortcut: { args: { shortcutStr: string }; return: void };
  set_shortcut_recording: { args: { recording: boolean }; return: void };

  // 指令管理
  get_commands: { args?: void; return: Command[] };
  update_command: { args: { command: Command }; return: void };
  refresh_commands: { args?: void; return: Command[] };

  // 使用统计
  get_usage_stats: { args?: void; return: CommandUsageStats[] };
  record_command_usage: {
    args: { name: string; source: string };
    return: void;
  };
  clear_usage_stats: { args?: void; return: void };

  // 窗口管理
  acquire_window_close_lock: { args?: void; return: void };
  release_window_close_lock: { args?: void; return: void };
  close_main_window: { args?: void; return: void };
}

export type TauriCommandName = keyof TauriCommandMap;

/**
 * 类型安全的 invoke 包装函数
 * 强制校验 command 名称必须合法，且参数与返回值严格匹配。
 * 如果拼错命令名或传递错误参数，TypeScript 编译期将直接报错。
 */
export async function safeInvoke<K extends TauriCommandName>(
  cmd: K,
  ...args: TauriCommandMap[K]["args"] extends void | undefined
    ? [args?: undefined]
    : [args: TauriCommandMap[K]["args"]]
): Promise<TauriCommandMap[K]["return"]> {
  const payload = args[0];
  if (payload !== undefined) {
    return await invoke<TauriCommandMap[K]["return"]>(
      cmd,
      payload as Record<string, unknown>,
    );
  }
  return await invoke<TauriCommandMap[K]["return"]>(cmd);
}
