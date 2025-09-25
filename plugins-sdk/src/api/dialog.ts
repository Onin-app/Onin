import { invoke } from '../core/ipc';
import { dispatch } from '../core/dispatch';

// Dialog 错误类型
export interface DialogError extends Error {
  name: 'DialogError';
  code?: string;
}

export function createDialogError(message: string, code?: string): DialogError {
  const error = new Error(message) as DialogError;
  error.name = 'DialogError';
  error.code = code;
  return error;
}

export function isDialogError(error: any): error is DialogError {
  return error && error.name === 'DialogError';
}

// 消息对话框选项
export interface MessageDialogOptions {
  title?: string;
  message: string;
  kind?: 'info' | 'warning' | 'error';
  okLabel?: string;
}

// 确认对话框选项
export interface ConfirmDialogOptions {
  title?: string;
  message: string;
  kind?: 'info' | 'warning' | 'error';
  okLabel?: string;
  cancelLabel?: string;
}

// 文件对话框过滤器
export interface DialogFilter {
  name: string;
  extensions: string[];
}

// 打开文件对话框选项
export interface OpenDialogOptions {
  title?: string;
  defaultPath?: string;
  filters?: DialogFilter[];
  multiple?: boolean;
  directory?: boolean;
}

// 保存文件对话框选项
export interface SaveDialogOptions {
  title?: string;
  defaultPath?: string;
  filters?: DialogFilter[];
}

// 通用的对话框调用辅助函数
function callDialogApi<T = any>(method: string, args?: any): Promise<T> {
  return dispatch({
    webview: () => invoke<T>(method, args),
    headless: () => invoke<T>(method, args),
  });
}

/**
 * 显示消息对话框
 * @param options 消息对话框选项
 */
export function showMessage(options: MessageDialogOptions): Promise<void> {
  return callDialogApi("plugin_dialog_message", options);
}

/**
 * 显示确认对话框
 * @param options 确认对话框选项
 * @returns 用户是否点击了确认按钮
 */
export function showConfirm(options: ConfirmDialogOptions): Promise<boolean> {
  return callDialogApi<boolean>("plugin_dialog_confirm", options);
}

/**
 * 显示打开文件对话框
 * @param options 打开文件对话框选项
 * @returns 选择的文件路径，如果取消则返回 null
 */
export async function showOpen(options?: OpenDialogOptions): Promise<string | string[] | null> {
  const result = await callDialogApi<any>("plugin_dialog_open", options || {});
  
  // 处理返回值类型转换
  if (result === null || result === undefined) {
    return null;
  }
  
  // 如果是数组，说明是多文件选择
  if (Array.isArray(result)) {
    return result as string[];
  }
  
  // 如果是字符串，说明是单文件选择
  if (typeof result === 'string') {
    return result;
  }
  
  return null;
}

/**
 * 显示保存文件对话框
 * @param options 保存文件对话框选项
 * @returns 选择的保存路径，如果取消则返回 null
 */
export function showSave(options?: SaveDialogOptions): Promise<string | null> {
  return callDialogApi<string | null>("plugin_dialog_save", options || {});
}

// 便捷方法
/**
 * 显示信息消息
 * @param message 消息内容
 * @param title 标题（可选）
 */
export function info(message: string, title?: string): Promise<void> {
  return showMessage({
    message,
    title,
    kind: 'info'
  });
}

/**
 * 显示警告消息
 * @param message 消息内容
 * @param title 标题（可选）
 */
export function warning(message: string, title?: string): Promise<void> {
  return showMessage({
    message,
    title,
    kind: 'warning'
  });
}

/**
 * 显示错误消息
 * @param message 消息内容
 * @param title 标题（可选）
 */
export function error(message: string, title?: string): Promise<void> {
  return showMessage({
    message,
    title,
    kind: 'error'
  });
}

/**
 * 显示确认对话框（简化版）
 * @param message 消息内容
 * @param title 标题（可选）
 * @returns 用户是否点击了确认按钮
 */
export function confirm(message: string, title?: string): Promise<boolean> {
  return showConfirm({
    message,
    title
  });
}

/**
 * 选择单个文件
 * @param filters 文件过滤器（可选）
 * @param defaultPath 默认路径（可选）
 * @returns 选择的文件路径，如果取消则返回 null
 */
export function selectFile(filters?: DialogFilter[], defaultPath?: string): Promise<string | null> {
  return showOpen({
    filters,
    defaultPath,
    multiple: false,
    directory: false
  }) as Promise<string | null>;
}

/**
 * 选择多个文件
 * @param filters 文件过滤器（可选）
 * @param defaultPath 默认路径（可选）
 * @returns 选择的文件路径数组，如果取消则返回 null
 */
export function selectFiles(filters?: DialogFilter[], defaultPath?: string): Promise<string[] | null> {
  return showOpen({
    filters,
    defaultPath,
    multiple: true,
    directory: false
  }) as Promise<string[] | null>;
}

/**
 * 选择文件夹
 * @param defaultPath 默认路径（可选）
 * @returns 选择的文件夹路径，如果取消则返回 null
 */
export function selectFolder(defaultPath?: string): Promise<string | null> {
  return showOpen({
    defaultPath,
    multiple: false,
    directory: true
  }) as Promise<string | null>;
}

/**
 * 保存文件对话框（简化版）
 * @param defaultName 默认文件名（可选）
 * @param filters 文件过滤器（可选）
 * @returns 选择的保存路径，如果取消则返回 null
 */
export function saveFile(defaultName?: string, filters?: DialogFilter[]): Promise<string | null> {
  return showSave({
    defaultPath: defaultName,
    filters
  });
}

// 创建 Dialog API 命名空间
export const dialog = {
  // 核心方法
  showMessage,
  showConfirm,
  showOpen,
  showSave,
  
  // 便捷方法
  info,
  warning,
  error,
  confirm,
  selectFile,
  selectFiles,
  selectFolder,
  saveFile,
  
  // 错误处理工具
  createDialogError,
  isDialogError,
};