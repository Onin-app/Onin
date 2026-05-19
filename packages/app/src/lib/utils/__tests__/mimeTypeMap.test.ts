import { describe, it, expect } from 'vitest';
import {
  extensionToMime,
  extensionsToMimes,
  getFileExtension,
  inferMimeType,
} from '../mimeTypeMap';

describe('extensionToMime', () => {
  it('maps known extension to MIME type', () => {
    expect(extensionToMime('.png')).toBe('image/png');
    expect(extensionToMime('.jpg')).toBe('image/jpeg');
    expect(extensionToMime('.pdf')).toBe('application/pdf');
    expect(extensionToMime('.rs')).toBe('text/x-rust');
  });

  it('handles extension without dot prefix', () => {
    expect(extensionToMime('png')).toBe('image/png');
    expect(extensionToMime('mp4')).toBe('video/mp4');
  });

  it('is case insensitive', () => {
    expect(extensionToMime('.PNG')).toBe('image/png');
    expect(extensionToMime('.JPG')).toBe('image/jpeg');
  });

  it('returns null for unknown extension', () => {
    expect(extensionToMime('.xyz')).toBeNull();
    expect(extensionToMime('.foobar')).toBeNull();
  });

  it('returns null for invalid input', () => {
    expect(extensionToMime('')).toBeNull();
    expect(extensionToMime(null as unknown as string)).toBeNull();
    expect(extensionToMime(undefined as unknown as string)).toBeNull();
  });
});

describe('extensionsToMimes', () => {
  it('converts array of extensions to MIME types', () => {
    expect(extensionsToMimes(['.png', '.jpg', '.gif'])).toEqual([
      'image/png',
      'image/jpeg',
      'image/gif',
    ]);
  });

  it('filters out unknown extensions', () => {
    expect(extensionsToMimes(['.png', '.xyz', '.jpg'])).toEqual([
      'image/png',
      'image/jpeg',
    ]);
  });

  it('returns empty array for empty input', () => {
    expect(extensionsToMimes([])).toEqual([]);
  });
});

describe('getFileExtension', () => {
  it('extracts extension from filename', () => {
    expect(getFileExtension('photo.png')).toBe('.png');
    expect(getFileExtension('archive.tar.gz')).toBe('.gz');
    expect(getFileExtension('file.JPEG')).toBe('.jpeg');
  });

  it('returns empty string for files without extension', () => {
    expect(getFileExtension('Makefile')).toBe('');
    expect(getFileExtension('README')).toBe('');
  });

  it('returns empty string for dotfiles', () => {
    expect(getFileExtension('.gitignore')).toBe('');
    expect(getFileExtension('.env')).toBe('');
  });

  it('returns empty string for empty input', () => {
    expect(getFileExtension('')).toBe('');
  });
});

describe('inferMimeType', () => {
  it('infers MIME type from known file extension', () => {
    expect(inferMimeType('image.png')).toBe('image/png');
    expect(inferMimeType('document.pdf')).toBe('application/pdf');
    expect(inferMimeType('script.js')).toBe('text/javascript');
  });

  it('returns application/octet-stream for unknown extension', () => {
    expect(inferMimeType('file.xyz')).toBe('application/octet-stream');
  });

  it('returns application/octet-stream for files without extension', () => {
    expect(inferMimeType('Makefile')).toBe('application/octet-stream');
  });
});
