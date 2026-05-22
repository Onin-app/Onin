import { describe, it, expect, vi, beforeEach } from 'vitest';

// Mock the ipc module to prevent scheduler.ts immediately calling listen and throwing environment error on import
vi.mock('../core/ipc', () => ({
  invoke: vi.fn(),
  listen: vi.fn(() => Promise.resolve(() => {})),
}));

beforeEach(() => {
  vi.unstubAllGlobals();
  vi.resetModules();
  // Clean up any global variables left from previous imports
  delete (globalThis as any).__PLUGIN_ID__;
  if (typeof window !== 'undefined') {
    delete (window as any).__PLUGIN_ID__;
  }
});

describe('index.ts', () => {
  it('在 Node 环境下（window 未定义），不执行任何初始化操作', async () => {
    // 确保 window 为 undefined
    vi.stubGlobal('window', undefined);

    const sdk = await import('../index');

    expect((globalThis as any).__PLUGIN_ID__).toBeUndefined();
    expect(sdk.http).toBeDefined();
    expect(sdk.storage).toBeDefined();
    expect(sdk.fs).toBeDefined();
  });

  it('在 Browser 环境下（window 已定义），但 URL 中没有 plugin_id 参数时，不进行初始化', async () => {
    // 模拟没有 plugin_id 的 window 对象
    const mockWindow = {
      location: {
        search: '?other_param=foo',
      },
    };
    vi.stubGlobal('window', mockWindow);

    const sdk = await import('../index');

    expect((mockWindow as any).__PLUGIN_ID__).toBeUndefined();
    expect((globalThis as any).__PLUGIN_ID__).toBeUndefined();
    expect(sdk.http).toBeDefined();
  });

  it('在 Browser 环境下（window 已定义），且 URL 中含有 plugin_id 参数时，正确将其设置到 window 和 globalThis', async () => {
    // 模拟带有 plugin_id 的 window 对象
    const mockWindow = {
      location: {
        search: '?plugin_id=com.example.test-plugin&another=123',
      },
    };
    vi.stubGlobal('window', mockWindow);

    const sdk = await import('../index');

    expect((mockWindow as any).__PLUGIN_ID__).toBe('com.example.test-plugin');
    expect((globalThis as any).__PLUGIN_ID__).toBe('com.example.test-plugin');
    expect(sdk.http).toBeDefined();
    expect(sdk.storage).toBeDefined();
  });

  it('确保正确导出了所有的 API 和命名空间对象', async () => {
    vi.stubGlobal('window', undefined);
    const sdk = await import('../index');

    // 验证核心 API 是否都被正确导出
    expect(sdk.http).toBeDefined();
    expect(sdk.storage).toBeDefined();
    expect(sdk.notification).toBeDefined();
    expect(sdk.command).toBeDefined();
    expect(sdk.fs).toBeDefined();
    expect(sdk.dialog).toBeDefined();
    expect(sdk.clipboard).toBeDefined();
    expect(sdk.settings).toBeDefined();
    expect(sdk.lifecycle).toBeDefined();
    expect(sdk.scheduler).toBeDefined();
    expect(sdk.pluginWindow).toBeDefined();
    expect(sdk.ai).toBeDefined();
    expect(sdk.toast).toBeDefined();
    expect(sdk.invoke).toBeDefined();
    expect(sdk.listen).toBeDefined();
    expect(sdk.debug).toBeDefined();
    expect(sdk.error).toBeDefined();
    expect(sdk.retry).toBeDefined();
    expect(sdk.types).toBeDefined();
  });
});
