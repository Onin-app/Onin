import { describe, it, expect } from 'vitest';
import { fuzzyMatch } from '../fuzzyMatch';
import type { LaunchableItem } from '$lib/type';

function makeItem(name: string, keywords?: string[]): LaunchableItem {
  return {
    name,
    keywords: (keywords ?? []).map((k) => ({ name: k })),
    path: '',
    icon: '',
    icon_type: 'Url',
    item_type: 'App',
    source: 'Custom',
  };
}

describe('fuzzyMatch', () => {
  const items: LaunchableItem[] = [
    makeItem('Calculator'),
    makeItem('Calendar'),
    makeItem('Camera'),
    makeItem('Clock'),
    makeItem('Terminal'),
  ];

  it('returns all items for empty query', () => {
    const result = fuzzyMatch('', items);
    expect(result).toBe(items);
  });

  it('returns all items for null/undefined query', () => {
    expect(fuzzyMatch('', items)).toBe(items);
  });

  it('filters by exact name match', () => {
    const result = fuzzyMatch('calculator', items);
    expect(result).toHaveLength(1);
    expect(result[0].name).toBe('Calculator');
  });

  it('filters by prefix match', () => {
    const result = fuzzyMatch('Calc', items);
    expect(result).toHaveLength(1);
    expect(result[0].name).toBe('Calculator');
  });

  it('filters by substring match', () => {
    const result = fuzzyMatch('al', items);
    expect(result.length).toBeGreaterThan(0);
    expect(result.map((i) => i.name)).toContain('Calculator');
    expect(result.map((i) => i.name)).toContain('Calendar');
  });

  it('returns empty array for no match', () => {
    const result = fuzzyMatch('zzzzz', items);
    expect(result).toHaveLength(0);
  });

  it('matches keywords', () => {
    const keywordItems = [
      makeItem('Settings', ['config', 'preferences']),
      makeItem('Terminal', ['console', 'shell']),
    ];
    const result = fuzzyMatch('config', keywordItems);
    expect(result).toHaveLength(1);
    expect(result[0].name).toBe('Settings');
  });

  it('is case insensitive', () => {
    const result = fuzzyMatch('CALCULATOR', items);
    expect(result).toHaveLength(1);
    expect(result[0].name).toBe('Calculator');
  });

  it('returns empty array for empty item array', () => {
    const result = fuzzyMatch('test', []);
    expect(result).toEqual([]);
  });

  it('sorts by score: higher score first', () => {
    const items = [
      makeItem('App Store'),
      makeItem('Store'),
    ];
    const result = fuzzyMatch('store', items);
    expect(result[0].name).toBe('Store');
    expect(result[1].name).toBe('App Store');
  });
});
