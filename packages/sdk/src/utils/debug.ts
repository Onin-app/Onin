import { invoke } from '../core/ipc';
import { getEnvironment } from '../core/environment';

/**
 * SDK information and debugging tools
 * @fileoverview Provides debugging utilities and SDK information
 */
export const debug = {
  version: "0.0.1",
  getEnvironment,
  getRuntimeInfo: () => ({
    timestamp: Date.now(),
    userAgent: typeof navigator !== 'undefined' ? navigator.userAgent : 'Deno Runtime',
    platform: typeof navigator !== 'undefined' ? navigator.platform : 'Unknown'
  }),
  async testConnection() {
    try {
      // Test basic invoke connection
      const result = await invoke('plugin_test_connection', {});
      return { success: true, result };
    } catch (error) {
      return { success: false, error: error instanceof Error ? error.message : String(error) };
    }
  }
};