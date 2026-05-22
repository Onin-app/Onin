import { describe, it, expect } from 'vitest';
import { errorCode, createError, errorUtils } from '../errors';

describe('errorCode', () => {
  it('应包含所有命名空间的错误代码', () => {
    expect(errorCode.common.UNKNOWN).toBe('COMMON_UNKNOWN');
    expect(errorCode.common.PERMISSION_DENIED).toBe('COMMON_PERMISSION_DENIED');
    expect(errorCode.common.INVALID_ARGUMENT).toBe('COMMON_INVALID_ARGUMENT');
    expect(errorCode.http.NETWORK_ERROR).toBe('HTTP_NETWORK_ERROR');
    expect(errorCode.http.TIMEOUT).toBe('HTTP_TIMEOUT');
    expect(errorCode.http.HTTP_ERROR).toBe('HTTP_HTTP_ERROR');
    expect(errorCode.fs.FILE_NOT_FOUND).toBe('FS_FILE_NOT_FOUND');
    expect(errorCode.fs.DISK_FULL).toBe('FS_DISK_FULL');
    expect(errorCode.clipboard.UNAVAILABLE).toBe('CLIPBOARD_UNAVAILABLE');
    expect(errorCode.dialog.CANCELLED).toBe('DIALOG_CANCELLED');
    expect(errorCode.storage.QUOTA_EXCEEDED).toBe('STORAGE_QUOTA_EXCEEDED');
  });
});

describe('createError.common', () => {
  it('unknown 创建正确错误', () => {
    const err = createError.common.unknown('something went wrong');
    expect(err.name).toBe('PluginError');
    expect(err.code).toBe('COMMON_UNKNOWN');
    expect(err.message).toBe('something went wrong');
  });

  it('permissionDenied 包含资源名称', () => {
    const err = createError.common.permissionDenied('clipboard');
    expect(err.code).toBe('COMMON_PERMISSION_DENIED');
    expect(err.message).toContain('clipboard');
  });

  it('invalidArgument 包含参数名称', () => {
    const err = createError.common.invalidArgument('timeout');
    expect(err.code).toBe('COMMON_INVALID_ARGUMENT');
    expect(err.message).toContain('timeout');
  });
});

describe('createError.http', () => {
  it('networkError 创建正确', () => {
    const err = createError.http.networkError('connection failed');
    expect(err.code).toBe('HTTP_NETWORK_ERROR');
    expect(err.message).toBe('connection failed');
  });

  it('timeout 包含 URL 和超时时间', () => {
    const err = createError.http.timeout('https://example.com', 5000);
    expect(err.code).toBe('HTTP_TIMEOUT');
    expect(err.message).toContain('https://example.com');
    expect(err.context).toEqual({ url: 'https://example.com', timeout: 5000 });
  });

  it('httpError 包含状态码', () => {
    const err = createError.http.httpError(404, 'Not Found');
    expect(err.code).toBe('HTTP_HTTP_ERROR');
    expect(err.message).toContain('404');
    expect(err.context?.status).toBe(404);
  });
});

describe('createError.fs', () => {
  it('fileNotFound 包含路径', () => {
    const err = createError.fs.fileNotFound('/path/to/file.txt');
    expect(err.code).toBe('FS_FILE_NOT_FOUND');
    expect(err.message).toContain('/path/to/file.txt');
  });

  it('fileAccessDenied 包含路径', () => {
    const err = createError.fs.fileAccessDenied('/restricted');
    expect(err.code).toBe('FS_FILE_ACCESS_DENIED');
  });

  it('diskFull 包含路径', () => {
    const err = createError.fs.diskFull('/data');
    expect(err.code).toBe('FS_DISK_FULL');
  });

  it('fileAlreadyExists 包含路径', () => {
    const err = createError.fs.fileAlreadyExists('/existing');
    expect(err.code).toBe('FS_FILE_ALREADY_EXISTS');
  });

  it('invalidPath 包含路径', () => {
    const err = createError.fs.invalidPath('//bad');
    expect(err.code).toBe('FS_INVALID_PATH');
  });
});

