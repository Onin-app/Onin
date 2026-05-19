import { describe, it, expect, vi, beforeEach } from 'vitest';

const mockInvoke = vi.fn();

vi.mock('@tauri-apps/api/core', () => ({
  invoke: (...args: unknown[]) => mockInvoke(...args),
}));

describe('detachWindowShortcut store', () => {
  beforeEach(() => {
    mockInvoke.mockReset();
    vi.resetModules();
  });

  it('loads shortcut from backend on first subscribe', async () => {
    mockInvoke.mockResolvedValue('Ctrl+Shift+D');
    const { detachWindowShortcut } = await import('../shortcuts');
    let value = '';
    const unsub = detachWindowShortcut.subscribe((v) => { value = v; });
    await vi.waitFor(() => {
      expect(mockInvoke).toHaveBeenCalledWith('get_detach_window_shortcut');
    });
    expect(value).toBe('Ctrl+Shift+D');
    unsub();
  });

  it('setShortcut updates backend and store', async () => {
    mockInvoke.mockResolvedValue(undefined);
    const { detachWindowShortcut } = await import('../shortcuts');
    await detachWindowShortcut.setShortcut('Ctrl+Alt+T');
    expect(mockInvoke).toHaveBeenCalledWith('set_detach_window_shortcut', {
      shortcutStr: 'Ctrl+Alt+T',
    });
    let value = '';
    const unsub = detachWindowShortcut.subscribe((v) => { value = v; });
    expect(value).toBe('Ctrl+Alt+T');
    unsub();
  });

  it('setShortcut throws when backend fails', async () => {
    mockInvoke.mockRejectedValue(new Error('backend error'));
    const { detachWindowShortcut } = await import('../shortcuts');
    const consoleSpy = vi.spyOn(console, 'error').mockImplementation(() => {});
    await expect(detachWindowShortcut.setShortcut('Ctrl+X')).rejects.toThrow('backend error');
    consoleSpy.mockRestore();
  });

  it('handles backend error on initial load gracefully', async () => {
    mockInvoke.mockRejectedValue(new Error('not found'));
    const consoleSpy = vi.spyOn(console, 'error').mockImplementation(() => {});
    const { detachWindowShortcut } = await import('../shortcuts');
    let value: string | undefined;
    const unsub = detachWindowShortcut.subscribe((v) => { value = v; });
    await vi.waitFor(() => {
      expect(consoleSpy).toHaveBeenCalledWith(
        'Failed to load detach window shortcut:',
        expect.any(Error),
      );
    });
    unsub();
    consoleSpy.mockRestore();
  });
});
