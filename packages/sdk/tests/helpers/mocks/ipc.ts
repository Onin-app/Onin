import { vi } from 'vitest';

// Mock IPC functions
export const mockInvoke = vi.fn();
export const mockListen = vi.fn();

// Mock environment detection
export const mockGetEnvironment = vi.fn().mockReturnValue('webview');

// Reset all mocks
export function resetMocks() {
  mockInvoke.mockReset();
  mockListen.mockReset();
  mockGetEnvironment.mockReset();
}

// Setup common mock responses
export function setupMockResponses() {
  // Default successful responses
  mockInvoke.mockResolvedValue(undefined);
}

// Mock specific error scenarios
export function mockError(error: any) {
  mockInvoke.mockRejectedValue(error);
}

// Mock specific success responses
export function mockSuccess(response?: any) {
  mockInvoke.mockResolvedValue(response);
}
