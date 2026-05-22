import { describe, it, expect, vi, beforeEach } from 'vitest';
import { fetchPlugins } from '../marketplace';

const mockInvoke = vi.fn();
const mockFetch = vi.fn();

vi.mock('@tauri-apps/api/core', () => ({
  invoke: (...args: unknown[]) => mockInvoke(...args),
}));

vi.mock('svelte-sonner', () => ({
  toast: { success: vi.fn(), error: vi.fn(), warning: vi.fn(), info: vi.fn() },
}));

global.fetch = mockFetch;

describe('marketplace API', () => {
  beforeEach(() => {
    mockInvoke.mockReset();
    mockFetch.mockReset();
  });

  it('fetches plugins with default params', async () => {
    mockInvoke.mockResolvedValue({
      marketplace_api_url: 'https://market.example.com',
    });
    mockFetch.mockResolvedValue({
      ok: true,
      json: () => Promise.resolve({ data: [], meta: { total: 0, page: 1, limit: 20, totalPages: 0 } }),
    });

    const result = await fetchPlugins();
    expect(mockInvoke).toHaveBeenCalledWith('get_app_config');
    expect(mockFetch).toHaveBeenCalledWith(
      'https://market.example.com/api/v1/plugins',
      expect.objectContaining({
        headers: expect.any(Headers),
      }),
    );
    expect(result).toEqual({ data: [], meta: { total: 0, page: 1, limit: 20, totalPages: 0 } });
  });

  it('passes query params correctly', async () => {
    mockInvoke.mockResolvedValue({
      marketplace_api_url: 'https://market.example.com',
    });
    let capturedUrl = '';
    mockFetch.mockImplementation((url: string) => {
      capturedUrl = url;
      return Promise.resolve({
        ok: true,
        json: () => Promise.resolve({ data: [], meta: { total: 0, page: 1, limit: 10, totalPages: 0 } }),
      });
    });

    await fetchPlugins({ page: 2, limit: 10, category: 'tools', keyword: 'search' });
    expect(capturedUrl).toContain('page=2');
    expect(capturedUrl).toContain('limit=10');
    expect(capturedUrl).toContain('category=tools');
    expect(capturedUrl).toContain('keyword=search');
  });

  it('throws when API URL not configured', async () => {
    mockInvoke.mockResolvedValue({ marketplace_api_url: null });
    await expect(fetchPlugins()).rejects.toThrow('Marketplace API URL not configured');
  });

  it('throws on non-ok response', async () => {
    mockInvoke.mockResolvedValue({
      marketplace_api_url: 'https://market.example.com',
    });
    mockFetch.mockResolvedValue({
      ok: false,
      statusText: 'Not Found',
    });
    await expect(fetchPlugins()).rejects.toThrow('API request failed: Not Found');
  });
});
