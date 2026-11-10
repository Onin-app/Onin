import { safeInvoke } from "./ipc";
import type { LaunchableItem } from "$lib/type";

/**
 * 获取所有文件快捷启动指令
 */
export function getFileCommands(): Promise<LaunchableItem[]> {
  return safeInvoke("get_file_commands");
}

/**
 * 添加文件或文件夹到快捷启动指令
 */
export function addFileCommands(paths: string[]): Promise<LaunchableItem[]> {
  return safeInvoke("add_file_commands", { paths });
}

/**
 * 从快捷启动指令中移除指定路径的项目
 */
export function removeFileCommand(path: string): Promise<LaunchableItem[]> {
  return safeInvoke("remove_file_command", { path });
}
