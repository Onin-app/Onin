import { describe, it, expect, vi, beforeEach } from 'vitest';
import { showToastOverlay } from '../toastOverlay';

const mockInvoke = vi.fn();

vi.mock('@tauri-apps/api/core', () => ({
  invoke: (...args: unknown[]) => mockInvoke(...args),
}));

describe('showToastOverlay', () => {
  beforeEach(() => {
    mockInvoke.mockReset();
    mockInvoke.mockResolvedValue(undefined);
  });

  it('calls invoke with message and defaults', async () => {
    await showToastOverlay('Hello');
    expect(mockInvoke).toHaveBeenCalledWith('show_toast_overlay', {
      message: 'Hello',
      kind: 'default',
      duration: 1400,
    });
  });

  it('uses provided kind and duration', async () => {
    await showToastOverlay('Error!', { kind: 'error', duration: 3000 });
    expect(mockInvoke).toHaveBeenCalledWith('show_toast_overlay', {
      message: 'Error!',
      kind: 'error',
      duration: 3000,
    });
  });

  it('supports all toast kinds', async () => {
    for (const kind of ['default', 'success', 'error', 'warning', 'info'] as const) {
      await showToastOverlay('test', { kind });
      expect(mockInvoke).toHaveBeenLastCalledWith('show_toast_overlay', {
        message: 'test',
        kind,
        duration: 1400,
      });
    }
  });

  it('propagates invoke errors', async () => {
    mockInvoke.mockRejectedValue(new Error('backend error'));
    await expect(showToastOverlay('test')).rejects.toThrow('backend error');
  });
});
