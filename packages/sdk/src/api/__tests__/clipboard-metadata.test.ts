/**
 * Test for clipboard metadata API v2 (Enhanced)
 */

import { describe, it, expect, beforeEach } from 'vitest';
import { clipboard } from '../clipboard';

describe('Clipboard Metadata API v2', () => {
  beforeEach(async () => {
    // Clear clipboard before each test
    try {
      await clipboard.clear();
    } catch (error) {
      console.warn('Failed to clear clipboard:', error);
    }
  });

  describe('Basic Metadata', () => {
    it('should get metadata with text, timestamp, and age', async () => {
      const testText = 'Test content for metadata';
      
      // Write text to clipboard
      await clipboard.writeText(testText);
      
      // Get metadata
      const metadata = await clipboard.getMetadata();
      
      // Verify text
      expect(metadata.text).toBe(testText);
      expect(metadata.contentType).toBe('text');
      
      // Verify timestamp exists and is recent
      expect(metadata.timestamp).toBeDefined();
      
      // Verify age is calculated
      expect(metadata.age).toBeDefined();
      if (metadata.age !== null) {
        // Content should be less than 5 seconds old
        expect(metadata.age).toBeLessThan(5);
        expect(metadata.age).toBeGreaterThanOrEqual(0);
      }
      
      // Verify age matches timestamp calculation
      if (metadata.timestamp && metadata.age !== null) {
        const now = Math.floor(Date.now() / 1000);
        const calculatedAge = now - metadata.timestamp;
        // Allow 1 second difference due to timing
        expect(Math.abs(metadata.age - calculatedAge)).toBeLessThanOrEqual(1);
      }
    });

    it('should return empty content type when clipboard is empty', async () => {
      await clipboard.clear();
      
      const metadata = await clipboard.getMetadata();
      
      // Should indicate empty clipboard
      expect(metadata.contentType).toBe('empty');
      expect(metadata.text === null || metadata.text === '').toBe(true);
      expect(metadata.files).toBeNull();
    });
  });

  describe('Age Field', () => {
    it('should provide age field for convenience', async () => {
      await clipboard.writeText('Test age field');
      
      // Wait a bit
      await new Promise(resolve => setTimeout(resolve, 1000));
      
      const metadata = await clipboard.getMetadata();
      
      // Age should be at least 1 second
      expect(metadata.age).toBeGreaterThanOrEqual(1);
      
      // Age should match timestamp calculation
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
      
      // Use the convenient age field
      if (metadata.age !== null) {
        // Plugin can decide based on age
        const isRecent = metadata.age < 10; // 10 seconds threshold
        expect(isRecent).toBe(true);
        
        console.log(`Content is ${metadata.age} seconds old, ${isRecent ? 'recent' : 'old'}`);
      }
    });

    it('should have age as null when timestamp is null', async () => {
      const metadata = await clipboard.getMetadata();
      
      // If timestamp is null, age should also be null
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
      const metadata = await clipboard.getMetadata();
      
      // If contentType is 'image', it means there's an image
      if (metadata.contentType === 'image') {
        console.log('Image detected in clipboard');
      }
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
      
      // files should be null or an array
      expect(metadata.files === null || Array.isArray(metadata.files)).toBe(true);
    });

    it('should have correct file structure when files exist', async () => {
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
      
      // Read metadata twice
      const metadata1 = await clipboard.getMetadata();
      await new Promise(resolve => setTimeout(resolve, 100)); // Wait 100ms
      const metadata2 = await clipboard.getMetadata();
      
      // Timestamp should be the same (content hasn't changed)
      expect(metadata1.timestamp).toBe(metadata2.timestamp);
      
      // Age should increase slightly
      if (metadata1.age !== null && metadata2.age !== null) {
        expect(metadata2.age).toBeGreaterThanOrEqual(metadata1.age);
      }
    });

    it('should update timestamp when content changes', async () => {
      // Write first content
      await clipboard.writeText('First content');
      const metadata1 = await clipboard.getMetadata();
      
      // Wait a bit
      await new Promise(resolve => setTimeout(resolve, 1000));
      
      // Write new content
      await clipboard.writeText('Second content');
      const metadata2 = await clipboard.getMetadata();
      
      // Timestamp should be different
      if (metadata1.timestamp && metadata2.timestamp) {
        expect(metadata2.timestamp).toBeGreaterThan(metadata1.timestamp);
      }
      
      // Age of new content should be less than old content
      if (metadata2.age !== null) {
        expect(metadata2.age).toBeLessThan(2);
      }
    });
  });

  describe('Complete Metadata Structure', () => {
    it('should have all required fields', async () => {
      await clipboard.writeText('Complete test');
      const metadata = await clipboard.getMetadata();
      
      // Check all fields exist
      expect(metadata).toHaveProperty('text');
      expect(metadata).toHaveProperty('files');
      expect(metadata).toHaveProperty('contentType');
      expect(metadata).toHaveProperty('timestamp');
      expect(metadata).toHaveProperty('age');
    });

    it('should have correct types for all fields', async () => {
      await clipboard.writeText('Type test');
      const metadata = await clipboard.getMetadata();
      
      // Check types
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
      
      // Simulate plugin logic
      const maxAge = 30; // 30 seconds
      const shouldProcess = metadata.age !== null && metadata.age <= maxAge;
      
      expect(shouldProcess).toBe(true);
    });

    it('should support content type switching', async () => {
      await clipboard.writeText('Switch test');
      const metadata = await clipboard.getMetadata();
      
      // Simulate switch statement
      let handled = false;
      switch (metadata.contentType) {
        case 'text':
          expect(metadata.text).toBeTruthy();
          handled = true;
          break;
        case 'image':
          // Image type detected
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
      
      // Plugin should handle null timestamp
      if (metadata.timestamp === null) {
        expect(metadata.age).toBeNull();
        
        // Can still process based on content type
        if (metadata.contentType !== 'empty') {
          console.log('Content exists but timestamp unavailable');
        }
      }
    });
  });
});
