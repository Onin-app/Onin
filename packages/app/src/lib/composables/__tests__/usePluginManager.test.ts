import { describe, it, expect, vi, beforeEach } from 'vitest';

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}));

vi.mock('@tauri-apps/api/event', () => ({
  listen: vi.fn(),
}));

import { usePluginManager } from '../usePluginManager.svelte';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';

const mockInvoke = vi.mocked(invoke);
const mockListen = vi.mocked(listen);

describe('usePluginManager', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mockInvoke.mockResolvedValue(undefined);
    mockListen.mockResolvedValue(() => {});
  });

  it('should initialize with default state', () => {
    const pm = usePluginManager();
    expect(pm.state.showPluginInline).toBe(false);
    expect(pm.state.currentPluginUrl).toBe('');
    expect(pm.state.currentPluginId).toBe('');
    expect(pm.state.currentPluginVersion).toBe('');
    expect(pm.state.currentPluginAutoDetach).toBe(false);
    expect(pm.state.currentPluginTerminateOnBg).toBe(false);
    expect(pm.state.currentPluginRunAtStartup).toBe(false);
  });

  it('closePlugin should reset state and call hide when terminateOnBg is false', () => {
    const pm = usePluginManager();
    pm.closePlugin();
    expect(mockInvoke).toHaveBeenCalledWith('hide_inline_plugin');
    expect(pm.state.showPluginInline).toBe(false);
    expect(pm.state.currentPluginId).toBe('');
    expect(pm.state.currentPluginUrl).toBe('');
  });

  it('closePlugin should call close when terminateOnBg is true', () => {
    const pm = usePluginManager();
    pm.state.currentPluginTerminateOnBg = true;
    pm.closePlugin();
    expect(mockInvoke).toHaveBeenCalledWith('close_inline_plugin');
  });

  it('detachPlugin should call confirm handler then close then open window', async () => {
    const pm = usePluginManager();
    pm.state.currentPluginId = 'test-plugin';
    const confirmHandler = vi.fn().mockResolvedValue(true);
    pm.setModeSwitchConfirmHandler(confirmHandler);

    await pm.detachPlugin();

    expect(confirmHandler).toHaveBeenCalledWith('inline-to-window');
    expect(mockInvoke).toHaveBeenNthCalledWith(3, 'close_inline_plugin');
    expect(mockInvoke).toHaveBeenNthCalledWith(4, 'open_plugin_in_window', {
      pluginId: 'test-plugin',
    });
  });

  it('detachPlugin should skip when confirm returns false', async () => {
    const pm = usePluginManager();
    pm.state.currentPluginId = 'test-plugin';
    const confirmHandler = vi.fn().mockResolvedValue(false);
    pm.setModeSwitchConfirmHandler(confirmHandler);

    await pm.detachPlugin();

    expect(mockInvoke).not.toHaveBeenCalled();
  });

  it('detachPlugin should skip when no pluginId', async () => {
    const pm = usePluginManager();
    await pm.detachPlugin();
    expect(mockInvoke).not.toHaveBeenCalled();
  });

  it('toggleAutoDetach should update state and call invoke', async () => {
    const pm = usePluginManager();
    pm.state.currentPluginId = 'test-plugin';

    await pm.toggleAutoDetach(true);

    expect(pm.state.currentPluginAutoDetach).toBe(true);
    expect(mockInvoke).toHaveBeenCalledWith('toggle_plugin_auto_detach', {
      pluginId: 'test-plugin',
      autoDetach: true,
    });
  });

  it('toggleAutoDetach should revert on error', async () => {
    const pm = usePluginManager();
    pm.state.currentPluginId = 'test-plugin';
    mockInvoke.mockImplementation(async (cmd) => {
      if (cmd === 'toggle_plugin_auto_detach') {
        throw new Error('fail');
      }
      return undefined;
    });

    await pm.toggleAutoDetach(true);

    expect(pm.state.currentPluginAutoDetach).toBe(false);
  });

  it('toggleAutoDetach should skip when no pluginId', async () => {
    const pm = usePluginManager();
    console.error = vi.fn();
    await pm.toggleAutoDetach(true);
    expect(console.error).toHaveBeenCalledWith('No current plugin ID');
  });

  it('toggleTerminateOnBg should update state and call invoke', async () => {
    const pm = usePluginManager();
    pm.state.currentPluginId = 'test-plugin';

    await pm.toggleTerminateOnBg(true);

    expect(pm.state.currentPluginTerminateOnBg).toBe(true);
    expect(mockInvoke).toHaveBeenCalledWith('toggle_plugin_terminate_on_bg', {
      pluginId: 'test-plugin',
      terminateOnBg: true,
    });
  });

  it('toggleTerminateOnBg should revert on error', async () => {
    const pm = usePluginManager();
    pm.state.currentPluginId = 'test-plugin';
    mockInvoke.mockRejectedValue(new Error('fail'));

    await pm.toggleTerminateOnBg(true);

    expect(pm.state.currentPluginTerminateOnBg).toBe(false);
  });

  it('toggleRunAtStartup should update state and call invoke', async () => {
    const pm = usePluginManager();
    pm.state.currentPluginId = 'test-plugin';

    await pm.toggleRunAtStartup(true);

    expect(pm.state.currentPluginRunAtStartup).toBe(true);
    expect(mockInvoke).toHaveBeenCalledWith('toggle_plugin_run_at_startup', {
      pluginId: 'test-plugin',
      runAtStartup: true,
    });
  });

  it('reloadPlugin should call invoke when showPluginInline', async () => {
    const pm = usePluginManager();
    pm.state.showPluginInline = true;
    await pm.reloadPlugin();
    expect(mockInvoke).toHaveBeenCalledWith('reload_inline_plugin');
  });

  it('reloadPlugin should skip when not inline', async () => {
    const pm = usePluginManager();
    await pm.reloadPlugin();
    expect(mockInvoke).not.toHaveBeenCalled();
  });

  it('openDevTools should call invoke', async () => {
    const pm = usePluginManager();
    await pm.openDevTools();
    expect(mockInvoke).toHaveBeenCalledWith('open_inline_plugin_devtools');
  });

  it('sendLifecycleEvent should call invoke with event', () => {
    const pm = usePluginManager();
    pm.sendLifecycleEvent('show');
    expect(mockInvoke).toHaveBeenCalledWith('post_inline_plugin_message', {
      message: { type: 'plugin-lifecycle-event', event: 'show' },
    });
  });

  it('setupListeners should register listeners and return cleanup', async () => {
    const unlisten1 = vi.fn();
    const unlisten2 = vi.fn();
    const unlisten3 = vi.fn();
    mockListen
      .mockResolvedValueOnce(unlisten1)
      .mockResolvedValueOnce(unlisten2)
      .mockResolvedValueOnce(unlisten3);

    const pm = usePluginManager();
    const cleanup = await pm.setupListeners();

    expect(mockListen).toHaveBeenCalledWith('show_plugin_inline', expect.any(Function));
    expect(mockListen).toHaveBeenCalledWith('window_visibility', expect.any(Function));
    expect(mockListen).toHaveBeenCalledWith('detach_window_shortcut', expect.any(Function));

    cleanup();
    expect(unlisten1).toHaveBeenCalled();
    expect(unlisten2).toHaveBeenCalled();
    expect(unlisten3).toHaveBeenCalled();
  });
});
