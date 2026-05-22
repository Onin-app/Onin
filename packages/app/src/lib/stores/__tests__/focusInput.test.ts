import { describe, it, expect, beforeEach } from 'vitest';
import { focusInputTrigger, requestInputFocus } from '../focusInput';

describe('focusInput store', () => {
  beforeEach(() => {
    focusInputTrigger.set(0);
  });

  it('starts at 0', () => {
    const value = get(focusInputTrigger);
    expect(value).toBe(0);
  });

  it('increments on requestInputFocus', () => {
    requestInputFocus();
    const value = get(focusInputTrigger);
    expect(value).toBe(1);
  });

  it('increments multiple times', () => {
    requestInputFocus();
    requestInputFocus();
    requestInputFocus();
    const value = get(focusInputTrigger);
    expect(value).toBe(3);
  });
});

function get<T>(store: { subscribe: (run: (value: T) => void) => () => void }): T {
  let value: T;
  store.subscribe((v: T) => { value = v; })();
  return value!;
}
