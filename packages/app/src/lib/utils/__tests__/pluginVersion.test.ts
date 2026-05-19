import { describe, it, expect } from 'vitest';
import {
  isValidPluginVersion,
  formatPluginVersion,
  comparePluginVersions,
} from '../pluginVersion';

describe('isValidPluginVersion', () => {
  it('returns true for valid versions', () => {
    expect(isValidPluginVersion('1.0.0')).toBe(true);
    expect(isValidPluginVersion('v2.3.1')).toBe(true);
    expect(isValidPluginVersion('0.0.1-beta')).toBe(true);
  });

  it('returns false for null or undefined', () => {
    expect(isValidPluginVersion(null)).toBe(false);
    expect(isValidPluginVersion(undefined)).toBe(false);
  });

  it('returns false for empty or N/A', () => {
    expect(isValidPluginVersion('')).toBe(false);
    expect(isValidPluginVersion('N/A')).toBe(false);
    expect(isValidPluginVersion('n/a')).toBe(false);
  });

  it('returns false for whitespace-only', () => {
    expect(isValidPluginVersion('   ')).toBe(false);
  });
});

describe('formatPluginVersion', () => {
  it('adds v prefix to numeric versions', () => {
    expect(formatPluginVersion('1.0.0')).toBe('v1.0.0');
    expect(formatPluginVersion('2.3.1')).toBe('v2.3.1');
  });

  it('preserves existing v prefix', () => {
    expect(formatPluginVersion('v1.0.0')).toBe('v1.0.0');
    expect(formatPluginVersion('V2.0.0')).toBe('V2.0.0');
  });

  it('adds v prefix to numeric-starting version with prerelease', () => {
    expect(formatPluginVersion('1.0.0-beta')).toBe('v1.0.0-beta');
  });

  it('returns empty string for invalid versions', () => {
    expect(formatPluginVersion(null)).toBe('');
    expect(formatPluginVersion('')).toBe('');
    expect(formatPluginVersion('N/A')).toBe('');
  });

  it('trims whitespace', () => {
    expect(formatPluginVersion('  1.0.0  ')).toBe('v1.0.0');
  });
});

describe('comparePluginVersions', () => {
  it('returns 0 for equal versions', () => {
    expect(comparePluginVersions('1.0.0', '1.0.0')).toBe(0);
    expect(comparePluginVersions('v1.0.0', '1.0.0')).toBe(0);
  });

  it('returns positive when left is greater (major)', () => {
    expect(comparePluginVersions('2.0.0', '1.0.0')).toBeGreaterThan(0);
  });

  it('returns negative when left is smaller (major)', () => {
    expect(comparePluginVersions('1.0.0', '2.0.0')).toBeLessThan(0);
  });

  it('compares minor version', () => {
    expect(comparePluginVersions('1.2.0', '1.1.0')).toBeGreaterThan(0);
    expect(comparePluginVersions('1.1.0', '1.2.0')).toBeLessThan(0);
  });

  it('compares patch version', () => {
    expect(comparePluginVersions('1.0.3', '1.0.2')).toBeGreaterThan(0);
    expect(comparePluginVersions('1.0.2', '1.0.3')).toBeLessThan(0);
  });

  it('treats missing minor/patch as 0', () => {
    expect(comparePluginVersions('1', '1.0.0')).toBe(0);
    expect(comparePluginVersions('2', '1.9.9')).toBeGreaterThan(0);
  });

  it('handles prerelease: release > prerelease', () => {
    expect(comparePluginVersions('1.0.0', '1.0.0-alpha')).toBeGreaterThan(0);
    expect(comparePluginVersions('1.0.0-alpha', '1.0.0')).toBeLessThan(0);
  });

  it('compares prerelease identifiers', () => {
    expect(comparePluginVersions('1.0.0-alpha', '1.0.0-beta')).toBeLessThan(0);
    expect(comparePluginVersions('1.0.0-beta', '1.0.0-alpha')).toBeGreaterThan(0);
  });

  it('compares numeric prerelease identifiers numerically', () => {
    expect(comparePluginVersions('1.0.0-1', '1.0.0-2')).toBeLessThan(0);
    expect(comparePluginVersions('1.0.0-2', '1.0.0-10')).toBeLessThan(0);
  });

  it('returns 0 when both are invalid', () => {
    expect(comparePluginVersions(null, null)).toBe(0);
    expect(comparePluginVersions('', '')).toBe(0);
  });

  it('invalid version sorts before valid', () => {
    expect(comparePluginVersions(null, '1.0.0')).toBeLessThan(0);
    expect(comparePluginVersions('1.0.0', null)).toBeGreaterThan(0);
  });
});
