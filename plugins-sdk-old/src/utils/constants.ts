// 环境检测常量
export const ENVIRONMENT_DETECTION = {
  DENO_CORE_OPS: 'Deno.core.ops',
  TAURI_WINDOW: 'window.__TAURI__',
  OP_INVOKE: 'op_invoke'
} as const;

// 默认配置
export const DEFAULT_CONFIG = {
  TIMEOUT: 5000,
  RETRY_COUNT: 3
} as const;

// 消息常量
export const MESSAGES = {
  UNSUPPORTED_ENVIRONMENT: '不支持的环境：无法检测到 Deno 或 Tauri 运行时',
  VALIDATION_FAILED: '参数验证失败',
  INVOKE_FAILED: '调用失败',
  TITLE_REQUIRED: '标题是必需的',
  TITLE_MUST_BE_STRING: '标题必须是字符串',
  BODY_MUST_BE_STRING: '内容必须是字符串',
  ICON_MUST_BE_STRING: '图标必须是字符串'
} as const;