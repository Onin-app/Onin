import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { sortByUsage } from '../sortByUsage';
import type { LaunchableItem, CommandUsageStats, SortMode } from '$lib/type';

function makeItem(name: string, action?: string): LaunchableItem {
  return {
    name,
    action,
    keywords: [],
    path: '',
    icon: '',
    icon_type: 'Url',
    item_type: 'App',
    source: 'Custom',
  };
}

function makeStat(command_name: string, usage_count: number, last_used?: number): CommandUsageStats {
  return {
    command_name,
    usage_count,
    last_used: last_used ?? Math.floor(Date.now() / 1000),
  };
}

describe('sortByUsage', () => {
  beforeEach(() => {
    vi.useFakeTimers();
    vi.setSystemTime(new Date('2026-01-15T12:00:00Z'));
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  it('returns original items when tracking disabled', () => {
    const items = [makeItem('b'), makeItem('a')];
    const result = sortByUsage(items, [], 'smart', false);
    expect(result).toBe(items);
  });

  it('returns original items in default mode', () => {
    const items = [makeItem('b'), makeItem('a')];
    const result = sortByUsage(items, [], 'default', true);
    expect(result).toBe(items);
  });

  it('sorts by frequency in frequency mode', () => {
    const items = [makeItem('low', 'low'), makeItem('high', 'high'), makeItem('none', 'none')];
    const stats = [
      makeStat('high', 100),
      makeStat('low', 1),
    ];
    const result = sortByUsage(items, stats, 'frequency', true);
    expect(result[0].action).toBe('high');
    expect(result[1].action).toBe('low');
    expect(result[2].action).toBe('none');
  });

  it('sorts by recency in recent mode', () => {
    const now = Math.floor(Date.now() / 1000);
    const items = [makeItem('old', 'old'), makeItem('new', 'new')];
    const stats = [
      makeStat('old', 1, now - 86400 * 30),
      makeStat('new', 1, now - 3600),
    ];
    const result = sortByUsage(items, stats, 'recent', true);
    expect(result[0].action).toBe('new');
    expect(result[1].action).toBe('old');
  });

  it('sorts by combined score in smart mode', () => {
    const now = Math.floor(Date.now() / 1000);
    const items = [
      makeItem('frequent', 'frequent'),
      makeItem('recent', 'recent'),
    ];
    const stats = [
      makeStat('frequent', 50, now - 86400 * 20),
      makeStat('recent', 1, now - 3600),
    ];
    const result = sortByUsage(items, stats, 'smart', true);
    expect(result).toHaveLength(2);
  });

  it('items without stats get 0 score and go to end', () => {
    const items = [makeItem('a', 'a'), makeItem('b', 'b'), makeItem('c', 'c')];
    const stats = [makeStat('a', 10)];
    const result = sortByUsage(items, stats, 'frequency', true);
    expect(result[0].action).toBe('a');
  });

  it('items without action get 0 score', () => {
    const items = [makeItem('no-action'), makeItem('has-action', 'act')];
    const stats = [makeStat('act', 5)];
    const result = sortByUsage(items, stats, 'frequency', true);
    expect(result[0].action).toBe('act');
  });

  it('does not mutate original array', () => {
    const items = [makeItem('b', 'b'), makeItem('a', 'a')];
    const stats = [makeStat('a', 5), makeStat('b', 1)];
    const copy = [...items];
    sortByUsage(items, stats, 'frequency', true);
    expect(items).toEqual(copy);
  });
});
