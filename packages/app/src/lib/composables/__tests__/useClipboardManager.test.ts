import { describe, it, expect, vi, beforeEach } from 'vitest';

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}));

vi.mock('$lib/utils/mimeTypeMap', () => ({
  inferMimeType: vi.fn((name: string) => {
    if (name.endsWith('.txt')) return 'text/plain';
    if (name.endsWith('.png')) return 'image/png';
    return 'application/octet-stream';
  }),
}));

vi.mock('$lib/utils/matchCommand', () => ({
  getMatchedCommands: vi.fn(() => []),
}));

import { useClipboardManager } from '../useClipboardManager.svelte';
import { invoke } from '@tauri-apps/api/core';

const mockInvoke = vi.mocked(invoke);

function createClipboardEvent(items: Array<{ kind: string; type: string; data?: string }>): ClipboardEvent {
  const dataTransfer = {
    items: items.map((item, i) => ({
      kind: item.kind,
      type: item.type,
      getAsFile: () => (item.kind === 'file' ? new File([''], `file-${i}.txt`) : null),
      getAsString: (cb: (s: string) => void) => {
        if (item.kind === 'string') cb(item.data || '');
      },
    })),
    getData: (_type: string) => '',
  };
  return {
    clipboardData: dataTransfer,
    preventDefault: vi.fn(),
    stopPropagation: vi.fn(),
  } as unknown as ClipboardEvent;
}

