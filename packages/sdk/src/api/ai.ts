import { invoke, listen } from '../core/ipc';

export type MessageContent =
  | { type: 'text'; text: string }
  | {
      type: 'image_url';
      image_url: { url: string; detail?: 'auto' | 'low' | 'high' };
    }
  | { type: 'image_base64'; image_base64: string; media_type?: string };

export interface ChatMessage {
  role: 'system' | 'user' | 'assistant';
  content: MessageContent[];
}

export interface ChatRequest {
  model?: string;
  messages: ChatMessage[];
  temperature?: number;
  max_tokens?: number;
  stream?: boolean;
}

export interface AIProvider {
  id: string;
  provider_type: string;
  name: string;
  display_name?: string | null;
  base_url: string;
  default_model: string | null;
}

export interface AICapabilities {
  /** Whether the provider supports image inputs (both URL and base64) */
  supports_images: boolean;
  /** Whether the provider supports streaming responses */
  supports_streaming: boolean;
  /** Whether the provider supports function calling */
  supports_function_calling: boolean;
  /** Maximum context window in tokens */
  max_context_tokens?: number;
  /** Maximum number of images allowed per message (undefined = unlimited) */
  max_images_per_message?: number;
}

export interface ModelInfo {
  id: string;
  name: string;
  description?: string;
  context_window?: number;
}

export interface ValidationResult {
  valid: boolean;
  message?: string;
  models_count?: number;
}

// Helper functions
export function createTextMessage(
  role: 'system' | 'user' | 'assistant',
  text: string,
): ChatMessage {
  return {
    role,
    content: [{ type: 'text', text }],
  };
}

export function createImageMessage(
  text: string,
  images: Array<string | { url: string; detail?: 'auto' | 'low' | 'high' }>,
): ChatMessage {
  const content: MessageContent[] = [{ type: 'text', text }];

  for (const img of images) {
    if (typeof img === 'string') {
      // Check if URL or base64 (simple check)
      if (img.startsWith('http://') || img.startsWith('https://')) {
        content.push({ type: 'image_url', image_url: { url: img } });
      } else {
        content.push({ type: 'image_base64', image_base64: img });
      }
    } else {
      content.push({ type: 'image_url', image_url: img });
    }
  }

  return { role: 'user', content };
}

/**
 * Send a chat request to the configured AI provider.
 * @param prompt - The user prompt or conversation history
 * @param options - Additional options like model override
 * @returns The AI response text
 */
async function ask(
  prompt: string | ChatMessage[],
  options: Partial<ChatRequest> = {},
): Promise<string> {
  let messages: ChatMessage[];
  if (typeof prompt === 'string') {
    messages = [createTextMessage('user', prompt)];
  } else {
    messages = prompt;
  }

  const request: ChatRequest = {
    messages,
    ...options,
  };

  return invoke<string>('plugin_ai_ask', { request });
}

/**
 * Stream a chat response from the configured AI provider.
 * @param prompt - The user prompt
 * @param onChunk - Callback for each text chunk
 * @param options - Additional options
 * @returns Promise that resolves when stream is complete
 */
