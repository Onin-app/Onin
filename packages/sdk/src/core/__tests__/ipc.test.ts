import { describe, it, expect, vi, beforeEach } from 'vitest';
import * as env from '../environment';

vi.mock('@tauri-apps/api/core', () => ({ invoke: vi.fn() }));
vi.mock('@tauri-apps/api/event', () => ({
  listen: vi.fn(() => Promise.resolve(vi.fn())),
}));
vi.mock('../environment', async (importOriginal) => {
  const original = await importOriginal<typeof import('../environment')>();
  return {
    ...original,
    getEnvironment: vi.fn(original.getEnvironment),
  };
});

beforeEach(() => {
  vi.resetModules();
  vi.unstubAllGlobals();
  vi.clearAllMocks();
  delete (globalThis as any).__ONIN_COMMAND_HANDLER__;
});

describe('invoke', () => {
  it('Webview 环境调用 @tauri-apps/api/core 的 invoke', async () => {
    vi.stubGlobal('window', { __TAURI_INTERNALS__: {} });
    const { invoke } = await import('../ipc');
    const { invoke: coreInvoke } = await import('@tauri-apps/api/core');
    vi.mocked(coreInvoke).mockResolvedValue('webview_ok');

    const result = await invoke('test_method', { key: 'val' });
    expect(result).toBe('webview_ok');
    expect(coreInvoke).toHaveBeenCalledWith('test_method', { key: 'val' });
  });

  it('Webview 环境下 loadInvoke 缓存有效（重复调用不重新检测环境）', async () => {
    vi.stubGlobal('window', { __TAURI_INTERNALS__: {} });

    const { invoke } = await import('../ipc');
    const { invoke: coreInvoke } = await import('@tauri-apps/api/core');
    vi.mocked(coreInvoke).mockResolvedValue('webview_ok');

    await invoke('a');
    await invoke('b');

    expect(env.getEnvironment).toHaveBeenCalledTimes(1);
  });

  it('Headless 环境调用 Deno.core.ops.op_invoke', async () => {
    const opInvoke = vi
      .fn()
      .mockResolvedValue({ type: 'ok', value: 'headless_ok' });
    vi.stubGlobal('Deno', { core: { ops: { op_invoke: opInvoke } } });
    const { invoke } = await import('../ipc');

    const result = await invoke('test_method', { key: 'val' });
    expect(result).toBe('headless_ok');
    expect(opInvoke).toHaveBeenCalledWith('test_method', { key: 'val' });
  });

  it('Headless 环境收到 type=error 时抛出', async () => {
    const opInvoke = vi
      .fn()
      .mockResolvedValue({ type: 'error', error: 'fail msg' });
    vi.stubGlobal('Deno', { core: { ops: { op_invoke: opInvoke } } });
    const { invoke } = await import('../ipc');

    await expect(invoke('fail')).rejects.toThrow('fail msg');
  });

  it('Headless 环境兼容旧格式 {error: ...} 时抛出', async () => {
    const opInvoke = vi.fn().mockResolvedValue({ error: 'legacy error' });
    vi.stubGlobal('Deno', { core: { ops: { op_invoke: opInvoke } } });
    const { invoke } = await import('../ipc');

    await expect(invoke('fail')).rejects.toThrow('legacy error');
  });

  it('Headless 环境原始值直接返回（无 type/error 字段）', async () => {
    const opInvoke = vi.fn().mockResolvedValue(42);
    vi.stubGlobal('Deno', { core: { ops: { op_invoke: opInvoke } } });
    const { invoke } = await import('../ipc');

    const result = await invoke('get_num');
    expect(result).toBe(42);
  });

  it('Unknown 环境抛出 Error', async () => {
    const { invoke } = await import('../ipc');
    await expect(invoke('x')).rejects.toThrow(
      'Unsupported runtime environment',
    );
  });
});

describe('listen', () => {
  it('Webview 环境调用 @tauri-apps/api/event 的 listen', async () => {
    vi.stubGlobal('window', { __TAURI_INTERNALS__: {} });
    const { listen } = await import('../ipc');
    const { listen: eventListen } = await import('@tauri-apps/api/event');
    const handler = vi.fn();

    await listen('my-event' as any, handler);
    expect(eventListen).toHaveBeenCalledWith('my-event', handler);
  });

  it('Webview 环境下 loadListen 缓存有效（重复调用不重新检测环境）', async () => {
    vi.stubGlobal('window', { __TAURI_INTERNALS__: {} });

    const { listen } = await import('../ipc');

    await listen('e1' as any, vi.fn());
    await listen('e2' as any, vi.fn());

    expect(env.getEnvironment).toHaveBeenCalledTimes(1);
  });

  it('Headless 环境 plugin_command_execute 注册全局 handler', async () => {
    vi.stubGlobal('Deno', { core: { ops: {} } });
    const { listen } = await import('../ipc');
    const handler = vi.fn();

    const unlisten = await listen('plugin_command_execute' as any, handler);
    expect(typeof unlisten).toBe('function');
    expect((globalThis as any).__ONIN_COMMAND_HANDLER__).toBe(handler);
  });

  it('Headless 环境其他事件返回 no-op unlisten', async () => {
    vi.stubGlobal('Deno', { core: { ops: {} } });
    const { listen } = await import('../ipc');

    const unlisten = await listen('other-event' as any, vi.fn());
    expect(typeof unlisten).toBe('function');
    expect((globalThis as any).__ONIN_COMMAND_HANDLER__).toBeUndefined();
  });

  it('Unknown 环境抛出 Error', async () => {
    const { listen } = await import('../ipc');
    await expect(listen('x' as any, vi.fn())).rejects.toThrow(
      'Unsupported runtime environment',
    );
  });
});
