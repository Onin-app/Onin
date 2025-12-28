import { beforeEach, vi } from 'vitest';
import { resetMocks, setupMockResponses } from './mocks/ipc';

// Mock the core modules
vi.mock('../../src/core/ipc', () => ({
  invoke: vi.fn(),
  listen: vi.fn()
}));

vi.mock('../../src/core/environment', () => ({
  getEnvironment: vi.fn().mockReturnValue('webview')
}));

// Global test setup
beforeEach(() => {
  resetMocks();
  setupMockResponses();
});