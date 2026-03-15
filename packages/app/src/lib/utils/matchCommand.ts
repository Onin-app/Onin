import type { LaunchableItem, CommandMatch } from "$lib/type";
import {
  extensionsToMimes,
  getFileExtension,
  inferMimeType,
} from "./mimeTypeMap";

/**
 * 三层优雅降级模型 - 匹配逻辑
 *
 * 设计理念：
 * 1. 开发者只需配置扩展名（如 [".png", ".jpg"]）
 * 2. 系统自动将扩展名映射为 MIME 类型
 * 3. 运行时优先使用 MIME 类型判断，fallback 到扩展名
 *
 * 这个文件实现了运行层（判断层）的逻辑
 */

/**
 * 检查 MIME 类型是否匹配
 *
 * 运行层逻辑：
 * - 优先基于文件的 MIME 类型进行判断
 * - 支持通配符（如 "image/*"）
 * - 支持精确匹配（如 "image/png"）
 */
function matchMimeType(fileMimeType: string, patterns: string[]): boolean {
  if (!patterns || patterns.length === 0) return true;

  for (const pattern of patterns) {
    if (pattern === "*") return true;

    // 支持通配符，如 "image/*"
    if (pattern.endsWith("/*")) {
      const prefix = pattern.slice(0, -2);
      if (fileMimeType.startsWith(prefix + "/")) return true;
    } else if (fileMimeType === pattern) {
      return true;
    }
  }

  return false;
}

/**
 * 检查文件扩展名是否匹配
 *
 * Fallback 逻辑：
 * - 当 MIME 类型不可靠或不存在时使用
 * - 支持通配符（如 "*"）
 * - 大小写不敏感
 */
function matchExtension(fileName: string, patterns: string[]): boolean {
  if (!patterns || patterns.length === 0) return true;

  const fileExt = getFileExtension(fileName); // 提取文件扩展名

  for (const pattern of patterns) {
    if (pattern === "*") return true;

    const patternLower = pattern.toLowerCase();

    // 确保 pattern 以点号开头
    const normalizedPattern = patternLower.startsWith(".")
      ? patternLower
      : `.${patternLower}`;

    if (fileExt === normalizedPattern) return true;
  }

  return false;
}

/**
 * 检查文本是否匹配
 *
 * 逻辑说明：
 * - min/max 始终表示文本的字符数
 * - regexp 是额外的匹配条件
 * - 执行顺序：先检查 min/max（字符数），通过后再检查 regexp
 */
function matchText(text: string, match: CommandMatch): boolean {
  // 1. 首先检查字符数限制（min/max 优先）
  const textLength = text.length;
  if (match.min !== undefined && textLength < match.min) return false;
  if (match.max !== undefined && textLength > match.max) return false;

  // 2. 如果配置了正则表达式，检查是否匹配
  if (match.regexp) {
    try {
      const regex = new RegExp(match.regexp);
      if (!regex.test(text)) return false;
    } catch (e) {
      console.error("[Match] Invalid regexp:", match.regexp, e);
      return false;
    }
  }

  return true;
}

/**
 * 根据类型过滤文件
 */
function filterFilesByType(files: File[], type: string): File[] {
  return files.filter((file) => {
    if (type === "image") {
      return file.type.startsWith("image/");
    } else if (type === "file") {
      return (
        !file.type.startsWith("image/") &&
        file.type !== "application/x-directory"
      );
    } else if (type === "folder") {
      return file.type === "application/x-directory";
    }
    return false;
  });
}

/**
 * 检查文件是否匹配
 *
 * 三层降级模型的核心实现：
 *
 * 1. 加载层处理：
 *    - 开发者只需配置 extensions（如 [".png", ".jpg"]）
 *    - 系统自动将 extensions 转换为 MIME 类型
 *
 * 2. 运行层判断：
 *    - 优先使用文件的 MIME 类型进行判断（更准确）
 *    - 如果 MIME 类型不存在或不可靠，fallback 到扩展名判断
 *    - 双重保障，确保匹配的可靠性
 */
function matchFiles(files: File[], match: CommandMatch): boolean {
  // 根据类型过滤文件
  const filteredFiles = filterFilesByType(files, match.type);

  // 检查文件数量
  if (match.min !== undefined && filteredFiles.length < match.min) return false;
  if (match.max !== undefined && filteredFiles.length > match.max) return false;

  // 如果没有配置 extensions，匹配所有该类型的文件
  if (!match.extensions || match.extensions.length === 0) {
    return filteredFiles.length > 0;
  }

  // 加载层：自动将 extensions 转换为 MIME 类型
  const effectiveMimeTypes = extensionsToMimes(match.extensions);

  // 检查每个文件是否符合条件
  for (const file of filteredFiles) {
    if (match.type === "image" || match.type === "file") {
      let matched = false;

      // 运行层：优先使用 MIME 类型判断
      if (effectiveMimeTypes.length > 0) {
        // 如果文件有 MIME 类型，使用 MIME 类型判断
        if (file.type) {
          matched = matchMimeType(file.type, effectiveMimeTypes);
        } else {
          // 如果文件没有 MIME 类型，尝试从文件名推断
          const inferredMime = inferMimeType(file.name);
          matched = matchMimeType(inferredMime, effectiveMimeTypes);
        }

        // Fallback：如果 MIME 类型判断失败，使用扩展名判断
        if (!matched) {
          matched = matchExtension(file.name, match.extensions);
        }
      } else {
        // 如果无法转换为 MIME 类型，直接使用扩展名判断
        matched = matchExtension(file.name, match.extensions);
      }

      if (!matched) {
        return false;
      }
    }
  }

  return filteredFiles.length > 0;
}

/**
 * 检查命令是否匹配当前输入内容
 * @param item 命令项
 * @param attachedText 粘贴/附件文本
 * @param attachedFiles 附件文件
 * @param inputText 手动输入的文本（可选）
 */
export function checkCommandMatch(
  item: LaunchableItem,
  attachedText: string,
  attachedFiles: File[],
  inputText: string = "",
): boolean {
  if (!item.matches || item.matches.length === 0) return false;

  // 合并文本：优先使用附件文本，其次使用输入文本
  const effectiveText = attachedText || inputText;

  // 只要有一个匹配条件满足即可
  for (const match of item.matches) {
    let isMatch = false;

    if (match.type === "text" && effectiveText) {
      isMatch = matchText(effectiveText, match);
    } else if (
      match.type === "image" ||
      match.type === "file" ||
      match.type === "folder"
    ) {
      if (attachedFiles.length > 0) {
        isMatch = matchFiles(attachedFiles, match);
      }
    }

    if (isMatch) return true;
  }

  return false;
}

/**
 * 过滤出所有匹配的命令
 * @param items 所有命令列表
 * @param attachedText 粘贴/附件文本
 * @param attachedFiles 附件文件
 * @param inputText 手动输入的文本（可选）
 */
export function getMatchedCommands(
  items: LaunchableItem[],
  attachedText: string,
  attachedFiles: File[],
  inputText: string = "",
): LaunchableItem[] {
  // 如果没有任何内容，返回空数组
  if (!attachedText && attachedFiles.length === 0 && !inputText) return [];

  return items.filter((item) =>
    checkCommandMatch(item, attachedText, attachedFiles, inputText),
  );
}
