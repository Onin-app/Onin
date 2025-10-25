/**
 * Plugin Settings API
 * 
 * Provides methods to define and read plugin settings configured by users.
 * Settings are defined programmatically using useSettingsSchema() and can be
 * configured through the plugin settings UI.
 * 
 * @module api/settings
 */

import { invoke } from '../core/ipc';
import { createError } from '../types/errors';

/**
 * JSON-serializable value type
 */
export type JsonValue =
  | string
  | number
  | boolean
  | null
  | JsonValue[]
  | { [key: string]: JsonValue };

/**
 * Settings values object
 */
export type SettingsValues = Record<string, JsonValue>;

/**
 * Option for enum/select type settings
 */
export interface SettingOption {
  label: string;
  value: string;
}

// Base properties for all field types
interface BaseSettingField {
  key: string;
  label: string;
  description?: string;
  required?: boolean;
}

// Text input field
export interface TextSettingField extends BaseSettingField {
  type: 'text';
  defaultValue?: string;
  placeholder?: string;
  maxLength?: number;
  minLength?: number;
}

// Password input field
export interface PasswordSettingField extends BaseSettingField {
  type: 'password';
  defaultValue?: string;
  placeholder?: string;
  maxLength?: number;
  minLength?: number;
}

// Textarea field
export interface TextareaSettingField extends BaseSettingField {
  type: 'textarea';
  defaultValue?: string;
  placeholder?: string;
  maxLength?: number;
  minLength?: number;
}

// Number input field
export interface NumberSettingField extends BaseSettingField {
  type: 'number';
  defaultValue?: number;
  placeholder?: string;
  min?: number;
  max?: number;
  step?: number;
}

// Color picker field
export interface ColorSettingField extends BaseSettingField {
  type: 'color';
  defaultValue?: string;
}

// Date picker field
export interface DateSettingField extends BaseSettingField {
  type: 'date';
  defaultValue?: string;
}

// Time picker field
export interface TimeSettingField extends BaseSettingField {
  type: 'time';
  defaultValue?: string;
}

// Datetime picker field
export interface DatetimeSettingField extends BaseSettingField {
  type: 'datetime';
  defaultValue?: string;
}

// Slider field
export interface SliderSettingField extends BaseSettingField {
  type: 'slider';
  defaultValue?: number;
  min?: number;
  max?: number;
  step?: number;
}

// Switch field
export interface SwitchSettingField extends BaseSettingField {
  type: 'switch';
  defaultValue?: boolean;
}

// Radio button group field
export interface RadioSettingField extends BaseSettingField {
  type: 'radio';
  defaultValue?: string;
  options: SettingOption[];
}

// Select field (single)
export interface SelectSettingField extends BaseSettingField {
  type: 'select';
  defaultValue?: string;
  placeholder?: string;
  options: SettingOption[];
  multiple?: false;
}

// Select field (multiple)
export interface MultiSelectSettingField extends BaseSettingField {
  type: 'select';
  defaultValue?: string[];
  placeholder?: string;
  options: SettingOption[];
  multiple: true;
}

// Button field
export interface ButtonSettingField extends BaseSettingField {
  type: 'button';
  buttonText?: string;
  onClick?: () => void;
}

/**
 * Discriminated union of all setting field types
 */
export type SettingField =
  | TextSettingField
  | PasswordSettingField
  | TextareaSettingField
  | NumberSettingField
  | ColorSettingField
  | DateSettingField
  | TimeSettingField
  | DatetimeSettingField
  | SliderSettingField
  | SwitchSettingField
  | RadioSettingField
  | SelectSettingField
  | MultiSelectSettingField
  | ButtonSettingField;

/**
 * Legacy setting schema descriptor (deprecated)
 * @deprecated Use SettingField types instead
 */
export type SettingSchemaDesc = SettingField;

// Global settings schema storage
let settingsSchema: SettingField[] = [];

