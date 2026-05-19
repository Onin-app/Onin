import { describe, it, expect, vi, beforeEach } from 'vitest';

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}));

vi.mock('@tauri-apps/api/event', () => ({
  listen: vi.fn(),
}));

vi.mock('$lib/utils/fuzzyMatch', () => ({
  fuzzyMatch: vi.fn((query: string, items: any[]) => {
    if (!query) return [];
    return items.filter((item) =>
      item.name.toLowerCase().includes(query.toLowerCase()),
    );
  }),
}));

vi.mock('svelte-sonner', () => ({
  toast: { error: vi.fn() },
}));

import { useAppList } from '../useAppList.svelte';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import type { LaunchableItem } from '$lib/type';

const mockInvoke = vi.mocked(invoke);
const mockListen = vi.mocked(listen);

const sampleItems: LaunchableItem[] = [
  {
    name: 'Calculator',
    path: 'calc',
    icon: 'calc',
    icon_type: 'Iconfont',
    item_type: 'App',
    source: 'Command',
    keywords: [],
    action: 'extension:calculator:calc',
  },
  {
    name: 'Terminal',
    path: 'terminal',
    icon: 'terminal',
    icon_type: 'Iconfont',
    item_type: 'App',
    source: 'Application',
    keywords: [],
    action: 'system:open_terminal',
  },
  {
    name: 'Settings',
    path: 'settings',
    icon: 'settings',
    icon_type: 'Iconfont',
    item_type: 'App',
    source: 'Command',
    keywords: [],
  },
];

