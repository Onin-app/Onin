import { describe, it, expect, vi, beforeEach } from 'vitest';
import { getEnvironment, RuntimeEnvironment } from '../environment';

beforeEach(() => {
  vi.unstubAllGlobals();
});

describe('getEnvironment', () => {
  it('window.__TAURI_INTERNALS__ 存在时返回 Webview', () => {
    vi.stubGlobal('window', { __TAURI_INTERNALS__: {} });
    expect(getEnvironment()).toBe(RuntimeEnvironment.Webview);
  });

  it('Deno.core 存在时返回 Headless', () => {
    vi.stubGlobal('window', undefined);
    vi.stubGlobal('Deno', { core: {} });
    expect(getEnvironment()).toBe(RuntimeEnvironment.Headless);
  });

  it('两者都不存在时返回 Unknown', () => {
    vi.stubGlobal('window', undefined);
    vi.stubGlobal('Deno', undefined);
    expect(getEnvironment()).toBe(RuntimeEnvironment.Unknown);
  });

  it('Webview 优先级高于 Headless', () => {
    vi.stubGlobal('window', { __TAURI_INTERNALS__: {} });
    vi.stubGlobal('Deno', { core: {} });
    expect(getEnvironment()).toBe(RuntimeEnvironment.Webview);
  });
});