/**
 * Register settings schema for the plugin
 * 
 * This should be called once during plugin initialization to define
 * the settings that users can configure.
 * 
 * @param schema - Array of setting field descriptors
 * 
 * @example
 * ```typescript
 * import { settings } from 'baize-plugin-sdk';
 * 
 * const settingsSchema: SettingField[] = [
 *   {
 *     key: 'apiKey',
 *     label: 'API Key',
 *     type: 'password',
 *     description: 'Your API key',
 *     required: true,
 *     placeholder: 'Enter your API key'
 *   },
 *   {
 *     key: 'refreshInterval',
 *     label: 'Refresh Interval',
 *     type: 'number',
 *     description: 'Auto refresh interval in minutes',
 *     defaultValue: 30,
 *     min: 5,
 *     max: 120
 *   },
 *   {
 *     key: 'theme',
 *     label: 'Theme',
 *     type: 'select',
 *     defaultValue: 'auto',
 *     options: [
 *       { label: 'Auto', value: 'auto' },
 *       { label: 'Light', value: 'light' },
 *       { label: 'Dark', value: 'dark' }
 *     ]
 *   },
 *   {
 *     key: 'enableNotifications',
 *     label: 'Enable Notifications',
 *     type: 'switch',
 *     defaultValue: true,
 *     description: 'Show desktop notifications'
 *   }
 * ];
 * 
 * settings.useSettingsSchema(settingsSchema);
 * ```
 */
async function useSettingsSchema(schema: SettingField[]): Promise<void> {
  settingsSchema = schema;

  try {
    const pluginId = (globalThis as { __PLUGIN_ID__?: string }).__PLUGIN_ID__;
    if (!pluginId) {
      throw createError.common.unknown('Plugin ID not found in global context');
    }

    // Convert to internal format and register with host
    await invoke('register_plugin_settings_schema', {
      pluginId,
      schema: convertSchemaToInternal(schema)
    });
  } catch (error) {
    // Re-throw if already a plugin error
    if (error && typeof error === 'object' && error !== null && 'name' in error && error.name === 'PluginError') {
      throw error;
    }
    const errorMessage = error instanceof Error ? error.message : String(error);
    throw createError.common.unknown(`Failed to register settings schema: ${errorMessage}`);
  }
}

/**
 * Convert schema to internal format
 */
function convertSchemaToInternal(schema: SettingField[]) {
  return {
    fields: schema
  };
}

/**
 * Get all settings for the current plugin
 * 
 * @returns Promise resolving to settings object
 * @throws {PluginError} If settings cannot be retrieved
 * 
 * @example
 * ```typescript
 * import { settings } from 'baize-plugin-sdk';
 * 
 * const config = await settings.getAll();
 * console.log('API Key:', config.apiKey);
 * console.log('Refresh Interval:', config.refreshInterval);
 * ```
 */
async function getAll<T extends SettingsValues = SettingsValues>(): Promise<T> {
  try {
    const pluginId = (globalThis as { __PLUGIN_ID__?: string }).__PLUGIN_ID__;
    if (!pluginId) {
      throw createError.common.unknown('Plugin ID not found in global context');
    }

    const result = await invoke<SettingsValues>('get_plugin_settings', { pluginId });

    // Merge with default values from schema
    const merged: SettingsValues = { ...result };
    for (const field of settingsSchema) {
      if (merged[field.key] === undefined && 'defaultValue' in field && field.defaultValue !== undefined) {
        merged[field.key] = field.defaultValue;
      }
    }

    return merged as T;
  } catch (error) {
    // Re-throw if already a plugin error
    if (error && typeof error === 'object' && error !== null && 'name' in error && error.name === 'PluginError') {
      throw error;
    }
    throw createError.common.unknown(`Failed to get plugin settings: ${error}`);
  }
}

/**
 * Get a specific setting value
 * 
 * @param key - Setting key
 * @param defaultValue - Default value if setting is not found
 * @returns Promise resolving to setting value
 * 
 * @example
 * ```typescript
 * import { settings } from 'baize-plugin-sdk';
 * 
 * const apiKey = await settings.get<string>('apiKey');
 * const timeout = await settings.get<number>('timeout', 30);
 * ```
 */
async function get<T extends JsonValue = JsonValue>(key: string, defaultValue?: T): Promise<T | undefined> {
  try {
    const allSettings = await getAll();
    const value = allSettings[key];
    if (value !== undefined) {
      // Type assertion is safe here because:
      // 1. Values come from JSON serialization/deserialization
      // 2. JsonValue type constraint ensures compatibility
      // 3. Caller is responsible for providing correct type parameter
      return value as T;
    }
    return defaultValue;
  } catch (error) {
    if (defaultValue !== undefined) {
      return defaultValue;
    }
    throw error;
  }
}

/**
 * Get the registered settings schema
 * 
 * @returns The current settings schema
 */
function getSchema(): SettingField[] {
  return settingsSchema;
}

/**
 * Settings API namespace
 */
export const settings = {
  useSettingsSchema,
  getAll,
  get,
  getSchema,
};
