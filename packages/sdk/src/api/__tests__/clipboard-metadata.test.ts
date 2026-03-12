/**
 * Test for clipboard metadata API v2 (Enhanced)
 */

import { describe, it, expect, beforeEach, vi } from 'vitest';

vi.mock('../../core/ipc', () => ({
  invoke: vi.fn(),
}));

vi.mock('../../core/environment', async (importOriginal) => {
  const actual = await importOriginal();
  return {
    ...actual,
    getEnvironment: vi.fn(),
  };
});

import { clipboard } from '../clipboard';
import { invoke } from '../../core/ipc';
import { getEnvironment, RuntimeEnvironment } from '../../core/environment';

const mockInvoke = vi.mocked(invoke);
const mockGetEnvironment = vi.mocked(getEnvironment);

interface MockClipboardState {
  text: string | null;
  files: Array<{ path: string; name: string; is_directory: boolean }> | null;
  image: string | null;
  timestamp: number | null;
}

let clipboardState: MockClipboardState;

function buildMetadata() {
  const now = Math.floor(Date.now() / 1000);
  const contentType = clipboardState.files?.length
    ? 'files'
    : clipboardState.image
      ? 'image'
      : clipboardState.text
        ? 'text'
        : 'empty';

  return {
    text: clipboardState.text,
    files: clipboardState.files,
    contentType,
    timestamp: clipboardState.timestamp,
    age:
      clipboardState.timestamp === null ? null : Math.max(0, now - clipboardState.timestamp),
  };
}

