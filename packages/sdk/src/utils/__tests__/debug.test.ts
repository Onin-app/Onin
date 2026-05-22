import { describe, it, expect, vi, beforeEach } from 'vitest';

vi.mock('../../core/ipc', () => ({
  invoke: vi.fn(),
}));

let debug: any;
let mockInvoke: any;

beforeEach(async () => {
  vi.clearAllMocks();
  vi.unstubAllGlobals();

  const ipc = await import('../../core/ipc');
  mockInvoke = vi.mocked(ipc.invoke);

  const mod = await import('../debug');
  debug = mod.debug;
});

describe('version', () => {
  it('应返回 0.0.1', () => {
    expect(debug.version).toBe('0.0.1');
  });
});

describe('getRuntimeInfo', () => {
  it('应包含 timestamp', () => {
    const info = debug.getRuntimeInfo();
    expect(info.timestamp).toBeGreaterThan(0);
  });

  it('应包含 userAgent', () => {
    vi.stubGlobal('navigator', { userAgent: 'vitest', platform: 'Node.js' });
    const info = debug.getRuntimeInfo();
    expect(info.userAgent).toBe('vitest');
    expect(info.platform).toBe('Node.js');
  });

  it('navigator 不可用时使用 Deno Runtime', () => {
    vi.stubGlobal('navigator', undefined);
    const info = debug.getRuntimeInfo();
    expect(info.userAgent).toBe('Deno Runtime');
    expect(info.platform).toBe('Unknown');
  });
});

describe('testConnection', () => {
  it('成功时返回 { success: true, result }', async () => {
    mockInvoke.mockResolvedValue('pong');
    const result = await debug.testConnection();
    expect(result).toEqual({ success: true, result: 'pong' });
    expect(mockInvoke).toHaveBeenCalledWith('plugin_test_connection', {});
  });

  it('失败时返回 { success: false, error }', async () => {
    mockInvoke.mockRejectedValue(new Error('connection failed'));
    const result = await debug.testConnection();
    expect(result.success).toBe(false);
    expect(result.error).toBe('connection failed');
  });
});
