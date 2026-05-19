import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { openExternalLink } from '../link';

const mockInvoke = vi.fn();

vi.mock('@tauri-apps/api/core', () => ({
  invoke: (...args: unknown[]) => mockInvoke(...args),
}));

vi.mock('@tauri-apps/plugin-opener', () => ({
  openUrl: vi.fn().mockResolvedValue(undefined),
}));

describe('openExternalLink', () => {
  beforeEach(() => {
    mockInvoke.mockReset();
    mockInvoke.mockResolvedValue(undefined);
    vi.useFakeTimers();
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  function makeEvent(overrides: Partial<MouseEvent> = {}): MouseEvent {
    return {
      button: 0,
      ctrlKey: false,
      metaKey: false,
      shiftKey: false,
      altKey: false,
      preventDefault: vi.fn(),
      ...overrides,
    } as unknown as MouseEvent;
  }

  it('does nothing for empty href', async () => {
    const event = makeEvent();
    await openExternalLink('', event);
    expect(event.preventDefault).not.toHaveBeenCalled();
  });

  it('does nothing for non-primary button', async () => {
    const event = makeEvent({ button: 1 });
    await openExternalLink('https://example.com', event);
    expect(event.preventDefault).not.toHaveBeenCalled();
  });

  it('does nothing when modifier keys held', async () => {
    const ctrl = makeEvent({ ctrlKey: true });
    await openExternalLink('https://example.com', ctrl);
    expect(ctrl.preventDefault).not.toHaveBeenCalled();

    const meta = makeEvent({ metaKey: true });
    await openExternalLink('https://example.com', meta);
    expect(meta.preventDefault).not.toHaveBeenCalled();
  });

  it('acquires lock and opens URL on primary click', async () => {
    const event = makeEvent();
    await openExternalLink('https://example.com', event);
    expect(event.preventDefault).toHaveBeenCalled();
    expect(mockInvoke).toHaveBeenCalledWith('acquire_window_close_lock');
  });

  it('releases lock after timeout', async () => {
    const event = makeEvent();
    await openExternalLink('https://example.com', event);
    await vi.advanceTimersByTimeAsync(500);
    expect(mockInvoke).toHaveBeenCalledWith('release_window_close_lock');
  });

  it('releases lock on error', async () => {
    const { openUrl } = await import('@tauri-apps/plugin-opener');
    vi.mocked(openUrl).mockRejectedValueOnce(new Error('failed'));
    const consoleSpy = vi.spyOn(console, 'error').mockImplementation(() => {});
    const event = makeEvent();
    await openExternalLink('https://example.com', event);
    expect(mockInvoke).toHaveBeenCalledWith('release_window_close_lock');
    consoleSpy.mockRestore();
  });
});
