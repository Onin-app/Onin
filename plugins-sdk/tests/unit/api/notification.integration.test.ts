import { describe, it, expect, vi } from 'vitest';

// Mock the modules using factory functions
vi.mock('../../../src/core/ipc', () => ({
  invoke: vi.fn()
}));

vi.mock('../../../src/core/environment', async (importOriginal) => {
  const actual = await importOriginal();
  return {
    ...actual,
    getEnvironment: vi.fn()
  };
});

// Import after mocking
import { showNotification } from '../../../src/api/notification';
import { invoke } from '../../../src/core/ipc';
import { getEnvironment, RuntimeEnvironment } from '../../../src/core/environment';

// Get the mocked functions
const mockInvoke = vi.mocked(invoke);
const mockGetEnvironment = vi.mocked(getEnvironment);

// Integration test that tests the actual dispatch logic
describe('Notification API Integration', () => {
  it('should properly integrate dispatch with environment detection', async () => {
    mockInvoke.mockResolvedValue(undefined);

    // Test webview environment
    mockGetEnvironment.mockReturnValue(RuntimeEnvironment.Webview);
    
    const options = { title: 'Test', body: 'Integration test' };
    await showNotification(options);

    // In webview, should wrap options in an object
    expect(mockInvoke).toHaveBeenCalledWith('show_notification', { options });

    // Reset and test headless environment
    mockInvoke.mockClear();
    mockGetEnvironment.mockReturnValue(RuntimeEnvironment.Headless);

    await showNotification(options);

    // In headless, should pass options directly
    expect(mockInvoke).toHaveBeenCalledWith('show_notification', options);
  });
});