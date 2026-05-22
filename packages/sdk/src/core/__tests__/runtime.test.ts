import { describe, it, expect, vi, beforeEach } from 'vitest';
import {
  getRuntime,
  _resetRuntimeCache,
  runtime,
  isInlineMode,
  isWindowMode,
  getPluginId,
} from '../runtime';

beforeEach(() => {
  vi.unstubAllGlobals();
  _resetRuntimeCache();
});

describe('getRuntime', () => {
  it('应从 window.__ONIN_RUNTIME__ 读取运行时信息', () => {
    vi.stubGlobal('window', {
      __ONIN_RUNTIME__: {
        mode: 'window',
        pluginId: 'my-plugin',
        version: '1.0.0',
        mainWindowLabel: 'main',
      },
      location: { search: '' },
    });
    const result = getRuntime();
    expect(result.mode).toBe('window');
    expect(result.pluginId).toBe('my-plugin');
    expect(result.version).toBe('1.0.0');
    expect(result.mainWindowLabel).toBe('main');
  });

  it('应返回缓存结果（重复调用不重新读取）', () => {
    const injected = {
      mode: 'inline',
      pluginId: 'cached',
      version: '2.0.0',
      mainWindowLabel: 'main',
    };
    vi.stubGlobal('window', {
      __ONIN_RUNTIME__: injected,
      location: { search: '' },
    });
    const first = getRuntime();
    delete (window as any).__ONIN_RUNTIME__;
    const second = getRuntime();
    expect(second.pluginId).toBe('cached');
  });

  it('无注入时从 URL mode 参数 fallback', () => {
    vi.stubGlobal('window', {
      location: { search: '?mode=inline&plugin_id=url-plugin' },
    });
    const result = getRuntime();
    expect(result.mode).toBe('inline');
    expect(result.pluginId).toBe('url-plugin');
    expect(result.version).toBe('0.0.0-dev');
  });

  it('无 URL 参数时返回 dev 默认值', () => {
    vi.stubGlobal('window', { location: { search: '' } });
    const result = getRuntime();
    expect(result.mode).toBe('window');
    expect(result.pluginId).toBe('dev-plugin');
    expect(result.version).toBe('0.0.0-dev');
  });

  it('URL mode 非 inline 时默认为 window', () => {
    vi.stubGlobal('window', {
      location: { search: '?mode=unknown' },
    });
    expect(getRuntime().mode).toBe('window');
  });

  it('__PLUGIN_ID__ 优先级高于 URL plugin_id', () => {
    vi.stubGlobal('window', {
      __PLUGIN_ID__: 'from-global',
      location: { search: '?plugin_id=from-url' },
    });
    expect(getRuntime().pluginId).toBe('from-global');
  });
});

describe('_resetRuntimeCache', () => {
  it('应清除缓存使下次调用重新读取', () => {
    vi.stubGlobal('window', {
      __ONIN_RUNTIME__: {
        mode: 'window',
        pluginId: 'old',
        version: '1.0.0',
        mainWindowLabel: 'main',
      },
      location: { search: '' },
    });
    getRuntime();
    _resetRuntimeCache();
    vi.stubGlobal('window', {
      __ONIN_RUNTIME__: {
        mode: 'inline',
        pluginId: 'new',
        version: '2.0.0',
        mainWindowLabel: 'main',
      },
      location: { search: '' },
    });
    expect(getRuntime().pluginId).toBe('new');
  });
});

describe('runtime getters', () => {
  it('应通过 getter 返回正确值', () => {
    vi.stubGlobal('window', {
      __ONIN_RUNTIME__: {
        mode: 'inline',
        pluginId: 'test',
        version: '3.0.0',
        mainWindowLabel: 'child',
      },
      location: { search: '' },
    });
    expect(runtime.mode).toBe('inline');
    expect(runtime.pluginId).toBe('test');
    expect(runtime.version).toBe('3.0.0');
    expect(runtime.mainWindowLabel).toBe('child');
  });
});

describe('isInlineMode / isWindowMode', () => {
  it('isInlineMode 在 inline 模式返回 true', () => {
    vi.stubGlobal('window', {
      __ONIN_RUNTIME__: {
        mode: 'inline',
        pluginId: 'p',
        version: '1',
        mainWindowLabel: 'main',
      },
      location: { search: '' },
    });
    expect(isInlineMode()).toBe(true);
    expect(isWindowMode()).toBe(false);
  });

  it('isWindowMode 在 window 模式返回 true', () => {
    vi.stubGlobal('window', {
      __ONIN_RUNTIME__: {
        mode: 'window',
        pluginId: 'p',
        version: '1',
        mainWindowLabel: 'main',
      },
      location: { search: '' },
    });
    expect(isWindowMode()).toBe(true);
    expect(isInlineMode()).toBe(false);
  });
});

describe('getPluginId', () => {
  it('应返回当前插件 ID', () => {
    vi.stubGlobal('window', {
      __ONIN_RUNTIME__: {
        mode: 'inline',
        pluginId: 'my-plugin',
        version: '1',
        mainWindowLabel: 'main',
      },
      location: { search: '' },
    });
    expect(getPluginId()).toBe('my-plugin');
  });
});
