import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import {
  calculateExponentialBackoff,
  isRetryableError,
  getRetryDelay,
  withRetry,
  createRetryWrapper,
} from '../retry';
import { createError } from '../../types/errors';

describe('calculateExponentialBackoff', () => {
  it('第 1 次尝试 = baseDelay', () => {
    expect(calculateExponentialBackoff(1000, 1)).toBe(1000);
  });

  it('第 2 次尝试 = baseDelay * 2', () => {
    expect(calculateExponentialBackoff(1000, 2)).toBe(2000);
  });

  it('第 3 次尝试 = baseDelay * 4', () => {
    expect(calculateExponentialBackoff(500, 3)).toBe(2000);
  });

  it('不应超过 maxDelay', () => {
    expect(calculateExponentialBackoff(1000, 10, 5000)).toBe(5000);
  });

  it('默认 maxDelay 为 30000', () => {
    expect(calculateExponentialBackoff(10000, 5)).toBe(30000);
  });
});

describe('isRetryableError', () => {
  it('NETWORK_ERROR 可重试', () => {
    expect(isRetryableError(createError.http.networkError('fail'))).toBe(true);
  });

  it('TIMEOUT 可重试', () => {
    expect(isRetryableError(createError.http.timeout('url', 5000))).toBe(true);
  });

  it('HTTP_ERROR 429/500/502/503/504 可重试', () => {
    expect(
      isRetryableError(createError.http.httpError(429, 'Too Many Requests')),
    ).toBe(true);
    expect(
      isRetryableError(
        createError.http.httpError(500, 'Internal Server Error'),
      ),
    ).toBe(true);
    expect(
      isRetryableError(createError.http.httpError(502, 'Bad Gateway')),
    ).toBe(true);
    expect(
      isRetryableError(createError.http.httpError(503, 'Service Unavailable')),
    ).toBe(true);
    expect(
      isRetryableError(createError.http.httpError(504, 'Gateway Timeout')),
    ).toBe(true);
  });

  it('HTTP_ERROR 400/401/403 不可重试', () => {
    expect(
      isRetryableError(createError.http.httpError(400, 'Bad Request')),
    ).toBe(false);
    expect(
      isRetryableError(createError.http.httpError(401, 'Unauthorized')),
    ).toBe(false);
    expect(isRetryableError(createError.http.httpError(403, 'Forbidden'))).toBe(
      false,
    );
  });

  it('CLIPBOARD_UNAVAILABLE 可重试', () => {
    expect(isRetryableError(createError.clipboard.unavailable())).toBe(true);
  });

  it('其他 PluginError 不可重试', () => {
    expect(isRetryableError(createError.fs.fileNotFound('/test'))).toBe(false);
    expect(isRetryableError(createError.dialog.cancelled())).toBe(false);
  });

  it('非 PluginError 不可重试', () => {
    expect(isRetryableError(new Error('generic'))).toBe(false);
    expect(isRetryableError('string error')).toBe(false);
    expect(isRetryableError(null)).toBe(false);
  });
});

describe('getRetryDelay', () => {
  it('HTTP 429 使用 retryAfter', () => {
    const err = createError.http.httpError(429, 'Too Many Requests', {
      retryAfter: 30,
    });
    expect(getRetryDelay(err)).toBe(30000);
  });

  it('HTTP 500 从 10s 线性增长最高 60s', () => {
    const err = createError.http.httpError(500, 'ISE');
    expect(getRetryDelay(err, 1)).toBe(10000);
    expect(getRetryDelay(err, 6)).toBe(60000);
    expect(getRetryDelay(err, 10)).toBe(60000);
  });

  it('HTTP 502/503/504 从 5s 线性增长最高 30s', () => {
    const err1 = createError.http.httpError(502, 'Bad Gateway');
    expect(getRetryDelay(err1, 1)).toBe(5000);
    expect(getRetryDelay(err1, 6)).toBe(30000);

    const err2 = createError.http.httpError(503, 'Service Unavailable');
    expect(getRetryDelay(err2, 1)).toBe(5000);
  });

  it('TIMEOUT 从 5s 线性增长最高 30s', () => {
    const err = createError.http.timeout('url', 5000);
    expect(getRetryDelay(err, 1)).toBe(5000);
  });

  it('其他 PluginError 默认 3s*attempt 最高 15s', () => {
    const err = createError.clipboard.unavailable();
    expect(getRetryDelay(err, 1)).toBe(3000);
    expect(getRetryDelay(err, 5)).toBe(15000);
    expect(getRetryDelay(err, 10)).toBe(15000);
  });

  it('非 PluginError 使用默认退避', () => {
    expect(getRetryDelay(new Error('generic'), 1)).toBe(3000);
  });
});

