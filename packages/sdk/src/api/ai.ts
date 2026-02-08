import { invoke, listen } from '../core/ipc';

export interface ChatRequest {
    model?: string;
    messages: ChatMessage[];
    temperature?: number;
    max_tokens?: number;
}

export interface ChatMessage {
    role: 'system' | 'user' | 'assistant';
    content: string;
}

/**
 * Send a chat request to the configured AI provider.
 * @param prompt - The user prompt or conversation history
 * @param options - Additional options like model override
 * @returns The AI response text
 */
async function ask(prompt: string | ChatMessage[], options: Partial<ChatRequest> = {}): Promise<string> {
    let messages: ChatMessage[];
    if (typeof prompt === 'string') {
        messages = [{ role: 'user', content: prompt }];
    } else {
        messages = prompt;
    }

    const request: ChatRequest = {
        messages,
        ...options
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
    options: Partial<ChatRequest> = {}
): Promise<void> {
    let messages: ChatMessage[];
    if (typeof prompt === 'string') {
        messages = [{ role: 'user', content: prompt }];
    } else {
        messages = prompt;
    }

    const request: ChatRequest = {
        messages,
        ...options
    };

    // Generate a unique event ID for this stream session
    const eventId = `ai_stream_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;

    return new Promise<void>(async (resolve, reject) => {
        // Setup listeners
        const unlistenChunk = await listen<string>(eventId, (event) => {
            onChunk(event.payload);
        });

        const unlistenError = await listen<string>(`${eventId}_error`, (event) => {
            cleanup();
            reject(new Error(event.payload));
        });

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
 * AI namespace for interacting with configured AI providers.
 * 
 * @example
 * ```typescript
 * // Simple question
 * const answer = await ai.ask("What is the capital of France?");
 * 
 * // With conversation history
 * const response = await ai.ask([
 *   { role: 'system', content: 'You are a helpful assistant.' },
 *   { role: 'user', content: 'Hello!' }
 * ]);
 * 
 * // Streaming response
 * await ai.stream("Tell me a story", (chunk) => {
 *   console.log(chunk);
 * });
 * ```
 */
export const ai = {
    ask,
    stream,
};
