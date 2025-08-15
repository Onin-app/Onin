/**
 * API Communication Layer for Plugin-App Communication
 * 
 * This module provides the communication bridge between plugins and the main application
 * using Tauri's invoke system with error handling and retry mechanisms.
 */

import { invoke } from '@tauri-apps/api/core';
import { PluginError, PluginErrorCode } from './types';

/**
 * Configuration for API calls
 */
export interface ApiCallConfig {
  /** Maximum number of retry attempts */
  maxRetries?: number;
  /** Delay between retries in milliseconds */
  retryDelay?: number;
  /** Timeout for individual API calls in milliseconds */
  timeout?: number;
  /** Whether to log API calls for debugging */
  debug?: boolean;
}

/**
 * Default configuration for API calls
 */
const DEFAULT_CONFIG: Required<ApiCallConfig> = {
  maxRetries: 3,
  retryDelay: 1000,
  timeout: 5000,
  debug: false
};

/**
 * API call result wrapper
 */
export interface ApiResult<T = any> {
  success: boolean;
  data?: T;
  error?: string;
  code?: string;
}

/**
 * Communication bridge class that handles all plugin-app communication
 */
export class CommunicationBridge {
  private config: Required<ApiCallConfig>;
  private pluginId: string;

  constructor(pluginId: string, config: ApiCallConfig = {}) {
    this.pluginId = pluginId;
    this.config = { ...DEFAULT_CONFIG, ...config };
  }

  /**
   * Make a Tauri invoke call with error handling and retry logic
   * @param command - Tauri command name
   * @param args - Command arguments
   * @param config - Override configuration for this call
   * @returns Promise with the result
   */
  async invoke<T = any>(
    command: string,
    args: Record<string, any> = {},
    config?: Partial<ApiCallConfig>
  ): Promise<T> {
    const callConfig = { ...this.config, ...config };
    const fullCommand = `plugin_${command}`;
    
    // Add plugin ID to all calls for security and isolation
    const argsWithPlugin = {
      ...args,
      plugin_id: this.pluginId
    };

    if (callConfig.debug) {
      console.log(`[Plugin SDK] Invoking ${fullCommand} with args:`, argsWithPlugin);
    }

    let lastError: Error | null = null;
    
    for (let attempt = 0; attempt <= callConfig.maxRetries; attempt++) {
      try {
        // Create a timeout promise
        const timeoutPromise = new Promise<never>((_, reject) => {
          setTimeout(() => {
            reject(new PluginError(
              `API call timeout after ${callConfig.timeout}ms`,
              PluginErrorCode.API_CALL_FAILED,
              this.pluginId
            ));
          }, callConfig.timeout);
        });

        // Race between the actual call and timeout
        const result = await Promise.race([
          invoke<ApiResult<T>>(fullCommand, argsWithPlugin),
          timeoutPromise
        ]);

        // Handle API result format
        if (typeof result === 'object' && result !== null && 'success' in result) {
          const apiResult = result as ApiResult<T>;
          if (!apiResult.success) {
            throw new PluginError(
              apiResult.error || 'API call failed',
              apiResult.code as PluginErrorCode || PluginErrorCode.API_CALL_FAILED,
              this.pluginId
            );
          }
          return apiResult.data as T;
        }

        // Direct result (for backward compatibility)
        return result as T;

      } catch (error) {
        lastError = error instanceof Error ? error : new Error(String(error));
        
        if (callConfig.debug) {
          console.warn(`[Plugin SDK] Attempt ${attempt + 1} failed for ${fullCommand}:`, lastError.message);
        }

        // Don't retry on certain error types
        if (error instanceof PluginError) {
          if (error.code === PluginErrorCode.PERMISSION_DENIED || 
              error.code === PluginErrorCode.NOT_FOUND) {
            throw error;
          }
        }

        // If this was the last attempt, throw the error
        if (attempt === callConfig.maxRetries) {
          break;
        }

        // Wait before retrying
        await this.delay(callConfig.retryDelay * (attempt + 1)); // Exponential backoff
      }
    }

    // All retries failed
    throw new PluginError(
      `API call failed after ${callConfig.maxRetries + 1} attempts: ${lastError?.message}`,
      PluginErrorCode.API_CALL_FAILED,
      this.pluginId
    );
  }

  /**
   * Utility method to delay execution
   * @param ms - Milliseconds to delay
   */
  private delay(ms: number): Promise<void> {
    return new Promise(resolve => setTimeout(resolve, ms));
  }

  /**
   * Update the configuration for this bridge
   * @param config - New configuration options
   */
  updateConfig(config: Partial<ApiCallConfig>): void {
    this.config = { ...this.config, ...config };
  }

  /**
   * Get the current plugin ID
   */
  getPluginId(): string {
    return this.pluginId;
  }

  /**
   * Check if the communication bridge is available (Tauri is loaded)
   */
  isAvailable(): boolean {
    try {
      return typeof invoke === 'function';
    } catch {
      return false;
    }
  }
}

/**
 * Create a communication bridge for a plugin
 * @param pluginId - Unique plugin identifier
 * @param config - Configuration options
 * @returns CommunicationBridge instance
 */
export function createCommunicationBridge(
  pluginId: string, 
  config?: ApiCallConfig
): CommunicationBridge {
  return new CommunicationBridge(pluginId, config);
}

/**
 * Global communication bridge instance (can be set by the plugin system)
 */
let globalBridge: CommunicationBridge | null = null;

/**
 * Set the global communication bridge
 * @param bridge - Communication bridge instance
 */
export function setGlobalBridge(bridge: CommunicationBridge): void {
  globalBridge = bridge;
}

/**
 * Get the global communication bridge
 * @returns Global bridge instance or null if not set
 */
export function getGlobalBridge(): CommunicationBridge | null {
  return globalBridge;
}

/**
 * Utility function to make API calls using the global bridge
 * @param command - Tauri command name
 * @param args - Command arguments
 * @param config - Override configuration
 * @returns Promise with the result
 */
export async function invokeApi<T = any>(
  command: string,
  args?: Record<string, any>,
  config?: Partial<ApiCallConfig>
): Promise<T> {
  if (!globalBridge) {
    throw new PluginError(
      'No global communication bridge available. Make sure the plugin is properly initialized.',
      PluginErrorCode.API_CALL_FAILED
    );
  }
  
  return globalBridge.invoke<T>(command, args, config);
}