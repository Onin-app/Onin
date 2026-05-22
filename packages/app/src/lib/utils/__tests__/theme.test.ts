import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { getTheme } from '../theme';
import { Theme } from '$lib/type';

describe('getTheme', () => {
  const originalMatchMedia = window.matchMedia;

  beforeEach(() => {
    window.matchMedia = vi.fn().mockImplementation((query: string) => ({
      matches: query === '(prefers-color-scheme: dark)',
      media: query,
      onchange: null,
      addListener: vi.fn(),
      removeListener: vi.fn(),
      addEventListener: vi.fn(),
      removeEventListener: vi.fn(),
      dispatchEvent: vi.fn(),
    }));
  });

  afterEach(() => {
    window.matchMedia = originalMatchMedia;
  });

  it('returns DARK for Theme.DARK', () => {
    expect(getTheme(Theme.DARK)).toBe(Theme.DARK);
  });

  it('returns LIGHT for Theme.LIGHT', () => {
    expect(getTheme(Theme.LIGHT)).toBe(Theme.LIGHT);
  });

  it('returns DARK from system when prefers dark', () => {
    expect(getTheme(Theme.SYSTEM)).toBe(Theme.DARK);
  });

  it('returns LIGHT from system when prefers light', () => {
    window.matchMedia = vi.fn().mockReturnValue({ matches: false });
    expect(getTheme(Theme.SYSTEM)).toBe(Theme.LIGHT);
  });
});
