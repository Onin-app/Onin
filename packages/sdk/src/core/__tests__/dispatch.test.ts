import { describe, it, expect, vi, beforeEach } from 'vitest';
import { dispatch } from '../dispatch';

beforeEach(() => {
  vi.unstubAllGlobals();
});

describe('dispatch', () => {
  it('Webview 环境执行 webview handler', () => {
    vi.stubGlobal('window', { __TAURI_INTERNALS__: {} });
    const webviewSpy = vi.fn(() => 'webview-result');
    const headlessSpy = vi.fn();
    const result = dispatch({ webview: webviewSpy, headless: headlessSpy });
    expect(webviewSpy).toHaveBeenCalledOnce();
    expect(headlessSpy).not.toHaveBeenCalled();
    expect(result).toBe('webview-result');
  });

  it('Headless 环境执行 headless handler', () => {
    vi.stubGlobal('window', undefined);
    vi.stubGlobal('Deno', { core: {} });
    const webviewSpy = vi.fn();
    const headlessSpy = vi.fn(() => 'headless-result');
    const result = dispatch({ webview: webviewSpy, headless: headlessSpy });
    expect(headlessSpy).toHaveBeenCalledOnce();
    expect(webviewSpy).not.toHaveBeenCalled();
    expect(result).toBe('headless-result');
  });

  it('Unknown 环境抛出错误', () => {
    vi.stubGlobal('window', undefined);
    vi.stubGlobal('Deno', undefined);
    expect(() => dispatch({ webview: () => 'w', headless: () => 'h' })).toThrow(
      'Unsupported environment: unknown',
    );
  });
});