describe('withRetry', () => {
  beforeEach(() => {
    vi.useFakeTimers();
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  it('首次成功直接返回结果', async () => {
    const op = vi.fn().mockResolvedValue('ok');
    await expect(withRetry(op, { getDelay: () => 1 })).resolves.toBe('ok');
    expect(op).toHaveBeenCalledTimes(1);
  });

  it('重试后最终成功返回结果', async () => {
    const op = vi
      .fn()
      .mockRejectedValueOnce(createError.http.networkError('fail'))
      .mockResolvedValueOnce('ok');
    const promise = withRetry(op, { getDelay: () => 1 });
    await vi.advanceTimersByTimeAsync(1);
    await expect(promise).resolves.toBe('ok');
    expect(op).toHaveBeenCalledTimes(2);
  });

  it('重试耗尽后抛最后错误', async () => {
    vi.useRealTimers();
    const err = createError.http.networkError('always fail');
    const op = vi.fn().mockRejectedValue(err);
    await expect(
      withRetry(op, { maxRetries: 2, getDelay: () => 1, baseDelay: 0 }),
    ).rejects.toThrow('always fail');
    expect(op).toHaveBeenCalledTimes(2);
    vi.useFakeTimers();
  });

  it('不可重试错误不重试直接抛出', async () => {
    const op = vi.fn().mockRejectedValue(createError.fs.fileNotFound('/x'));
    const promise = withRetry(op, { getDelay: () => 1 });
    await expect(promise).rejects.toThrow();
    expect(op).toHaveBeenCalledTimes(1);
  });

  it('自定义 shouldRetry 生效', async () => {
    const shouldRetry = vi.fn().mockReturnValue(true);
    const op = vi
      .fn()
      .mockRejectedValueOnce(new Error('any error'))
      .mockResolvedValueOnce('ok');
    const promise = withRetry(op, { shouldRetry, getDelay: () => 1 });
    await vi.advanceTimersByTimeAsync(1);
    await expect(promise).resolves.toBe('ok');
    expect(shouldRetry).toHaveBeenCalled();
  });

  it('onRetry 回调被调用', async () => {
    const onRetry = vi.fn();
    const op = vi
      .fn()
      .mockRejectedValueOnce(createError.http.networkError('fail'))
      .mockResolvedValueOnce('ok');
    const promise = withRetry(op, { onRetry, getDelay: () => 1 });
    await vi.advanceTimersByTimeAsync(1);
    await expect(promise).resolves.toBe('ok');
    expect(onRetry).toHaveBeenCalledOnce();
  });

  it('使用 exponentialBackoff 模式', async () => {
    const op = vi
      .fn()
      .mockRejectedValueOnce(createError.http.networkError('fail'))
      .mockRejectedValueOnce(createError.http.networkError('fail'))
      .mockResolvedValueOnce('ok');
    const promise = withRetry(op, {
      exponentialBackoff: true,
      baseDelay: 10,
      maxDelay: 100,
    });
    await vi.advanceTimersByTimeAsync(10);
    await vi.advanceTimersByTimeAsync(20);
    await expect(promise).resolves.toBe('ok');
  });
});

describe('createRetryWrapper', () => {
  beforeEach(() => {
    vi.useFakeTimers();
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  it('返回的函数包含重试逻辑', async () => {
    const fn = vi
      .fn()
      .mockRejectedValueOnce(createError.http.networkError('fail'))
      .mockResolvedValueOnce('ok');
    const wrapped = createRetryWrapper(fn, { getDelay: () => 1 });
    const promise = wrapped('arg1', 'arg2');
    await vi.advanceTimersByTimeAsync(1);
    await expect(promise).resolves.toBe('ok');
    expect(fn).toHaveBeenCalledWith('arg1', 'arg2');
  });
});
