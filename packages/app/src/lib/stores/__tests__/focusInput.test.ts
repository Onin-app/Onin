import { describe, it, expect, beforeEach, vi } from "vitest";
import {
  focusInputTrigger,
  focusExtensionInputTrigger,
  requestInputFocus,
  requestExtensionInputFocus,
  focusInputElement,
} from "../focusInput";

describe("focusInput store", () => {
  beforeEach(() => {
    focusInputTrigger.set(0);
    focusExtensionInputTrigger.set(0);
  });

  it("starts at 0", () => {
    const value = get(focusInputTrigger);
    expect(value).toBe(0);
  });

  it("increments on requestInputFocus", () => {
    requestInputFocus();
    const value = get(focusInputTrigger);
    expect(value).toBe(1);
  });

  it("increments multiple times", () => {
    requestInputFocus();
    requestInputFocus();
    requestInputFocus();
    const value = get(focusInputTrigger);
    expect(value).toBe(3);
  });

  it("increments focusExtensionInputTrigger on requestExtensionInputFocus", () => {
    requestExtensionInputFocus();
    const value = get(focusExtensionInputTrigger);
    expect(value).toBe(1);
  });

  it("focusInputElement calls focus in microtask", async () => {
    const fakeEl = {
      focus: vi.fn(),
    } as unknown as HTMLElement;

    focusInputElement(fakeEl);
    expect(fakeEl.focus).not.toHaveBeenCalled();

    await Promise.resolve();
    expect(fakeEl.focus).toHaveBeenCalledTimes(1);
  });
});

function get<T>(store: {
  subscribe: (run: (value: T) => void) => () => void;
}): T {
  let value: T;
  store.subscribe((v: T) => {
    value = v;
  })();
  return value!;
}
