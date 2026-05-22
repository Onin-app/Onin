import { describe, it, expect, vi, beforeEach } from 'vitest';

vi.mock('../../core/ipc', () => ({
  invoke: vi.fn(),
  listen: vi.fn(() => Promise.resolve(() => {})),
}));

let settings: any;
let mockInvoke: any;
let mockListen: any;

beforeEach(async () => {
  vi.clearAllMocks();
  vi.unstubAllGlobals();

  const ipc = await import('../../core/ipc');
  mockInvoke = vi.mocked(ipc.invoke);
  mockListen = vi.mocked(ipc.listen);

  const mod = await import('../settings');
  settings = mod.settings;
  vi.stubGlobal('__PLUGIN_ID__', 'test-plugin');
});

function makeField(overrides: Record<string, any> = {}) {
  return {
    key: 'apiKey',
    label: 'API Key',
    type: 'text' as const,
    defaultValue: 'default-key',
    ...overrides,
  };
}

describe('getSchema', () => {
  it('初始返回空数组', () => {
    expect(settings.getSchema()).toEqual([]);
  });
});

describe('useSettingsSchema', () => {
  it('应设置 schema 并调用 invoke', async () => {
    const schema = [makeField()];
    await settings.useSettingsSchema(schema);
    expect(settings.getSchema()).toEqual(schema);
    expect(mockInvoke).toHaveBeenCalledWith('register_plugin_settings_schema', {
      pluginId: 'test-plugin',
      schema: { fields: schema },
    });
  });

  it('pluginId 缺失时抛出 PluginError', async () => {
    vi.stubGlobal('__PLUGIN_ID__', undefined);
    await expect(
      settings.useSettingsSchema([makeField()]),
    ).rejects.toMatchObject({
      name: 'PluginError',
    });
  });

  it('非 PluginError 包装为 PluginError', async () => {
    mockInvoke.mockRejectedValue(new Error('network fail'));
    await expect(
      settings.useSettingsSchema([makeField()]),
    ).rejects.toMatchObject({
      name: 'PluginError',
    });
  });
});

describe('getAll', () => {
  it('应调用 invoke 并与默认值合并', async () => {
    mockInvoke.mockResolvedValue({ apiKey: 'user-key' });
    const schema = [makeField()];
    await settings.useSettingsSchema(schema);

    const result = await settings.getAll();
    expect(result.apiKey).toBe('user-key');
    expect(mockInvoke).toHaveBeenCalledWith('get_plugin_settings', {
      pluginId: 'test-plugin',
    });
  });

  it('schema 中缺失的值应使用默认值', async () => {
    mockInvoke.mockResolvedValue({});
    const schema = [makeField()];
    await settings.useSettingsSchema(schema);

    const result = await settings.getAll();
    expect(result.apiKey).toBe('default-key');
  });

  it('pluginId 缺失时抛出 PluginError', async () => {
    vi.stubGlobal('__PLUGIN_ID__', undefined);
    await expect(settings.getAll()).rejects.toMatchObject({
      name: 'PluginError',
    });
  });
});

describe('get', () => {
  it('应返回指定 key 的值', async () => {
    mockInvoke.mockResolvedValue({ name: 'test' });
    await settings.useSettingsSchema([]);

    const value = await settings.get('name');
    expect(value).toBe('test');
  });

  it('key 不存在时返回 defaultValue', async () => {
    mockInvoke.mockResolvedValue({});
    await settings.useSettingsSchema([]);

    const value = await settings.get('missing', 'fallback');
    expect(value).toBe('fallback');
  });

  it('key 不存在且无 defaultValue 时抛出', async () => {
    mockInvoke
      .mockResolvedValueOnce(undefined)
      .mockRejectedValue(new Error('not found'));
    await settings.useSettingsSchema([]);

    await expect(settings.get('missing')).rejects.toThrow();
  });
});

describe('onChange', () => {
  it('应监听 plugin-settings-changed 事件', async () => {
    await settings.onChange(vi.fn());
    expect(mockListen).toHaveBeenCalledWith(
      'plugin-settings-changed',
      expect.any(Function),
    );
  });

  it('应过滤其他 plugin 的事件', async () => {
    const callback = vi.fn();
    mockInvoke.mockResolvedValue({ key: 'val' });
    await settings.useSettingsSchema([]);

    await settings.onChange(callback);
    const handler = mockListen.mock.calls[0][1];

    handler({
      payload: { pluginId: 'other-plugin', settings: { key: 'val' } },
    });
    expect(callback).not.toHaveBeenCalled();

    handler({ payload: { pluginId: 'test-plugin', settings: { key: 'val' } } });
    expect(callback).toHaveBeenCalled();
  });

  it('应与默认值合并后调用 callback', async () => {
    const callback = vi.fn();
    await settings.useSettingsSchema([makeField()]);

    await settings.onChange(callback);
    const handler = mockListen.mock.calls[0][1];
    handler({ payload: { pluginId: 'test-plugin', settings: {} } });

    expect(callback).toHaveBeenCalledWith({ apiKey: 'default-key' });
  });

  it('pluginId 缺失时抛出 PluginError', async () => {
    vi.stubGlobal('__PLUGIN_ID__', undefined);
    await expect(settings.onChange(vi.fn())).rejects.toMatchObject({
      name: 'PluginError',
    });
  });
});
