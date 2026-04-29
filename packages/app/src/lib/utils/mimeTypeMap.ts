/**
 * MIME 类型映射表
 *
 * 设计理念：
 * 为了提升开发者体验，我们采用三层优雅降级模型：
 *
 * 1️⃣ 开发者层（配置层）
 *    - 开发者只需要配置简单易记的扩展名（如 [".png", ".jpg"]）
 *    - 无需记忆复杂的 MIME 类型字符串
 *
 * 2️⃣ 系统加载层（解析层）
 *    - 系统自动将扩展名映射为标准 MIME 类型
 *    - 如果配置中同时存在 extensions 和 mimeTypes，以 mimeTypes 为准
 *    - 这样既保持了灵活性，又提供了便利性
 *
 * 3️⃣ 运行层（判断层）
 *    - 优先基于文件的 MIME 类型进行判断（更准确）
 *    - 如果文件没有 MIME 类型或 MIME 类型不可靠，fallback 到扩展名判断
 *    - 双重保障，确保匹配的可靠性
 *
 * 维护说明：
 * - 这个映射表包含了最常见的文件类型
 * - 如需添加新类型，请保持格式一致
 * - 扩展名统一使用小写，带点号（如 ".png"）
 */

/**
 * 扩展名到 MIME 类型的映射表
 */
export const EXTENSION_TO_MIME: Record<string, string> = {
  // 图片类型
  ".png": "image/png",
  ".jpg": "image/jpeg",
  ".jpeg": "image/jpeg",
  ".gif": "image/gif",
  ".webp": "image/webp",
  ".svg": "image/svg+xml",
  ".bmp": "image/bmp",
  ".ico": "image/x-icon",
  ".tiff": "image/tiff",
  ".tif": "image/tiff",

  // 视频类型
  ".mp4": "video/mp4",
  ".webm": "video/webm",
  ".avi": "video/x-msvideo",
  ".mov": "video/quicktime",
  ".wmv": "video/x-ms-wmv",
  ".flv": "video/x-flv",
  ".mkv": "video/x-matroska",

  // 音频类型
  ".mp3": "audio/mpeg",
  ".wav": "audio/wav",
  ".ogg": "audio/ogg",
  ".oga": "audio/ogg",
  ".m4a": "audio/mp4",
  ".flac": "audio/flac",
  ".aac": "audio/aac",

  // 文档类型
  ".pdf": "application/pdf",
  ".doc": "application/msword",
  ".docx":
    "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
  ".xls": "application/vnd.ms-excel",
  ".xlsx": "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
  ".ppt": "application/vnd.ms-powerpoint",
  ".pptx":
    "application/vnd.openxmlformats-officedocument.presentationml.presentation",
  ".odt": "application/vnd.oasis.opendocument.text",
  ".ods": "application/vnd.oasis.opendocument.spreadsheet",
  ".odp": "application/vnd.oasis.opendocument.presentation",

  // 压缩包类型
  ".zip": "application/zip",
  ".rar": "application/x-rar-compressed",
  ".7z": "application/x-7z-compressed",
  ".tar": "application/x-tar",
  ".gz": "application/gzip",
  ".bz2": "application/x-bzip2",

  // 代码文件
  ".js": "text/javascript",
  ".mjs": "text/javascript",
  ".ts": "text/typescript",
  ".tsx": "text/typescript",
  ".jsx": "text/javascript",
  ".json": "application/json",
  ".html": "text/html",
  ".htm": "text/html",
  ".css": "text/css",
  ".xml": "text/xml",
  ".py": "text/x-python",
  ".java": "text/x-java",
  ".cpp": "text/x-c++src",
  ".c": "text/x-csrc",
  ".h": "text/x-chdr",
  ".rs": "text/x-rust",
  ".go": "text/x-go",
  ".php": "text/x-php",
  ".rb": "text/x-ruby",
  ".vue": "text/x-vue",
  ".svelte": "text/x-svelte",
  ".sh": "text/x-shellscript",
  ".ps1": "text/x-powershell",
  ".sql": "text/x-sql",

  // 文本文件
  ".txt": "text/plain",
  ".md": "text/markdown",
  ".markdown": "text/markdown",
  ".csv": "text/csv",
  ".log": "text/plain",
  ".yaml": "text/yaml",
  ".yml": "text/yaml",
  ".toml": "text/toml",
  ".ini": "text/plain",
  ".conf": "text/plain",

  // 字体文件
  ".ttf": "font/ttf",
  ".otf": "font/otf",
  ".woff": "font/woff",
  ".woff2": "font/woff2",

  // 其他常见类型
  ".exe": "application/x-msdownload",
  ".dmg": "application/x-apple-diskimage",
  ".apk": "application/vnd.android.package-archive",
};

/**
 * 将扩展名转换为 MIME 类型
 *
 * @param extension 文件扩展名（如 ".png" 或 "png"）
 * @returns MIME 类型字符串，如果未找到则返回 null
 */
export function extensionToMime(extension: string): string | null {
  // 输入验证
  if (!extension || typeof extension !== "string") {
    return null;
  }

  // 确保扩展名以点号开头且为小写
  const normalizedExt = extension.startsWith(".")
    ? extension.toLowerCase()
    : `.${extension.toLowerCase()}`;

  return EXTENSION_TO_MIME[normalizedExt] || null;
}

/**
 * 将扩展名数组转换为 MIME 类型数组
 *
 * @param extensions 扩展名数组
 * @returns MIME 类型数组（过滤掉未找到的扩展名）
 */
export function extensionsToMimes(extensions: string[]): string[] {
  return extensions
    .map((ext) => extensionToMime(ext))
    .filter((mime): mime is string => mime !== null);
}

/**
 * 从文件名提取扩展名
 *
 * @param fileName 文件名
 * @returns 扩展名（带点号，小写），如果没有扩展名则返回空字符串
 */
export function getFileExtension(fileName: string): string {
  const lastDot = fileName.lastIndexOf(".");

  // 没有点号，或点号在最后，或点号在开头（隐藏文件如 .gitignore）
  if (lastDot === -1 || lastDot === fileName.length - 1 || lastDot === 0) {
    return "";
  }

  return fileName.substring(lastDot).toLowerCase();
}

/**
 * 根据文件名推断 MIME 类型
 *
 * @param fileName 文件名
 * @returns MIME 类型，如果无法推断则返回 'application/octet-stream'
 */
export function inferMimeType(fileName: string): string {
  const ext = getFileExtension(fileName);
  return extensionToMime(ext) || "application/octet-stream";
}
