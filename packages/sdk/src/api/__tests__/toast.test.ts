import { describe, it, expect, vi, beforeEach } from 'vitest';

vi.mock('../../core/ipc', () => ({
  invoke: vi.fn(),
}));

let toast: any;
let showToast: any;
let mockInvoke: any;

beforeEach(async () => {
  vi.clearAllMocks();
  const ipc = await import('../../core/ipc');
  mockInvoke = vi.mocked(ipc.invoke);

  const mod = await import('../toast');
  toast = mod.toast;
  showToast = mod.showToast;
});

describe('toast namespace', () => {
  it('应包含所有预期方法', () => {
    expect(typeof toast.show).toBe('function');
    expect(typeof toast.success).toBe('function');
    expect(typeof toast.error).toBe('function');
    expect(typeof toast.warning).toBe('function');
    expect(typeof toast.info).toBe('function');
  });
});

describe('showToast', () => {
  it('应调用 plugin_toast 命令', async () => {
    await showToast('Hello World');
    expect(mockInvoke).toHaveBeenCalledWith('plugin_toast', {
      message: 'Hello World',
      kind: 'default',
      duration: undefined,
    });
  });

  it('应透传 kind 和 duration', async () => {
    await showToast('Test', { kind: 'error', duration: 5000 });
    expect(mockInvoke).toHaveBeenCalledWith('plugin_toast', {
      message: 'Test',
      kind: 'error',
      duration: 5000,
    });
  });
});

describe('success', () => {
  it('应设置 kind 为 success', async () => {
    await toast.success('Done!');
    expect(mockInvoke).toHaveBeenCalledWith('plugin_toast', {
      message: 'Done!',
      kind: 'success',
      duration: undefined,
    });
  });

  it('应透传 duration', async () => {
    await toast.success('Done!', { duration: 3000 });
    expect(mockInvoke).toHaveBeenCalledWith('plugin_toast', {
      message: 'Done!',
      kind: 'success',
      duration: 3000,
    });
  });
});

describe('error', () => {
  it('应设置 kind 为 error', async () => {
    await toast.error('Failed!');
    expect(mockInvoke).toHaveBeenCalledWith('plugin_toast', {
      message: 'Failed!',
      kind: 'error',
      duration: undefined,
    });
  });
});

describe('warning', () => {
  it('应设置 kind 为 warning', async () => {
    await toast.warning('Caution');
    expect(mockInvoke).toHaveBeenCalledWith('plugin_toast', {
      message: 'Caution',
      kind: 'warning',
      duration: undefined,
    });
  });
});

describe('info', () => {
  it('应设置 kind 为 info', async () => {
    await toast.info('FYI');
    expect(mockInvoke).toHaveBeenCalledWith('plugin_toast', {
      message: 'FYI',
      kind: 'info',
      duration: undefined,
    });
  });
});
