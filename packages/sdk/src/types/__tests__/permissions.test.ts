import { describe, it, expect } from 'vitest';
import type {
  HttpPermission,
  StoragePermission,
  NotificationPermission,
  CommandPermission,
  FileSystemPermission,
  DialogPermission,
  ClipboardPermission,
  PluginPermissions,
  PluginCommandKeyword,
  PluginCommandMatch,
  PluginCommand,
  PluginManifest,
} from '../permissions';

describe('HttpPermission', () => {
  it('accepts minimal shape', () => {
    const p: HttpPermission = { enable: true, allowUrls: [] };
    expect(p.enable).toBe(true);
    expect(p.allowUrls).toEqual([]);
  });

  it('accepts full shape with optionals', () => {
    const p: HttpPermission = {
      enable: true,
      allowUrls: ['https://api.example.com/*'],
      timeout: 5000,
      maxRetries: 3,
    };
    expect(p.timeout).toBe(5000);
    expect(p.maxRetries).toBe(3);
  });
});

describe('StoragePermission', () => {
  it('accepts correct shape', () => {
    const p: StoragePermission = { enable: true, local: true, session: false };
    expect(p.local).toBe(true);
    expect(p.session).toBe(false);
  });

  it('accepts optional maxSize', () => {
    const p: StoragePermission = {
      enable: true,
      local: true,
      session: true,
      maxSize: '10MB',
    };
    expect(p.maxSize).toBe('10MB');
  });
});

describe('NotificationPermission', () => {
  it('accepts correct shape', () => {
    const p: NotificationPermission = {
      enable: true,
      sound: true,
      badge: false,
    };
    expect(p.sound).toBe(true);
    expect(p.badge).toBe(false);
  });
});

describe('CommandPermission', () => {
  it('accepts correct shape', () => {
    const p: CommandPermission = {
      enable: true,
      allowCommands: ['cmd_*'],
      maxExecutionTime: 5000,
    };
    expect(p.allowCommands).toContain('cmd_*');
  });
});

describe('PluginPermissions', () => {
  it('accepts partial permissions', () => {
    const p: PluginPermissions = {
      http: { enable: true, allowUrls: [] },
    };
    expect(p.http?.enable).toBe(true);
    expect(p.storage).toBeUndefined();
  });

  it('accepts all permission types', () => {
    const p: PluginPermissions = {
      http: { enable: true, allowUrls: [] },
      storage: { enable: true, local: true, session: true },
      notification: { enable: true, sound: true, badge: true },
      command: { enable: true, allowCommands: ['test'] },
      fs: { enable: true, read: true, write: false, delete: false },
      dialog: {
        enable: true,
        message: true,
        confirm: false,
        fileDialog: false,
      },
      clipboard: {
        enable: true,
        readText: true,
        writeText: true,
        readImage: false,
        writeImage: false,
        clear: false,
      },
    };
    expect(p.clipboard?.readText).toBe(true);
    expect(p.fs?.read).toBe(true);
  });
});

describe('PluginCommandKeyword', () => {
  it('accepts correct shape', () => {
    const kw: PluginCommandKeyword = { name: 'search', type: 'prefix' };
    expect(kw.name).toBe('search');
    expect(kw.type).toBe('prefix');
  });
});

describe('PluginCommandMatch', () => {
  it('accepts text type', () => {
    const m: PluginCommandMatch = {
      type: 'text',
      name: 'text match',
      description: '',
      regexp: '^hello',
      min: 1,
      max: 100,
    };
    expect(m.type).toBe('text');
    expect(m.regexp).toBe('^hello');
  });

  it('accepts file type with extensions', () => {
    const m: PluginCommandMatch = {
      type: 'file',
      name: 'file match',
      description: '',
      extensions: ['.png', '.jpg'],
    };
    expect(m.extensions).toContain('.png');
  });
});

describe('PluginCommand', () => {
  it('accepts minimal shape', () => {
    const cmd: PluginCommand = {
      code: 'test_cmd',
      name: 'Test',
      description: '',
      keywords: [],
    };
    expect(cmd.code).toBe('test_cmd');
  });

  it('accepts with matches', () => {
    const cmd: PluginCommand = {
      code: 'search',
      name: 'Search',
      description: 'Performs search',
      keywords: [{ name: 'find', type: 'prefix' }],
      matches: [{ type: 'text', name: 'query', description: '' }],
    };
    expect(cmd.matches).toHaveLength(1);
  });
});

describe('PluginManifest', () => {
  it('accepts minimal shape', () => {
    const m: PluginManifest = {
      id: 'test-plugin',
      name: 'Test',
      version: '1.0.0',
      description: '',
      entry: 'index.html',
    };
    expect(m.id).toBe('test-plugin');
  });

  it('accepts full shape', () => {
    const m: PluginManifest = {
      id: 'full-plugin',
      name: 'Full',
      version: '2.0.0',
      description: 'A full plugin',
      entry: 'app.html',
      type: 'app',
      displayMode: 'inline',
      autoDetach: true,
      permissions: {
        http: { enable: true, allowUrls: [] },
      },
      commands: [
        {
          code: 'cmd1',
          name: 'Command 1',
          description: '',
          keywords: [],
        },
      ],
    };
    expect(m.type).toBe('app');
    expect(m.commands).toHaveLength(1);
    expect(m.autoDetach).toBe(true);
  });
});
