import { describe, it, expect } from 'vitest';
import { 
  validateNotificationOptions, 
  validatePipeline, 
  createStringValidator 
} from './validation';
import { ValidationError } from '../types';

describe('参数验证', () => {
  describe('validateNotificationOptions', () => {
    it('应该验证有效的通知选项', () => {
      const validOptions = {
        title: '测试标题',
        body: '测试内容',
        icon: 'test-icon'
      };

      expect(() => validateNotificationOptions(validOptions)).not.toThrow();
    });

    it('应该验证只有标题的通知选项', () => {
      const validOptions = {
        title: '测试标题'
      };

      expect(() => validateNotificationOptions(validOptions)).not.toThrow();
    });

    it('应该在缺少标题时抛出验证错误', () => {
      const invalidOptions = {
        body: '测试内容'
      };

      expect(() => validateNotificationOptions(invalidOptions)).toThrow(ValidationError);
      
      try {
        validateNotificationOptions(invalidOptions);
      } catch (error) {
        expect(error).toBeInstanceOf(ValidationError);
        expect((error as ValidationError).field).toBe('title');
      }
    });

    it('应该在标题不是字符串时抛出验证错误', () => {
      const invalidOptions = {
        title: 123
      };

      expect(() => validateNotificationOptions(invalidOptions)).toThrow(ValidationError);
    });

    it('应该在 body 不是字符串时抛出验证错误', () => {
      const invalidOptions = {
        title: '测试标题',
        body: 123
      };

      expect(() => validateNotificationOptions(invalidOptions)).toThrow(ValidationError);
    });

    it('应该在 icon 不是字符串时抛出验证错误', () => {
      const invalidOptions = {
        title: '测试标题',
        icon: 123
      };

      expect(() => validateNotificationOptions(invalidOptions)).toThrow(ValidationError);
    });

    it('应该在传入 null 或 undefined 时抛出验证错误', () => {
      expect(() => validateNotificationOptions(null)).toThrow(ValidationError);
      expect(() => validateNotificationOptions(undefined)).toThrow(ValidationError);
    });
  });

  describe('validatePipeline', () => {
    it('应该通过所有验证器', () => {
      const validator1 = (value: unknown): asserts value is string => {
        if (typeof value !== 'string') throw new Error('不是字符串');
      };
      
      const validator2 = (value: unknown): asserts value is string => {
        if ((value as string).length < 3) throw new Error('太短');
      };

      const result = validatePipeline('测试字符串', [validator1, validator2]);
      expect(result).toBe('测试字符串');
    });

    it('应该在验证失败时抛出错误', () => {
      const validator = (value: unknown): asserts value is string => {
        if (typeof value !== 'string') throw new Error('不是字符串');
      };

      expect(() => validatePipeline(123, [validator])).toThrow('不是字符串');
    });
  });

  describe('createStringValidator', () => {
    it('应该创建必需字符串验证器', () => {
      const validator = createStringValidator('测试字段', true);
      
      expect(() => validator('测试值')).not.toThrow();
      expect(() => validator(undefined)).toThrow(ValidationError);
      expect(() => validator(123)).toThrow(ValidationError);
    });

    it('应该创建可选字符串验证器', () => {
      const validator = createStringValidator('测试字段', false);
      
      expect(() => validator('测试值')).not.toThrow();
      expect(() => validator(undefined)).not.toThrow();
      expect(() => validator(123)).toThrow(ValidationError);
    });
  });
});