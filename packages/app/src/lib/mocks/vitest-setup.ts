import { vi } from 'vitest';

const store: Record<string, string> = {};

if (!globalThis.localStorage) {
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
}

if (!globalThis.document) {
  Object.defineProperty(globalThis, 'document', {
    value: {
      documentElement: { classList: { add: vi.fn(), remove: vi.fn() } },
    },
    configurable: true,
  });
} else {
  // If document already exists (e.g. under jsdom), just ensure documentElement is augmented if needed
  if (!globalThis.document.documentElement) {
    Object.defineProperty(globalThis.document, 'documentElement', {
      value: { classList: { add: vi.fn(), remove: vi.fn() } },
      configurable: true,
      writable: true,
    });
  } else {
    if (!globalThis.document.documentElement.classList) {
      (globalThis.document.documentElement as any).classList = { add: vi.fn(), remove: vi.fn() };
    }
  }
}

if (!globalThis.window) {
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
} else {
  if (!globalThis.window.matchMedia) {
    Object.defineProperty(globalThis.window, 'matchMedia', {
      value: vi.fn((query: string) => ({
        matches: query === '(prefers-color-scheme: dark)',
        media: query,
        onchange: null,
        addListener: vi.fn(),
        removeListener: vi.fn(),
        addEventListener: vi.fn(),
        removeEventListener: vi.fn(),
        dispatchEvent: vi.fn(),
      })),
      configurable: true,
      writable: true,
    });
  }
}

// Mock scrollIntoView for JSDOM
if (typeof globalThis.window !== 'undefined' && globalThis.window.HTMLElement) {
  globalThis.window.HTMLElement.prototype.scrollIntoView = vi.fn();
}