describe('useAppList', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mockInvoke.mockResolvedValue(undefined);
    mockListen.mockResolvedValue(() => {});
  });

  it('should initialize with empty state', () => {
    const al = useAppList();
    expect(al.state.originAppList).toEqual([]);
    expect(al.state.appList).toEqual([]);
    expect(al.state.selectedIndex).toBe(0);
    expect(al.state.usageStats).toEqual([]);
    expect(al.state.isRefreshing).toBe(false);
    expect(al.state.appConfig.auto_paste_time_limit).toBe(5);
  });

  it('fetchApps should populate app lists from invoke', async () => {
    const al = useAppList();
    mockInvoke.mockResolvedValue(sampleItems);

    await al.fetchApps();

    expect(mockInvoke).toHaveBeenCalledWith('get_all_launchable_items');
    expect(al.state.originAppList).toHaveLength(3);
    expect(al.state.appList).toHaveLength(3);
    expect(al.state.originAppList[0].name).toBe('Calculator');
  });

  it('fetchApps should handle errors gracefully', async () => {
    const al = useAppList();
    console.error = vi.fn();
    mockInvoke.mockRejectedValue(new Error('fail'));

    await al.fetchApps();

    expect(console.error).toHaveBeenCalled();
    expect(al.state.originAppList).toEqual([]);
  });

  it('handleInput should filter apps using fuzzyMatch', () => {
    const al = useAppList();
    al.state.originAppList = sampleItems;
    al.state.appList = sampleItems;

    al.handleInput('calc');

    expect(al.state.appList).toHaveLength(1);
    expect(al.state.appList[0].name).toBe('Calculator');
    expect(al.state.selectedIndex).toBe(0);
  });

  it('handleInput with empty string should return empty list', () => {
    const al = useAppList();
    al.state.originAppList = sampleItems;
    al.state.appList = sampleItems;

    al.handleInput('');

    expect(al.state.appList).toHaveLength(0);
  });

  it('openApp should call invoke with action when app has action', async () => {
    const al = useAppList();
    mockInvoke
      .mockResolvedValueOnce(undefined)
      .mockResolvedValueOnce([])
      .mockResolvedValueOnce([]);

    const onSuccess = vi.fn();
    await al.openApp(sampleItems[0], {}, onSuccess);

    expect(mockInvoke).toHaveBeenCalledWith('execute_command', {
      name: 'extension:calculator:calc',
      args: null,
    });
    expect(onSuccess).toHaveBeenCalled();
  });

  it('openApp should pass args when provided', async () => {
    const al = useAppList();
    mockInvoke
      .mockResolvedValueOnce(undefined)
      .mockResolvedValueOnce([])
      .mockResolvedValueOnce([]);

    const args = { input: 'hello' };
    const onSuccess = vi.fn();
    await al.openApp(sampleItems[0], args, onSuccess);

    expect(mockInvoke).toHaveBeenCalledWith('execute_command', {
      name: 'extension:calculator:calc',
      args,
    });
  });

  it('openApp should call open_app for FileCommand source', async () => {
    const al = useAppList();
    const fileItem: LaunchableItem = {
      name: 'File',
      path: '/path/to/file',
      icon: 'file',
      icon_type: 'Iconfont',
      item_type: 'File',
      source: 'FileCommand',
      keywords: [],
    };
    mockInvoke.mockResolvedValue(undefined);

    const onSuccess = vi.fn();
    await al.openApp(fileItem, {}, onSuccess);

    expect(mockInvoke).toHaveBeenCalledWith('open_app', {
      path: '/path/to/file',
    });
  });

  it('handleKeyDown ArrowDown should increment selectedIndex', () => {
    const al = useAppList();
    al.state.originAppList = sampleItems;
    al.state.appList = sampleItems;
    al.state.selectedIndex = 0;

    const container = document.createElement('div');
    container.className = 'app-list';
    for (let i = 0; i < 3; i++) {
      const el = document.createElement('div');
      container.appendChild(el);
    }
    document.body.appendChild(container);

    al.handleKeyDown(
      { key: 'ArrowDown', preventDefault: vi.fn() } as unknown as KeyboardEvent,
      sampleItems,
      vi.fn(),
    );

    expect(al.state.selectedIndex).toBe(1);
    document.body.removeChild(container);
  });

  it('handleKeyDown ArrowDown should wrap to 0 at end', () => {
    const al = useAppList();
    al.state.originAppList = sampleItems;
    al.state.appList = sampleItems;
    al.state.selectedIndex = 2;

    const container = document.createElement('div');
    container.className = 'app-list';
    for (let i = 0; i < 3; i++) {
      const el = document.createElement('div');
      container.appendChild(el);
    }
    document.body.appendChild(container);

    al.handleKeyDown(
      { key: 'ArrowDown', preventDefault: vi.fn() } as unknown as KeyboardEvent,
      sampleItems,
      vi.fn(),
    );

    expect(al.state.selectedIndex).toBe(0);
    document.body.removeChild(container);
  });

  it('handleKeyDown ArrowUp should decrement selectedIndex', () => {
    const al = useAppList();
    al.state.originAppList = sampleItems;
    al.state.appList = sampleItems;
    al.state.selectedIndex = 2;

    const container = document.createElement('div');
    container.className = 'app-list';
    for (let i = 0; i < 3; i++) {
      const el = document.createElement('div');
      container.appendChild(el);
    }
    document.body.appendChild(container);

    al.handleKeyDown(
      { key: 'ArrowUp', preventDefault: vi.fn() } as unknown as KeyboardEvent,
      sampleItems,
      vi.fn(),
    );

    expect(al.state.selectedIndex).toBe(1);
    document.body.removeChild(container);
  });

  it('handleKeyDown ArrowUp should wrap to end at 0', () => {
    const al = useAppList();
    al.state.appList = sampleItems;
    al.state.selectedIndex = 0;

    al.handleKeyDown(
      { key: 'ArrowUp', preventDefault: vi.fn() } as unknown as KeyboardEvent,
      sampleItems,
      vi.fn(),
    );

    expect(al.state.selectedIndex).toBe(2);
  });

  it('handleKeyDown Enter should call onEnter with selected item', () => {
    const al = useAppList();
    al.state.originAppList = sampleItems;
    al.state.appList = sampleItems;
    al.state.selectedIndex = 1;
    const onEnter = vi.fn();

    al.handleKeyDown(
      { key: 'Enter', preventDefault: vi.fn() } as unknown as KeyboardEvent,
      sampleItems,
      onEnter,
    );

    expect(onEnter).toHaveBeenCalledWith(sampleItems[1]);
  });

  it('handleKeyDown Tab (no shift) should act like ArrowDown', () => {
    const al = useAppList();
    al.state.appList = sampleItems;
    al.state.selectedIndex = 0;

    al.handleKeyDown(
      { key: 'Tab', shiftKey: false, preventDefault: vi.fn() } as unknown as KeyboardEvent,
      sampleItems,
      vi.fn(),
    );

    expect(al.state.selectedIndex).toBe(1);
  });

  it('handleKeyDown Tab+Shift should act like ArrowUp', () => {
    const al = useAppList();
    al.state.appList = sampleItems;
    al.state.selectedIndex = 1;

    al.handleKeyDown(
      { key: 'Tab', shiftKey: true, preventDefault: vi.fn() } as unknown as KeyboardEvent,
      sampleItems,
      vi.fn(),
    );

    expect(al.state.selectedIndex).toBe(0);
  });

  it('handleKeyDown should not call onEnter when displayList is empty', () => {
    const al = useAppList();
    const onEnter = vi.fn();

    al.handleKeyDown(
      { key: 'Enter', preventDefault: vi.fn() } as unknown as KeyboardEvent,
      [],
      onEnter,
    );

    expect(onEnter).not.toHaveBeenCalled();
  });

  it('resetSelection should set selectedIndex to 0', () => {
    const al = useAppList();
    al.state.selectedIndex = 5;
    al.resetSelection();
    expect(al.state.selectedIndex).toBe(0);
  });

  it('resetToOriginList should restore appList from originAppList', () => {
    const al = useAppList();
    al.state.originAppList = sampleItems;
    al.state.appList = [];
    al.state.selectedIndex = 3;

    al.resetToOriginList();

    expect(al.state.appList).toEqual(sampleItems);
    expect(al.state.selectedIndex).toBe(0);
  });

  it('loadConfig should fetch config and usage stats', async () => {
    const al = useAppList();
    mockInvoke
      .mockResolvedValueOnce({ auto_paste_time_limit: 10, auto_clear_time_limit: 60, sort_mode: 'frequency', enable_usage_tracking: true })
      .mockResolvedValueOnce([{ command_name: 'calc', usage_count: 5, last_used: Date.now() }]);

    await al.loadConfig();

    expect(mockInvoke).toHaveBeenCalledWith('get_app_config');
    expect(mockInvoke).toHaveBeenCalledWith('get_usage_stats');
    expect(al.state.appConfig.auto_paste_time_limit).toBe(10);
    expect(al.state.usageStats).toHaveLength(1);
  });

  it('loadConfig should handle errors gracefully', async () => {
    const al = useAppList();
    console.error = vi.fn();
    mockInvoke.mockRejectedValue(new Error('fail'));

    await al.loadConfig();

    expect(console.error).toHaveBeenCalled();
  });

  it('setupListeners should register listeners and return cleanup', async () => {
    const unlisten1 = vi.fn();
    const unlisten2 = vi.fn();
    const unlisten3 = vi.fn();
    const unlisten4 = vi.fn();
    mockListen
      .mockResolvedValueOnce(unlisten1)
      .mockResolvedValueOnce(unlisten2)
      .mockResolvedValueOnce(unlisten3)
      .mockResolvedValueOnce(unlisten4);

    const al = useAppList();
    const cleanup = await al.setupListeners();

    expect(mockListen).toHaveBeenCalledWith('apps_updated', expect.any(Function));
    expect(mockListen).toHaveBeenCalledWith('commands_ready', expect.any(Function));
    expect(mockListen).toHaveBeenCalledWith('refresh_started', expect.any(Function));
    expect(mockListen).toHaveBeenCalledWith('commands_refreshed', expect.any(Function));

    cleanup();
    expect(unlisten1).toHaveBeenCalled();
    expect(unlisten2).toHaveBeenCalled();
    expect(unlisten3).toHaveBeenCalled();
    expect(unlisten4).toHaveBeenCalled();
  });
});
