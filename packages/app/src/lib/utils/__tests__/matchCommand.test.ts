import { describe, it, expect } from 'vitest';
import { checkCommandMatch, getMatchedCommands } from '../matchCommand';
import type { LaunchableItem } from '$lib/type';

function makeFile(name: string, type = ''): File {
  return new File([''], name, { type });
}

function makeItem(name: string, matches: LaunchableItem['matches']): LaunchableItem {
  return {
    name,
    matches,
    path: '',
    icon: '',
    icon_type: 'Url',
    item_type: 'App',
    source: 'Command',
    keywords: [],
  };
}

describe('checkCommandMatch', () => {
  const textItem = makeItem('Text Command', [
    { type: 'text', name: 'Text', description: '', min: 1, max: 100 },
  ]);

  it('matches text with sufficient length', () => {
    expect(checkCommandMatch(textItem, 'hello', [])).toBe(true);
  });

  it('fails text when too short', () => {
    expect(checkCommandMatch(textItem, '', [])).toBe(false);
  });

  it('fails text when too long', () => {
    const shortItem = makeItem('Short', [
      { type: 'text', name: 'Text', description: '', max: 5 },
    ]);
    expect(checkCommandMatch(shortItem, 'hello world', [])).toBe(false);
  });

  it('matches text with regexp', () => {
    const regexItem = makeItem('Email', [
      { type: 'text', name: 'Email', description: '', regexp: '^\\S+@\\S+\\.\\S+$' },
    ]);
    expect(checkCommandMatch(regexItem, 'user@example.com', [])).toBe(true);
    expect(checkCommandMatch(regexItem, 'not-an-email', [])).toBe(false);
  });

  it('handles invalid regexp gracefully', () => {
    const badRegexItem = makeItem('Bad', [
      { type: 'text', name: 'Bad', description: '', regexp: '[invalid' },
    ]);
    expect(checkCommandMatch(badRegexItem, 'test', [])).toBe(false);
  });

  it('combines attachedText and inputText', () => {
    expect(checkCommandMatch(textItem, '', [], 'input')).toBe(true);
  });

  it('prioritizes attachedText over inputText', () => {
    const exactItem = makeItem('Exact', [
      { type: 'text', name: 'Exact', description: '', max: 3 },
    ]);
    expect(checkCommandMatch(exactItem, 'ab', [], 'toolongtext')).toBe(true);
  });
});

describe('checkCommandMatch - file/image types', () => {
  it('matches image files with image type', () => {
    const imageItem = makeItem('Image Opener', [
      { type: 'image', name: 'Image', description: '', extensions: ['.png', '.jpg'] },
    ]);
    expect(checkCommandMatch(imageItem, '', [makeFile('photo.png', 'image/png')])).toBe(true);
    expect(checkCommandMatch(imageItem, '', [makeFile('doc.pdf', 'application/pdf')])).toBe(false);
  });

  it('matches file type with wildcard extension', () => {
    const anyFileItem = makeItem('Any File', [
      { type: 'file', name: 'Any', description: '', extensions: ['*'] },
    ]);
    expect(checkCommandMatch(anyFileItem, '', [makeFile('test.xyz', 'application/octet-stream')])).toBe(true);
  });

  it('matches files without MIME type via extension fallback', () => {
    const extItem = makeItem('JS Opener', [
      { type: 'file', name: 'JS', description: '', extensions: ['.js'] },
    ]);
    expect(checkCommandMatch(extItem, '', [makeFile('script.js', '')])).toBe(true);
    expect(checkCommandMatch(extItem, '', [makeFile('script.ts', '')])).toBe(false);
  });

  it('matches folder type', () => {
    const folderItem = makeItem('Folder Opener', [
      { type: 'folder', name: 'Folder', description: '' },
    ]);
    expect(checkCommandMatch(folderItem, '', [makeFile('mydir', 'application/x-directory')])).toBe(true);
  });

  it('respects min/max file count', () => {
    const multiFileItem = makeItem('Multi Select', [
      { type: 'file', name: 'Files', description: '', min: 2, max: 5, extensions: ['*'] },
    ]);
    expect(checkCommandMatch(multiFileItem, '', [
      makeFile('a.txt'), makeFile('b.txt'),
    ])).toBe(true);
    expect(checkCommandMatch(multiFileItem, '', [makeFile('a.txt')])).toBe(false);
  });

  it('returns false when no matches defined', () => {
    const noMatchItem = makeItem('No Match', []);
    expect(checkCommandMatch(noMatchItem, 'text', [])).toBe(false);
  });

  it('returns false when item has no matches field', () => {
    const item = makeItem('No Field', undefined);
    expect(checkCommandMatch(item, 'text', [])).toBe(false);
  });
});

describe('getMatchedCommands', () => {
  const items = [
    makeItem('Text', [{ type: 'text', name: 'Text', description: '' }]),
    makeItem('Image', [{ type: 'image', name: 'Image', description: '', extensions: ['.png'] }]),
  ];

  it('returns empty when no content', () => {
    expect(getMatchedCommands(items, '', [])).toEqual([]);
  });

  it('filters by text', () => {
    const result = getMatchedCommands(items, 'hello', []);
    expect(result).toHaveLength(1);
    expect(result[0].name).toBe('Text');
  });

  it('filters by files', () => {
    const result = getMatchedCommands(items, '', [makeFile('img.png', 'image/png')]);
    expect(result).toHaveLength(1);
    expect(result[0].name).toBe('Image');
  });
});
