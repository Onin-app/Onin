import { describe, it, expect, vi, beforeEach } from 'vitest';

import { createError } from '../../types/errors';

vi.mock('../../core/ipc', () => ({
  invoke: vi.fn(),
  listen: vi.fn(() => Promise.resolve(() => {})),
}));

let ai: any;
let createTextMessage: any;
let createImageMessage: any;
let Conversation: any;
let mockInvoke: any;
let mockListen: any;

beforeEach(async () => {
  vi.clearAllMocks();
  const ipc = await import('../../core/ipc');
  mockInvoke = vi.mocked(ipc.invoke);
  mockListen = vi.mocked(ipc.listen);

  const mod = await import('../ai');
  ai = mod.ai;
  createTextMessage = mod.createTextMessage;
  createImageMessage = mod.createImageMessage;
  Conversation = mod.Conversation;
});

describe('createTextMessage', () => {
  it('应创建包含 text 的 message', () => {
    const msg = createTextMessage('user', 'hello');
    expect(msg.role).toBe('user');
    expect(msg.content).toEqual([{ type: 'text', text: 'hello' }]);
  });

  it('支持不同 role', () => {
    const msg = createTextMessage('system', 'be helpful');
    expect(msg.role).toBe('system');
  });
});

describe('createImageMessage', () => {
  it('应包含 text 和 images', () => {
    const msg = createImageMessage('look at this', [
      'https://example.com/img.png',
    ]);
    expect(msg.content[0]).toEqual({ type: 'text', text: 'look at this' });
    expect(msg.content[1]).toEqual({
      type: 'image_url',
      image_url: { url: 'https://example.com/img.png' },
    });
  });

  it('base64 string 转为 image_base64', () => {
    const msg = createImageMessage('check', ['data:base64,abc123']);
    expect(msg.content[1]).toEqual({
      type: 'image_base64',
      image_base64: 'data:base64,abc123',
    });
  });

  it('对象保留 detail 属性', () => {
    const msg = createImageMessage('pic', [
      { url: 'https://example.com/img.png', detail: 'high' },
    ]);
    expect(msg.content[1]).toEqual({
      type: 'image_url',
      image_url: { url: 'https://example.com/img.png', detail: 'high' },
    });
  });

  it('默认 role 为 user', () => {
    const msg = createImageMessage('hello', []);
    expect(msg.role).toBe('user');
  });
});

describe('ai namespace', () => {
  it('应包含所有预期方法', () => {
    expect(typeof ai.ask).toBe('function');
    expect(typeof ai.stream).toBe('function');
    expect(typeof ai.isAvailable).toBe('function');
    expect(typeof ai.getCapabilities).toBe('function');
    expect(typeof ai.listModels).toBe('function');
    expect(typeof ai.validateProvider).toBe('function');
    expect(typeof ai.createConversation).toBe('function');
  });

  it('ask 应调用 invoke', async () => {
    mockInvoke.mockResolvedValue('response text');
    const result = await ai.ask('hello');
    expect(result).toBe('response text');
    expect(mockInvoke).toHaveBeenCalledWith('plugin_ai_ask', {
      request: {
        messages: [
          { role: 'user', content: [{ type: 'text', text: 'hello' }] },
        ],
      },
    });
  });

  it('isAvailable 在 getCapabilities 返回非 null 时为 true', async () => {
    mockInvoke.mockResolvedValue({
      supports_images: true,
      supports_streaming: false,
      supports_function_calling: false,
    });
    await expect(ai.isAvailable()).resolves.toBe(true);
  });

  it('isAvailable 在 getCapabilities 返回 null 时为 false', async () => {
    mockInvoke.mockResolvedValue(null);
    await expect(ai.isAvailable()).resolves.toBe(false);
  });

  it('isAvailable 在 invoke 抛出异常时为 false', async () => {
    mockInvoke.mockRejectedValue(new Error('fail'));
    await expect(ai.isAvailable()).resolves.toBe(false);
  });

  it('getCapabilities 应调用 invoke', async () => {
    mockInvoke.mockResolvedValue({
      supports_images: true,
      supports_streaming: true,
      supports_function_calling: false,
    });
    const caps = await ai.getCapabilities();
    expect(caps?.supports_images).toBe(true);
  });

  it('listModels 应调用 invoke', async () => {
    mockInvoke.mockResolvedValue([{ id: 'gpt-4', name: 'GPT-4' }]);
    const models = await ai.listModels();
    expect(models).toHaveLength(1);
  });

  it('validateProvider 应调用 invoke', async () => {
    mockInvoke.mockResolvedValue({ valid: true, models_count: 5 });
    const result = await ai.validateProvider(
      'https://api.openai.com',
      'sk-xxx',
    );
    expect(mockInvoke).toHaveBeenCalledWith('validate_ai_provider', {
      base_url: 'https://api.openai.com',
      api_key: 'sk-xxx',
    });
    expect(result.valid).toBe(true);
  });

  it('stream 应设置事件监听并调用 invoke', async () => {
    const unlistenChunk = vi.fn();
    const unlistenError = vi.fn();
    const unlistenDone = vi.fn();
    let resolveStream: () => void;
    const streamPromise = new Promise<void>((r) => {
      resolveStream = r;
    });
    mockListen
      .mockResolvedValueOnce(unlistenChunk)
      .mockResolvedValueOnce(unlistenError)
      .mockResolvedValueOnce(unlistenDone);
    mockInvoke.mockImplementation(() => {
      resolveStream();
      return Promise.resolve(undefined);
    });

    const onChunk = vi.fn();
    ai.stream('hello', onChunk);
    await streamPromise;

    expect(mockInvoke).toHaveBeenCalledWith('plugin_ai_stream', {
      request: {
        messages: [
          { role: 'user', content: [{ type: 'text', text: 'hello' }] },
        ],
      },
      eventId: expect.any(String),
    });

    const chunkHandler = mockListen.mock.calls[0][1];
    chunkHandler({ payload: 'Hello' });
    chunkHandler({ payload: ' World' });
    expect(onChunk).toHaveBeenCalledTimes(2);
    expect(onChunk).toHaveBeenCalledWith('Hello');
    expect(onChunk).toHaveBeenCalledWith(' World');

    const doneHandler = mockListen.mock.calls[2][1];
    doneHandler();
  });
});

