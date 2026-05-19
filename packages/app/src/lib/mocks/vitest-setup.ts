import { vi } from 'vitest';

const store: Record<string, string> = {};

Object.defineProperty(globalThis, 'localStorage', {
  value: {
    getItem: vi.fn((key: string) => store[key] ?? null),
    setItem: vi.fn((key: string, value: string) => { store[key] = value; }),
    removeItem: vi.fn((key: string) => { delete store[key]; }),
    clear: vi.fn(() => { Object.keys(store).forEach((k) => delete store[k]); }),
    get length() { return Object.keys(store).length; },
    key: vi.fn(() => null),
  },
  configurable: true,
});

Object.defineProperty(globalThis, 'document', {
  value: {
    documentElement: { classList: { add: vi.fn(), remove: vi.fn() } },
  },
  configurable: true,
});

Object.defineProperty(globalThis, 'window', {
  value: {
    matchMedia: vi.fn((query: string) => ({
      matches: query === '(prefers-color-scheme: dark)',
      media: query,
      onchange: null,
      addListener: vi.fn(),
      removeListener: vi.fn(),
      addEventListener: vi.fn(),
      removeEventListener: vi.fn(),
      dispatchEvent: vi.fn(),
    })),
  },
  configurable: true,
});