async function stream(
  prompt: string | ChatMessage[],
  onChunk: (chunk: string) => void,
  options: Partial<ChatRequest> = {},
): Promise<void> {
  let messages: ChatMessage[];
  if (typeof prompt === 'string') {
    messages = [createTextMessage('user', prompt)];
  } else {
    messages = prompt;
  }

  const request: ChatRequest = {
    messages,
    ...options,
  };

  // Generate a unique event ID for this stream session
  const eventId = `ai_stream_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;

  return new Promise<void>(async (resolve, reject) => {
    // Setup listeners
    const unlistenChunk = await listen<string>(eventId, (event: any) => {
      onChunk(event.payload);
    });

    const unlistenError = await listen<string>(
      `${eventId}_error`,
      (event: any) => {
        cleanup();
        reject(new Error(event.payload));
      },
    );

    const unlistenDone = await listen<void>(`${eventId}_done`, () => {
      cleanup();
      resolve();
    });

    const cleanup = () => {
      unlistenChunk();
      unlistenError();
      unlistenDone();
    };

    // Start the stream
    try {
      await invoke('plugin_ai_stream', { request, eventId });
    } catch (e) {
      cleanup();
      reject(e);
    }
  });
}

/**
 * Check if AI functionality is available (provider configured)
 */
async function isAvailable(): Promise<boolean> {
  try {
    const caps = await getCapabilities();
    return caps !== null;
  } catch {
    return false;
  }
}

/**
 * Get capabilities of the current active provider
 */
async function getCapabilities(): Promise<AICapabilities | null> {
  return invoke<AICapabilities | null>('get_ai_capabilities', {});
}

/**
 * List available models from the current active provider
 */
async function listModels(): Promise<ModelInfo[]> {
  return invoke<ModelInfo[]>('list_ai_models', {});
}

/**
 * Validate a provider configuration.
 * Note: Mainly for settings UI, generic plugins might not need this.
 * @internal
 */
async function validateProvider(
  baseUrl: string,
  apiKey?: string,
): Promise<ValidationResult> {
  return invoke<ValidationResult>('validate_ai_provider', {
    base_url: baseUrl,
    api_key: apiKey ?? null,
  });
}

/**
 * Create a conversation manager for handling multi-turn chats
 */
function createConversation(systemPrompt?: string): Conversation {
  return new Conversation(systemPrompt);
}

/**
 * Helper class to manage conversation history
 */
export class Conversation {
  private messages: ChatMessage[] = [];

  constructor(systemPrompt?: string) {
    if (systemPrompt) {
      this.messages.push(createTextMessage('system', systemPrompt));
    }
  }

  addMessage(message: ChatMessage): void {
    this.messages.push(message);
  }

  async ask(
    prompt: string | ChatMessage,
    options?: Partial<ChatRequest>,
  ): Promise<string> {
    const userMessage =
      typeof prompt === 'string' ? createTextMessage('user', prompt) : prompt;

    this.messages.push(userMessage);

    try {
      const response = await ask(this.messages, options || {});
      this.messages.push(createTextMessage('assistant', response));
      return response;
    } catch (error) {
      // Rollback: remove the user message if request failed
      this.messages.pop();
      throw error;
    }
  }

  async stream(
    prompt: string | ChatMessage,
    onChunk: (chunk: string) => void,
    options?: Partial<ChatRequest>,
  ): Promise<void> {
    const userMessage =
      typeof prompt === 'string' ? createTextMessage('user', prompt) : prompt;

    this.messages.push(userMessage);

    let fullResponse = '';

    try {
      await stream(
        this.messages,
        (chunk) => {
          fullResponse += chunk;
          onChunk(chunk);
        },
        options || {},
      );

      this.messages.push(createTextMessage('assistant', fullResponse));
    } catch (error) {
      // Rollback: remove the user message if request failed
      this.messages.pop();
      throw error;
    }
  }

  getHistory(): ChatMessage[] {
    return [...this.messages];
  }

  clear(): void {
    const systemMessage = this.messages.find((m) => m.role === 'system');
    this.messages = systemMessage ? [systemMessage] : [];
  }

  getLastResponse(): string | null {
    for (let i = this.messages.length - 1; i >= 0; i--) {
      if (this.messages[i].role === 'assistant') {
        const content = this.messages[i].content;
        // Simple heuristic: find first text part
        const textPart = content.find((c) => c.type === 'text');
        if (textPart && textPart.type === 'text') return textPart.text;
      }
    }
    return null;
  }
}

/**
 * AI namespace for interacting with configured AI providers.
 */
export const ai = {
  ask,
  stream,
  isAvailable,
  getCapabilities,
  listModels,
  validateProvider,
  createConversation,
};
