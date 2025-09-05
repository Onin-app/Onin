// 类型导出
export type {
  PluginEnvironment,
  ApiResponse,
  NotificationOptions,
  PluginSDKFunctions,
  ErrorCode
} from './types';

// 错误类导出
export {
  PluginSDKError,
  ValidationError,
  InvokeError,
  ERROR_CODES
} from './types';

// 核心功能导出
export {
  detectEnvironment,
  isHeadlessEnvironment,
  isUIEnvironment
} from './core/environment';

export {
  validateNotificationOptions,
  validatePipeline,
  createStringValidator
} from './core/validation';

// 适配器导出
export {
  createHeadlessAdapter
} from './adapters/headless';

export {
  createUIAdapter
} from './adapters/ui';

// 主 SDK 导出
export {
  createPluginSDK,
  sdk
} from './sdk';

// 常量导出
export {
  ENVIRONMENT_DETECTION,
  DEFAULT_CONFIG,
  MESSAGES
} from './utils/constants';