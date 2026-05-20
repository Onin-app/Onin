import { describe, it, expect } from 'vitest';
import { Theme, type ColorConversion, type LaunchableItem } from '../type';

describe('Theme', () => {
  it('has correct enum values', () => {
    expect(Theme.LIGHT).toBe('light');
    expect(Theme.DARK).toBe('dark');
    expect(Theme.SYSTEM).toBe('system');
  });
});

describe('ColorConversion', () => {
  it('has correct shape', () => {
    const color: ColorConversion = {
      hex: '#ff0000',
      rgb: 'rgb(255, 0, 0)',
      hsl: 'hsl(0, 100%, 50%)',
      red: 255,
      green: 0,
      blue: 0,
      alpha: 1,
    };
    expect(color.hex).toBe('#ff0000');
    expect(color.red).toBe(255);
    expect(color.alpha).toBe(1);
  });
});

describe('LaunchableItem', () => {
  it('accepts minimal shape', () => {
    const item: LaunchableItem = {
      name: 'test',
      path: '/path/to/app',
      icon: 'icon.png',
      icon_type: 'Url',
      item_type: 'App',
      source: 'Application',
      keywords: [],
    };
    expect(item.name).toBe('test');
    expect(item.item_type).toBe('App');
  });

  it('accepts full shape with optional fields', () => {
    const item: LaunchableItem = {
      name: 'test',
      path: '/path',
      icon: 'icon',
      icon_type: 'Base64',
      item_type: 'File',
      source: 'FileSearch',
      keywords: [{ name: 'kw', disabled: false, is_default: true }],
      description: 'desc',
      action: 'open',
      origin: 'Hkey',
      source_display: 'Display',
      matches: [],
      modified_time: 123456,
      requires_confirmation: true,
      trigger_mode: 'preview',
    };
    expect(item.keywords[0].name).toBe('kw');
    expect(item.requires_confirmation).toBe(true);
  });
});
