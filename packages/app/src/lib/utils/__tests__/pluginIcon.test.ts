import { describe, it, expect, vi, beforeEach } from 'vitest';
import { getPluginIconUrl } from '../pluginIcon';
import type { PluginIconInfo } from '../pluginIcon';

const mockInvoke = vi.fn();
const mockFetch = vi.fn();

vi.mock('@tauri-apps/api/core', () => ({
  invoke: (...args: unknown[]) => mockInvoke(...args),
}));

global.fetch = mockFetch;

describe('getPluginIconUrl', () => {
  beforeEach(() => {
    mockInvoke.mockReset();
    mockFetch.mockReset();
  });

  it('returns marketplace URL directly for http(s) icons', async () => {
    const plugin: PluginIconInfo = { id: 'test', icon: 'https://example.com/icon.svg' };
    const result = await getPluginIconUrl(plugin);
    expect(result).toBe('https://example.com/icon.svg');
    expect(mockInvoke).not.toHaveBeenCalled();
  });

  it('resolves relative icon path via plugin server', async () => {
    mockInvoke.mockResolvedValue(8080);
    const plugin: PluginIconInfo = { id: 'my-plugin', icon: 'icon.svg' };
    const result = await getPluginIconUrl(plugin);
    expect(mockInvoke).toHaveBeenCalledWith('get_plugin_server_port');
    expect(result).toBe('http://127.0.0.1:8080/plugin/my-plugin/icon.svg');
  });

  it('uses dir_name when provided', async () => {
    mockInvoke.mockResolvedValue(8080);
    const plugin: PluginIconInfo = { id: 'test', dir_name: 'custom-dir', icon: 'icon.svg' };
    const result = await getPluginIconUrl(plugin);
    expect(result).toBe('http://127.0.0.1:8080/plugin/custom-dir/icon.svg');
  });

  it('returns undefined when plugin server port fails', async () => {
    mockInvoke.mockRejectedValue(new Error('port unavailable'));
    const consoleSpy = vi.spyOn(console, 'error').mockImplementation(() => {});
    const plugin: PluginIconInfo = { id: 'test', icon: 'icon.svg' };
    const result = await getPluginIconUrl(plugin);
    expect(result).toBeUndefined();
    consoleSpy.mockRestore();
  });

  it('tries default icon names when no icon specified', async () => {
    mockInvoke.mockResolvedValue(8080);
    const plugin: PluginIconInfo = { id: 'my-plugin' };

    mockFetch
      .mockRejectedValueOnce(new Error('not found'))
      .mockRejectedValueOnce(new Error('not found'))
      .mockRejectedValueOnce(new Error('not found'))
      .mockResolvedValueOnce({ ok: true });

    const result = await getPluginIconUrl(plugin);
    expect(result).toBe('http://127.0.0.1:8080/plugin/my-plugin/icon.jpeg');
    expect(mockFetch).toHaveBeenCalledTimes(4);
  });

  it('returns undefined when no icon found', async () => {
    mockInvoke.mockResolvedValue(8080);
    const plugin: PluginIconInfo = { id: 'my-plugin' };
    mockFetch.mockRejectedValue(new Error('not found'));

    const consoleSpy = vi.spyOn(console, 'error').mockImplementation(() => {});
    const result = await getPluginIconUrl(plugin);
    expect(result).toBeUndefined();
    expect(mockFetch).toHaveBeenCalledTimes(4);
    consoleSpy.mockRestore();
  });
});