describe('createError.clipboard', () => {
  it('unavailable 创建正确', () => {
    const err = createError.clipboard.unavailable();
    expect(err.code).toBe('CLIPBOARD_UNAVAILABLE');
  });

  it('formatUnsupported 包含格式', () => {
    const err = createError.clipboard.formatUnsupported('image/gif');
    expect(err.code).toBe('CLIPBOARD_FORMAT_UNSUPPORTED');
    expect(err.context?.format).toBe('image/gif');
  });

  it('empty 创建正确', () => {
    const err = createError.clipboard.empty();
    expect(err.code).toBe('CLIPBOARD_EMPTY');
  });

  it('accessDenied 创建正确', () => {
    const err = createError.clipboard.accessDenied();
    expect(err.code).toBe('CLIPBOARD_ACCESS_DENIED');
  });
});

describe('createError.dialog', () => {
  it('cancelled 创建正确', () => {
    const err = createError.dialog.cancelled();
    expect(err.code).toBe('DIALOG_CANCELLED');
  });

  it('unavailable 创建正确', () => {
    const err = createError.dialog.unavailable();
    expect(err.code).toBe('DIALOG_UNAVAILABLE');
  });

  it('invalidOptions 包含原因', () => {
    const err = createError.dialog.invalidOptions('bad selection');
    expect(err.code).toBe('DIALOG_INVALID_OPTIONS');
    expect(err.context?.reason).toBe('bad selection');
  });
});

describe('createError.storage', () => {
  it('quotaExceeded 创建正确', () => {
    const err = createError.storage.quotaExceeded();
    expect(err.code).toBe('STORAGE_QUOTA_EXCEEDED');
  });

  it('unavailable 创建正确', () => {
    const err = createError.storage.unavailable();
    expect(err.code).toBe('STORAGE_UNAVAILABLE');
  });
});

describe('errorUtils', () => {
  describe('isPluginError', () => {
    it('应对 PluginError 返回 true', () => {
      const err = createError.common.unknown('test');
      expect(errorUtils.isPluginError(err)).toBe(true);
    });

    it('应对普通 Error 返回 false', () => {
      expect(errorUtils.isPluginError(new Error('test'))).toBe(false);
    });

    it('应对 null 返回 falsy', () => {
      expect(errorUtils.isPluginError(null)).toBeFalsy();
    });

    it('应对 undefined 返回 falsy', () => {
      expect(errorUtils.isPluginError(undefined)).toBeFalsy();
    });

    it('应对普通对象返回 false', () => {
      expect(errorUtils.isPluginError({ message: 'test' })).toBe(false);
    });
  });

  describe('isErrorCode', () => {
    it('应精确匹配错误代码', () => {
      const err = createError.http.networkError('fail');
      expect(errorUtils.isErrorCode(err, 'HTTP_NETWORK_ERROR')).toBe(true);
      expect(errorUtils.isErrorCode(err, 'HTTP_TIMEOUT')).toBe(false);
    });

    it('非 PluginError 返回 false', () => {
      expect(errorUtils.isErrorCode(new Error('fail'), 'COMMON_UNKNOWN')).toBe(
        false,
      );
    });
  });

  describe('isOneOfErrorCodes', () => {
    it('应匹配多个代码中的任意一个', () => {
      const err = createError.http.timeout('url', 5000);
      expect(
        errorUtils.isOneOfErrorCodes(err, [
          'HTTP_NETWORK_ERROR',
          'HTTP_TIMEOUT',
        ]),
      ).toBe(true);
      expect(errorUtils.isOneOfErrorCodes(err, ['COMMON_UNKNOWN'])).toBe(false);
    });
  });

  describe('getErrorInfo', () => {
    it('应返回结构化的错误信息', () => {
      const err = createError.common.unknown('oops', { detail: 'info' });
      const info = errorUtils.getErrorInfo(err);
      expect(info).toEqual({
        name: 'PluginError',
        code: 'COMMON_UNKNOWN',
        message: 'oops',
        context: { detail: 'info' },
      });
    });

    it('非 PluginError 返回 null', () => {
      expect(errorUtils.getErrorInfo('string error')).toBeNull();
    });
  });
});
