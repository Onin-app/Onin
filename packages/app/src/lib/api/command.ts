import { safeInvoke } from "./ipc";
import type { Command } from "$lib/type";

/**
 * 获取所有命令列表
 */
export function getCommands(): Promise<Command[]> {
  return safeInvoke("get_commands");
}

/**
 * 更新单个命令
 */
export function updateCommand(commandToUpdate: Command): Promise<void> {
  return safeInvoke("update_command", { commandToUpdate });
}

/**
 * 刷新命令列表
 */
export function refreshCommands(): Promise<void> {
  return safeInvoke("refresh_commands");
}

/**
 * 执行指定命令
 */
export function executeCommand(name: string, args?: unknown): Promise<void> {
  return safeInvoke("execute_command", { name, args });
}