describe('Clipboard Metadata API v2', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mockGetEnvironment.mockReturnValue(RuntimeEnvironment.Webview);
    clipboardState = {
      text: null,
      files: null,
      image: null,
      timestamp: null,
    };

    mockInvoke.mockImplementation(async (method: string, args?: any) => {
      switch (method) {
        case 'plugin_clipboard_clear':
          clipboardState = { text: null, files: null, image: null, timestamp: Math.floor(Date.now() / 1000) };
          return undefined;
        case 'plugin_clipboard_write_text':
          clipboardState = {
            text: args.text,
            files: null,
            image: null,
            timestamp: Math.floor(Date.now() / 1000),
          };
          return undefined;
        case 'plugin_clipboard_read_text':
          return clipboardState.text;
        case 'plugin_clipboard_read_image':
          return clipboardState.image;
        case 'plugin_clipboard_get_metadata':
          return buildMetadata();
        default:
          throw new Error(`Unexpected clipboard method: ${method}`);
      }
    });
  });

  describe('Basic Metadata', () => {
    it('should get metadata with text, timestamp, and age', async () => {
      const testText = 'Test content for metadata';
      await clipboard.writeText(testText);
      const metadata = await clipboard.getMetadata();

      expect(metadata.text).toBe(testText);
      expect(metadata.contentType).toBe('text');
      expect(metadata.timestamp).toBeDefined();
      expect(metadata.age).toBeDefined();
      if (metadata.age !== null) {
        expect(metadata.age).toBeLessThan(5);
        expect(metadata.age).toBeGreaterThanOrEqual(0);
      }

      if (metadata.timestamp && metadata.age !== null) {
        const now = Math.floor(Date.now() / 1000);
        const calculatedAge = now - metadata.timestamp;
        expect(Math.abs(metadata.age - calculatedAge)).toBeLessThanOrEqual(1);
      }
    });

    it('should return empty content type when clipboard is empty', async () => {
      await clipboard.clear();
      const metadata = await clipboard.getMetadata();

      expect(metadata.contentType).toBe('empty');
      expect(metadata.text === null || metadata.text === '').toBe(true);
      expect(metadata.files).toBeNull();
    });
  });

  describe('Age Field', () => {
    it('should provide age field for convenience', async () => {
      await clipboard.writeText('Test age field');
      await new Promise(resolve => setTimeout(resolve, 1000));
      const metadata = await clipboard.getMetadata();

      expect(metadata.age).toBeGreaterThanOrEqual(1);
      if (metadata.timestamp && metadata.age !== null) {
        const now = Math.floor(Date.now() / 1000);
        const expectedAge = now - metadata.timestamp;
        expect(Math.abs(metadata.age - expectedAge)).toBeLessThanOrEqual(1);
      }
    });

    it('should allow plugins to implement time-based logic using age field', async () => {
      const testText = 'Time-sensitive content';
      await clipboard.writeText(testText);
      const metadata = await clipboard.getMetadata();

      if (metadata.age !== null) {
        const isRecent = metadata.age < 10;
        expect(isRecent).toBe(true);
      }
    });

    it('should have age as null when timestamp is null', async () => {
      const metadata = await clipboard.getMetadata();
      if (metadata.timestamp === null) {
        expect(metadata.age).toBeNull();
      }
    });
  });

  describe('Content Type Detection', () => {
    it('should detect text content type', async () => {
      await clipboard.writeText('Text content');
      const metadata = await clipboard.getMetadata();

      expect(metadata.contentType).toBe('text');
      expect(metadata.text).toBe('Text content');
      expect(metadata.files).toBeNull();
    });

    it('should detect empty content type', async () => {
      await clipboard.clear();
      const metadata = await clipboard.getMetadata();

      expect(metadata.contentType).toBe('empty');
      expect(metadata.text === null || metadata.text === '').toBe(true);
    });
  });

  describe('Image Detection', () => {
    it('should detect image content type', async () => {
      clipboardState.image = 'data:image/png;base64,test';
      clipboardState.timestamp = Math.floor(Date.now() / 1000);
      const metadata = await clipboard.getMetadata();

      expect(metadata.contentType).toBe('image');
    });

    it('should not detect image in text content', async () => {
      await clipboard.writeText('Plain text');
      const metadata = await clipboard.getMetadata();

      expect(metadata.contentType).toBe('text');
    });
  });

  describe('Files Detection', () => {
    it('should have files field', async () => {
      const metadata = await clipboard.getMetadata();
      expect(metadata.files === null || Array.isArray(metadata.files)).toBe(true);
    });

    it('should have correct file structure when files exist', async () => {
      clipboardState.files = [
        { path: '/tmp/demo.txt', name: 'demo.txt', is_directory: false },
      ];
      clipboardState.timestamp = Math.floor(Date.now() / 1000);
      const metadata = await clipboard.getMetadata();

      if (metadata.files && metadata.files.length > 0) {
        expect(metadata.contentType).toBe('files');
        metadata.files.forEach(file => {
          expect(file).toHaveProperty('path');
          expect(file).toHaveProperty('name');
          expect(file).toHaveProperty('is_directory');
          expect(typeof file.path).toBe('string');
          expect(typeof file.name).toBe('string');
          expect(typeof file.is_directory).toBe('boolean');
        });
      }
    });
  });

  describe('Timestamp Consistency', () => {
    it('should have consistent timestamp across multiple reads', async () => {
      const testText = 'Consistent timestamp test';
      await clipboard.writeText(testText);
      const metadata1 = await clipboard.getMetadata();
      await new Promise(resolve => setTimeout(resolve, 100));
      const metadata2 = await clipboard.getMetadata();

      expect(metadata1.timestamp).toBe(metadata2.timestamp);
      if (metadata1.age !== null && metadata2.age !== null) {
        expect(metadata2.age).toBeGreaterThanOrEqual(metadata1.age);
      }
    });

    it('should update timestamp when content changes', async () => {
      await clipboard.writeText('First content');
      const metadata1 = await clipboard.getMetadata();
      await new Promise(resolve => setTimeout(resolve, 1000));
      await clipboard.writeText('Second content');
      const metadata2 = await clipboard.getMetadata();

      if (metadata1.timestamp && metadata2.timestamp) {
        expect(metadata2.timestamp).toBeGreaterThan(metadata1.timestamp);
      }
      if (metadata2.age !== null) {
        expect(metadata2.age).toBeLessThan(2);
      }
    });
  });

  describe('Complete Metadata Structure', () => {
    it('should have all required fields', async () => {
      await clipboard.writeText('Complete test');
      const metadata = await clipboard.getMetadata();

      expect(metadata).toHaveProperty('text');
      expect(metadata).toHaveProperty('files');
      expect(metadata).toHaveProperty('contentType');
      expect(metadata).toHaveProperty('timestamp');
      expect(metadata).toHaveProperty('age');
    });

    it('should have correct types for all fields', async () => {
      await clipboard.writeText('Type test');
      const metadata = await clipboard.getMetadata();

      expect(metadata.text === null || typeof metadata.text === 'string').toBe(true);
      expect(metadata.files === null || Array.isArray(metadata.files)).toBe(true);
      expect(['text', 'image', 'files', 'empty'].includes(metadata.contentType)).toBe(true);
      expect(metadata.timestamp === null || typeof metadata.timestamp === 'number').toBe(true);
      expect(metadata.age === null || typeof metadata.age === 'number').toBe(true);
    });
  });

  describe('Real-world Usage Scenarios', () => {
    it('should support time-based filtering', async () => {
      await clipboard.writeText('Time-based content');
      const metadata = await clipboard.getMetadata();
      const maxAge = 30;
      const shouldProcess = metadata.age !== null && metadata.age <= maxAge;
      expect(shouldProcess).toBe(true);
    });

    it('should support content type switching', async () => {
      await clipboard.writeText('Switch test');
      const metadata = await clipboard.getMetadata();

      let handled = false;
      switch (metadata.contentType) {
        case 'text':
          expect(metadata.text).toBeTruthy();
          handled = true;
          break;
        case 'image':
          handled = true;
          break;
        case 'files':
          expect(metadata.files).toBeTruthy();
          handled = true;
          break;
        case 'empty':
          handled = true;
          break;
      }

      expect(handled).toBe(true);
    });

    it('should handle null timestamp gracefully', async () => {
      const metadata = await clipboard.getMetadata();
      if (metadata.timestamp === null) {
        expect(metadata.age).toBeNull();
        if (metadata.contentType !== 'empty') {
          expect(true).toBe(true);
        }
      }
    });
  });
});