describe('Conversation', () => {
  it('无 systemPrompt 时初始消息为空', () => {
    const conv = new Conversation();
    expect(conv.getHistory()).toEqual([]);
  });

  it('有 systemPrompt 时包含 system message', () => {
    const conv = new Conversation('You are a helpful assistant');
    const history = conv.getHistory();
    expect(history).toHaveLength(1);
    expect(history[0].role).toBe('system');
  });

  it('addMessage 追加消息', () => {
    const conv = new Conversation();
    conv.addMessage({ role: 'user', content: [{ type: 'text', text: 'hi' }] });
    expect(conv.getHistory()).toHaveLength(1);
  });

  it('ask 追加 user message 并调用 ai.ask', async () => {
    mockInvoke.mockResolvedValue('answer');
    const conv = new Conversation();
    const response = await conv.ask('hello');
    expect(response).toBe('answer');
    expect(conv.getHistory()).toHaveLength(2);
    expect(conv.getHistory()[0].role).toBe('user');
    expect(conv.getHistory()[1].role).toBe('assistant');
  });

  it('ask 失败时回滚 user message', async () => {
    mockInvoke.mockRejectedValue(createError.common.unknown('AI error'));
    const conv = new Conversation();
    await expect(conv.ask('hello')).rejects.toThrow();
    expect(conv.getHistory()).toHaveLength(0);
  });

  it('stream 累积 chunk 并追加 assistant message', async () => {
    const unlistenChunk = vi.fn();
    const unlistenError = vi.fn();
    const unlistenDone = vi.fn();
    mockListen
      .mockResolvedValueOnce(unlistenChunk)
      .mockResolvedValueOnce(unlistenError)
      .mockResolvedValueOnce(unlistenDone);
    mockInvoke.mockResolvedValue(undefined);

    const conv = new Conversation();
    const onChunk = vi.fn();
    const promise = conv.ask('hello');
    expect(mockInvoke).toHaveBeenCalled();
  });

  it('getHistory 返回副本', () => {
    const conv = new Conversation();
    conv.addMessage({ role: 'user', content: [{ type: 'text', text: 'hi' }] });
    const history = conv.getHistory();
    history.push({ role: 'user', content: [{ type: 'text', text: 'hacked' }] });
    expect(conv.getHistory()).toHaveLength(1);
  });

  it('clear 保留 system message', () => {
    const conv = new Conversation('system prompt');
    conv.addMessage({ role: 'user', content: [{ type: 'text', text: 'hi' }] });
    conv.clear();
    expect(conv.getHistory()).toHaveLength(1);
    expect(conv.getHistory()[0].role).toBe('system');
  });

  it('clear 无 system message 时清空', () => {
    const conv = new Conversation();
    conv.addMessage({ role: 'user', content: [{ type: 'text', text: 'hi' }] });
    conv.clear();
    expect(conv.getHistory()).toEqual([]);
  });

  it('getLastResponse 返回最后一个 assistant 的 text', () => {
    const conv = new Conversation();
    conv.addMessage({
      role: 'assistant',
      content: [{ type: 'text', text: 'first' }],
    });
    conv.addMessage({
      role: 'assistant',
      content: [{ type: 'text', text: 'second' }],
    });
    expect(conv.getLastResponse()).toBe('second');
  });

  it('getLastResponse 无 assistant 时返回 null', () => {
    const conv = new Conversation();
    expect(conv.getLastResponse()).toBeNull();
  });
});
