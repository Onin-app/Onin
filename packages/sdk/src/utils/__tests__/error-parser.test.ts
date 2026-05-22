import { describe, it, expect, vi } from 'vitest';
import {
  parseError,
  parseHttpError,
  parseFsError,
  parseClipboardError,
  parseDialogError,
  httpErrorPatterns,
  fsErrorPatterns,
  clipboardErrorPatterns,
  dialogErrorPatterns,
} from '../error-parser';

function createTestError(message: string): Error {
  return new Error(message);
}

describe('parseError', () => {
  it('匹配到模式时返回具体的 PluginError', () => {
    const err = parseError(
      createTestError('connection timed out'),
      httpErrorPatterns,
    );
    expect(err.code).toBe('HTTP_TIMEOUT');
  });

  it('无匹配时返回 unknown 错误', () => {
    const err = parseError(
      createTestError('some random error'),
      httpErrorPatterns,
    );
    expect(err.code).toBe('COMMON_UNKNOWN');
  });

  it('context 透传到 createError', () => {
    const err = parseError(createTestError('not found'), fsErrorPatterns, {
      path: '/test.txt',
    });
    expect(err.code).toBe('FS_FILE_NOT_FOUND');
    expect(err.context?.path).toBe('/test.txt');
  });

  it('string 类型错误也能解析', () => {
    const err = parseError('network error', httpErrorPatterns);
    expect(err.code).toBe('HTTP_NETWORK_ERROR');
  });

  it('对象类型错误提取 .message', () => {
    const err = parseError({ message: 'disk full' }, fsErrorPatterns);
    expect(err.code).toBe('FS_DISK_FULL');
  });

  it('对象类型错误提取 .error', () => {
    const err = parseError({ error: 'timed out' }, httpErrorPatterns);
    expect(err.code).toBe('HTTP_TIMEOUT');
  });
});

describe('parseHttpError', () => {
  it('timeout 模式匹配', () => {
    const err = parseHttpError(createTestError('request timed out'));
    expect(err.code).toBe('HTTP_TIMEOUT');
  });

  it('network error 模式匹配', () => {
    const err = parseHttpError(createTestError('Network error occurred'));
    expect(err.code).toBe('HTTP_NETWORK_ERROR');
  });

  it('permission denied 匹配', () => {
    const err = parseHttpError(createTestError('Permission denied'), {
      url: 'https://api.example.com',
    });
    expect(err.code).toBe('COMMON_PERMISSION_DENIED');
    expect(err.context?.url).toBe('https://api.example.com');
  });

  it('不匹配返回 unknown', () => {
    const err = parseHttpError(createTestError('something weird happened'));
    expect(err.code).toBe('COMMON_UNKNOWN');
  });
});

describe('parseFsError', () => {
  it('not found 匹配', () => {
    const err = parseFsError(createTestError('File not found'), {
      path: '/data/file.txt',
    });
    expect(err.code).toBe('FS_FILE_NOT_FOUND');
    expect(err.context?.path).toBe('/data/file.txt');
  });

  it('access denied 匹配', () => {
    const err = parseFsError(createTestError('Access denied'));
    expect(err.code).toBe('FS_FILE_ACCESS_DENIED');
  });

  it('disk full 匹配', () => {
    const err = parseFsError(createTestError('No space left on device'));
    expect(err.code).toBe('FS_DISK_FULL');
  });

  it('already exists 匹配', () => {
    const err = parseFsError(createTestError('file already exists'));
    expect(err.code).toBe('FS_FILE_ALREADY_EXISTS');
  });

  it('invalid path 匹配', () => {
    const err = parseFsError(createTestError('invalid path'));
    expect(err.code).toBe('FS_INVALID_PATH');
  });

  it('permission denied 优先于 fs access denied', () => {
    const err = parseFsError(createTestError('Permission denied'), {
      path: '/secret',
    });
    expect(err.code).toBe('COMMON_PERMISSION_DENIED');
  });

  it('不匹配返回 unknown', () => {
    const err = parseFsError(createTestError('unrelated error'));
    expect(err.code).toBe('COMMON_UNKNOWN');
  });
});

describe('parseClipboardError', () => {
  it('format not supported 匹配', () => {
    const err = parseClipboardError(createTestError('format not supported'));
    expect(err.code).toBe('CLIPBOARD_FORMAT_UNSUPPORTED');
  });

  it('empty 匹配', () => {
    const err = parseClipboardError(createTestError('clipboard empty'));
    expect(err.code).toBe('CLIPBOARD_EMPTY');
  });

  it('access denied 匹配', () => {
    const err = parseClipboardError(createTestError('clipboard access denied'));
    expect(err.code).toBe('CLIPBOARD_ACCESS_DENIED');
  });

  it('unavailable 匹配', () => {
    const err = parseClipboardError(createTestError('clipboard not available'));
    expect(err.code).toBe('CLIPBOARD_UNAVAILABLE');
  });

  it('permission denied 匹配', () => {
    const err = parseClipboardError(createTestError('Permission denied'));
    expect(err.code).toBe('COMMON_PERMISSION_DENIED');
  });

  it('不匹配返回 unknown', () => {
    const err = parseClipboardError(createTestError('some error'));
    expect(err.code).toBe('COMMON_UNKNOWN');
  });
});

describe('parseDialogError', () => {
  it('cancelled 匹配', () => {
    const err = parseDialogError(createTestError('user cancelled'));
    expect(err.code).toBe('DIALOG_CANCELLED');
  });

  it('unavailable 匹配', () => {
    const err = parseDialogError(createTestError('dialog not available'));
    expect(err.code).toBe('DIALOG_UNAVAILABLE');
  });

  it('invalid options 匹配', () => {
    const err = parseDialogError(createTestError('invalid options'));
    expect(err.code).toBe('DIALOG_INVALID_OPTIONS');
  });

  it('permission denied 匹配', () => {
    const err = parseDialogError(createTestError('Permission denied'));
    expect(err.code).toBe('COMMON_PERMISSION_DENIED');
  });
});
