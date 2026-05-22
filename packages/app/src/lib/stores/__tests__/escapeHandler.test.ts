import { describe, it, expect } from "vitest";
import { escapeHandler } from "../escapeHandler";

describe("escapeHandler store", () => {
  it("starts as null", () => {
    let value: (() => void) | null | string = "initial";
    escapeHandler.subscribe((v) => {
      value = v;
    })();
    expect(value).toBeNull();
  });

  it("can set a handler", () => {
    const handler = () => {};
    escapeHandler.set(handler);
    let value: (() => void) | null = null;
    escapeHandler.subscribe((v) => {
      value = v;
    })();
    expect(value).toBe(handler);
  });

  it("can clear the handler", () => {
    escapeHandler.set(() => {});
    escapeHandler.set(null);
    let value: (() => void) | null | string = "not-cleared";
    escapeHandler.subscribe((v) => {
      value = v;
    })();
    expect(value).toBeNull();
  });
});
