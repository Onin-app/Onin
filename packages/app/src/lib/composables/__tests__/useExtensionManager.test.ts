import { describe, it, expect, vi, beforeEach } from 'vitest';

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}));

import { useExtensionManager } from '../useExtensionManager.svelte';
import { invoke } from '@tauri-apps/api/core';
import type { ExtensionPreview } from '../useExtensionManager.svelte';

const mockInvoke = vi.mocked(invoke);

describe('useExtensionManager', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('should initialize with null preview', () => {
    const em = useExtensionManager();
    expect(em.state.currentPreview).toBeNull();
  });

  it('getPreview should return cached result for same input', async () => {
    const em = useExtensionManager();
    mockInvoke.mockResolvedValue({ title: 'Test', extension_id: 'ext-1', command_code: 'cmd-1', description: '', icon: '', copyable: '' });

    const result1 = await em.getPreview('hello');
    const result2 = await em.getPreview('hello');

    expect(mockInvoke).toHaveBeenCalledTimes(1);
    expect(result1).toEqual(result2);
  });

  it('getPreview should return null on invoke error', async () => {
    const em = useExtensionManager();
    console.error = vi.fn();
    mockInvoke.mockRejectedValue(new Error('fail'));

    const result = await em.getPreview('hello');
    expect(result).toBeNull();
    expect(em.state.currentPreview).toBeNull();
  });

  it('getPreview should update currentPreview state', async () => {
    const em = useExtensionManager();
    const preview: ExtensionPreview = {
      extension_id: 'ext-1',
      command_code: 'cmd-1',
      title: 'Calculator',
      description: 'Result: 42',
      icon: 'calc',
      copyable: '42',
    };
    mockInvoke.mockResolvedValue(preview);

    const result = await em.getPreview('2+2');

    expect(result).toEqual(preview);
    expect(em.state.currentPreview).toEqual(preview);
    expect(mockInvoke).toHaveBeenCalledWith('get_extension_preview', { input: '2+2' });
  });

  it('getPreviewAsItem should return null when no preview', () => {
    const em = useExtensionManager();
    expect(em.getPreviewAsItem()).toBeNull();
  });

  it('getPreviewAsItem should convert preview to LaunchableItem', async () => {
    const em = useExtensionManager();
    const preview: ExtensionPreview = {
      extension_id: 'ext-1',
      command_code: 'cmd-1',
      title: 'Calculator',
      description: 'Result: 42',
      icon: 'calc',
      copyable: '42',
    };
    mockInvoke.mockResolvedValue(preview);
    await em.getPreview('2+2');

    const item = em.getPreviewAsItem();
    expect(item).not.toBeNull();
    expect(item!.name).toBe('Calculator');
    expect(item!.icon).toBe('calc');
    expect(item!.source).toBe('Command');
    expect(item!.action).toBe('extension:ext-1:cmd-1');
    expect(item!.trigger_mode).toBe('preview');
  });

  it('getPreviewAsItem should include view_type and grid_data', async () => {
    const em = useExtensionManager();
    const preview: ExtensionPreview = {
      extension_id: 'ext-1',
      command_code: 'cmd-1',
      title: 'Emoji',
      description: 'Pick an emoji',
      icon: 'emoji',
      copyable: '',
      view_type: 'grid',
      grid_data: {
        groups: [{ name: 'Smileys', slug: 'smileys', emojis: [{ emoji: '😀', name: 'grinning' }] }],
      },
    };
    mockInvoke.mockResolvedValue(preview);
    await em.getPreview('emoji');

    const item = em.getPreviewAsItem();
    expect(item!.view_type).toBe('grid');
    expect(item!.grid_data!.groups[0].name).toBe('Smileys');
  });

  it('execute should call invoke and return copyable result', async () => {
    const em = useExtensionManager();
    mockInvoke.mockResolvedValue({ success: true, copyable: '42' });

    const result = await em.execute('ext-1', 'calc', '2+2');

    expect(result).toBe('42');
    expect(mockInvoke).toHaveBeenCalledWith('execute_extension', {
      extensionId: 'ext-1',
      commandCode: 'calc',
      input: '2+2',
    });
  });

  it('execute should return null on failure', async () => {
    const em = useExtensionManager();
    mockInvoke.mockResolvedValue({ success: false, copyable: '' });

    const result = await em.execute('ext-1', 'calc', '2+2');
    expect(result).toBeNull();
  });

  it('execute should return null on invoke error', async () => {
    const em = useExtensionManager();
    console.error = vi.fn();
    mockInvoke.mockRejectedValue(new Error('fail'));

    const result = await em.execute('ext-1', 'calc', '2+2');
    expect(result).toBeNull();
  });

  it('clearPreview should reset state', async () => {
    const em = useExtensionManager();
    mockInvoke.mockResolvedValue({ title: 'Test', extension_id: 'ext-1', command_code: 'cmd-1', description: '', icon: '', copyable: '' });
    await em.getPreview('hello');

    em.clearPreview();
    expect(em.state.currentPreview).toBeNull();
  });
});
