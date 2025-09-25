import { invoke } from '../core/ipc';
import { dispatch } from '../core/dispatch';

// Clipboard 错误类型
export interface ClipboardError extends Error {
  name: 'ClipboardError';
  code?: string;
}

export function createClipboardError(message: string, code?: string): ClipboardError {
  const error = new Error(message) as ClipboardError;
  error.name = 'ClipboardError';
  error.code = code;
  return error;
}

export function isClipboardError(error: any): error is ClipboardError {
  return error && error.name === 'ClipboardError';
}

// 通用的剪贴板调用辅助函数
function callClipboardApi<T = any>(method: string, args?: any): Promise<T> {
  return dispatch({
    webview: () => invoke<T>(method, args),
    headless: () => invoke<T>(method, args),
  });
}

/**
 * 读取剪贴板中的文本内容
 * @returns 剪贴板中的文本，如果为空或不是文本则返回 null
 */
export function readText(): Promise<string | null> {
  return callClipboardApi<string | null>("plugin_clipboard_read_text");
}

/**
 * 将文本写入剪贴板
 * @param text 要写入的文本内容
 */
export function writeText(text: string): Promise<void> {
  return callClipboardApi("plugin_clipboard_write_text", { text });
}

/**
 * 读取剪贴板中的图像数据（Base64 格式）
 * @returns 图像的 Base64 数据，如果为空或不是图像则返回 null
 */
export function readImage(): Promise<string | null> {
  return callClipboardApi<string | null>("plugin_clipboard_read_image");
}

/**
 * 将图像写入剪贴板
 * @param imageData 图像的 Base64 数据或 Uint8Array
 */
export function writeImage(imageData: string | Uint8Array): Promise<void> {
  const data = typeof imageData === 'string' ? imageData : Array.from(imageData);
  return callClipboardApi("plugin_clipboard_write_image", { imageData: data });
}

/**
 * 清空剪贴板内容
 */
export function clear(): Promise<void> {
  return callClipboardApi("plugin_clipboard_clear");
}

/**
 * 检查剪贴板是否包含文本
 * @returns 如果剪贴板包含文本则返回 true
 */
export async function hasText(): Promise<boolean> {
  const text = await readText();
  return text !== null && text.length > 0;
}

/**
 * 检查剪贴板是否包含图像
 * @returns 如果剪贴板包含图像则返回 true
 */
export async function hasImage(): Promise<boolean> {
  const image = await readImage();
  return image !== null;
}

// 便捷方法
/**
 * 复制文本到剪贴板（writeText 的别名）
 * @param text 要复制的文本
 */
export function copy(text: string): Promise<void> {
  return writeText(text);
}

/**
 * 粘贴剪贴板中的文本（readText 的别名）
 * @returns 剪贴板中的文本
 */
export function paste(): Promise<string | null> {
  return readText();
}

// 创建 Clipboard API 命名空间
export const clipboard = {
  // 核心方法
  readText,
  writeText,
  readImage,
  writeImage,
  clear,
  
  // 检查方法
  hasText,
  hasImage,
  
  // 便捷方法
  copy,
  paste,
  
  // 错误处理工具
  createClipboardError,
  isClipboardError,
};