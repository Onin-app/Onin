import type { NotificationOptions } from '../types';
import { ValidationError } from '../types';
import { MESSAGES } from '../utils/constants';

/**
 * 验证通知选项
 * @param options 通知选项
 * @throws {ValidationError} 当验证失败时
 */
export function validateNotificationOptions(options: unknown): asserts options is NotificationOptions {
  if (!options || typeof options !== 'object') {
    throw new ValidationError(MESSAGES.VALIDATION_FAILED);
  }

  const opts = options as Record<string, unknown>;

  // 验证 title
  if (!opts.title) {
    throw new ValidationError(MESSAGES.TITLE_REQUIRED, 'title');
  }

  if (typeof opts.title !== 'string') {
    throw new ValidationError(MESSAGES.TITLE_MUST_BE_STRING, 'title');
  }

  // 验证 body（可选）
  if (opts.body !== undefined && typeof opts.body !== 'string') {
    throw new ValidationError(MESSAGES.BODY_MUST_BE_STRING, 'body');
  }

  // 验证 icon（可选）
  if (opts.icon !== undefined && typeof opts.icon !== 'string') {
    throw new ValidationError(MESSAGES.ICON_MUST_BE_STRING, 'icon');
  }
}

/**
 * 通用验证管道
 * @param value 要验证的值
 * @param validators 验证器数组
 * @returns 验证后的值
 */
export function validatePipeline<T>(
  value: unknown,
  validators: Array<(value: unknown) => asserts value is T>
): T {
  for (const validator of validators) {
    validator(value);
  }
  return value as T;
}

/**
 * 创建字符串验证器
 * @param fieldName 字段名
 * @param required 是否必需
 * @returns 字符串验证器
 */
export function createStringValidator(fieldName: string, required = false) {
  return (value: unknown): asserts value is string => {
    if (required && (value === undefined || value === null)) {
      throw new ValidationError(`${fieldName}是必需的`, fieldName);
    }
    
    if (value !== undefined && typeof value !== 'string') {
      throw new ValidationError(`${fieldName}必须是字符串`, fieldName);
    }
  };
}