describe('useClipboardManager', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mockInvoke.mockResolvedValue(undefined);
  });

  it('should initialize with empty state', () => {
    const cm = useClipboardManager();
    expect(cm.state.attachedFiles).toEqual([]);
    expect(cm.state.attachedText).toBe('');
    expect(cm.state.showAllFiles).toBe(false);
  });

  it('handlePaste should process pasted text', async () => {
    const cm = useClipboardManager();
    const event = createClipboardEvent([
      { kind: 'string', type: 'text/plain', data: 'pasted text content' },
    ]);

    await cm.handlePaste(event);

    expect(cm.state.attachedText).toBe('pasted text content');
    expect(cm.state.attachedFiles).toEqual([]);
    expect(event.preventDefault).toHaveBeenCalled();
  });

  it('handlePaste should process pasted files', async () => {
    const cm = useClipboardManager();
    const event = createClipboardEvent([
      { kind: 'file', type: 'image/png' },
      { kind: 'string', type: 'text/plain', data: 'text' },
    ]);

    await cm.handlePaste(event);

    expect(cm.state.attachedFiles).toHaveLength(1);
    expect(cm.state.attachedText).toBe('');
  });

  it('handlePaste should return early if no file or text', async () => {
    const cm = useClipboardManager();
    const preventDefault = vi.fn();
    const event = {
      clipboardData: { items: [{ kind: 'string', type: 'text/html' }] },
      preventDefault,
    } as unknown as ClipboardEvent;

    await cm.handlePaste(event);

    expect(preventDefault).not.toHaveBeenCalled();
  });

  it('handlePaste should skip items without clipboardData', async () => {
    const cm = useClipboardManager();
    const event = { clipboardData: null } as unknown as ClipboardEvent;

    await cm.handlePaste(event);

    expect(cm.state.attachedText).toBe('');
  });

  it('handleDrop should add files', () => {
    const cm = useClipboardManager();
    const files = [new File([''], 'dropped.txt')];
    const event = {
      preventDefault: vi.fn(),
      dataTransfer: { files },
    } as unknown as DragEvent;

    cm.handleDrop(event);

    expect(cm.state.attachedFiles).toHaveLength(1);
    expect(cm.state.attachedFiles[0].name).toBe('dropped.txt');
  });

  it('handleDrop should append to existing files', () => {
    const cm = useClipboardManager();
    cm.state.attachedFiles = [new File([''], 'existing.txt')];

    const event = {
      preventDefault: vi.fn(),
      dataTransfer: { files: [new File([''], 'new.txt')] },
    } as unknown as DragEvent;

    cm.handleDrop(event);

    expect(cm.state.attachedFiles).toHaveLength(2);
  });

  it('handleDragOver should prevent default', () => {
    const cm = useClipboardManager();
    const preventDefault = vi.fn();
    cm.handleDragOver({ preventDefault } as unknown as DragEvent);
    expect(preventDefault).toHaveBeenCalled();
  });

  it('toggleShowAllFiles should toggle state', () => {
    const cm = useClipboardManager();
    expect(cm.state.showAllFiles).toBe(false);
    cm.toggleShowAllFiles();
    expect(cm.state.showAllFiles).toBe(true);
    cm.toggleShowAllFiles();
    expect(cm.state.showAllFiles).toBe(false);
  });

  it('removeFile should remove file by index', () => {
    const cm = useClipboardManager();
    cm.state.attachedFiles = [
      new File([''], 'a.txt'),
      new File([''], 'b.txt'),
      new File([''], 'c.txt'),
    ];

    cm.removeFile(1);

    expect(cm.state.attachedFiles).toHaveLength(2);
    expect(cm.state.attachedFiles[0].name).toBe('a.txt');
    expect(cm.state.attachedFiles[1].name).toBe('c.txt');
  });

  it('removeFile should hide showAllFiles when only 1 left', () => {
    const cm = useClipboardManager();
    cm.state.attachedFiles = [
      new File([''], 'a.txt'),
      new File([''], 'b.txt'),
    ];
    cm.state.showAllFiles = true;

    cm.removeFile(0);
    expect(cm.state.showAllFiles).toBe(false);
  });

  it('clearAttachments should reset state', () => {
    const cm = useClipboardManager();
    cm.state.attachedFiles = [new File([''], 'file.txt')];
    cm.state.attachedText = 'text';
    cm.state.showAllFiles = true;

    cm.clearAttachments();

    expect(cm.state.attachedFiles).toEqual([]);
    expect(cm.state.attachedText).toBe('');
    expect(cm.state.showAllFiles).toBe(false);
  });

  it('editTextAttachment should call callback and clear text', () => {
    const cm = useClipboardManager();
    cm.state.attachedText = 'editable text';
    const callback = vi.fn();

    cm.editTextAttachment(callback);

    expect(callback).toHaveBeenCalledWith('editable text');
    expect(cm.state.attachedText).toBe('');
  });

  it('autoPasteClipboard should populate files from invoke result', async () => {
    const cm = useClipboardManager();
    mockInvoke
      .mockResolvedValueOnce({
        files: [{ path: '/tmp/file.txt', name: 'file.txt', is_directory: false }],
        timestamp: Math.floor(Date.now() / 1000),
      })
      .mockResolvedValueOnce({ auto_paste_time_limit: 30 });

    await cm.autoPasteClipboard();

    expect(cm.state.attachedFiles).toHaveLength(1);
    expect(cm.state.attachedText).toBe('');
  });

  it('autoPasteClipboard should populate text from invoke result', async () => {
    const cm = useClipboardManager();
    mockInvoke
      .mockResolvedValueOnce({
        text: 'clipboard text content',
        timestamp: Math.floor(Date.now() / 1000),
      })
      .mockResolvedValueOnce({ auto_paste_time_limit: 30 });

    await cm.autoPasteClipboard();

    expect(cm.state.attachedText).toBe('clipboard text content');
    expect(cm.state.attachedFiles).toEqual([]);
  });

  it('autoPasteClipboard should respect time limit', async () => {
    const cm = useClipboardManager();
    mockInvoke
      .mockResolvedValueOnce({
        text: 'old clipboard content',
        timestamp: 0,
      })
      .mockResolvedValueOnce({ auto_paste_time_limit: 5 });

    await cm.autoPasteClipboard();

    expect(cm.state.attachedText).toBe('');
    expect(cm.state.attachedFiles).toEqual([]);
  });

  it('autoPasteClipboard should handle directory files', async () => {
    const cm = useClipboardManager();
    mockInvoke
      .mockResolvedValueOnce({
        files: [
          { path: '/tmp/mydir', name: 'mydir', is_directory: true },
        ],
        timestamp: Math.floor(Date.now() / 1000),
      })
      .mockResolvedValueOnce({ auto_paste_time_limit: 30 });

    await cm.autoPasteClipboard();

    expect(cm.state.attachedFiles).toHaveLength(1);
    expect((cm.state.attachedFiles[0] as any).path).toBe('/tmp/mydir');
  });

  it('autoPasteClipboard should handle errors gracefully', async () => {
    const cm = useClipboardManager();
    console.error = vi.fn();
    mockInvoke.mockRejectedValue(new Error('fail'));

    await cm.autoPasteClipboard();

    expect(console.error).toHaveBeenCalled();
  });
});
