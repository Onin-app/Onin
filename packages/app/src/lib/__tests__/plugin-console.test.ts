import { describe, it, expect, vi, beforeEach } from 'vitest';

vi.mock('@tauri-apps/api/event', () => ({
  listen: vi.fn(),
}));

import { setupPluginConsoleListener } from '../plugin-console';
import { listen } from '@tauri-apps/api/event';

const mockListen = vi.mocked(listen);

describe('setupPluginConsoleListener', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mockListen.mockResolvedValue(() => {});
  });

  it('listens to plugin_console_log event', () => {
    setupPluginConsoleListener();
    expect(mockListen).toHaveBeenCalledWith(
      'plugin_console_log',
      expect.any(Function),
    );
  });

  it('returns undefined (listen not awaited in setup)', () => {
    const result = setupPluginConsoleListener();
    expect(result).toBeUndefined();
  });
});
