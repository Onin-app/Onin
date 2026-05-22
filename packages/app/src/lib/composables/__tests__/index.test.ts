import { describe, it, expect, vi } from "vitest";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

vi.mock("@tauri-apps/api/event", () => ({
  listen: vi.fn(),
}));

import { usePluginManager, useClipboardManager, useAppList } from "../index";
import { usePluginManager as usePluginManagerOrig } from "../usePluginManager.svelte";
import { useClipboardManager as useClipboardManagerOrig } from "../useClipboardManager.svelte";
import { useAppList as useAppListOrig } from "../useAppList.svelte";

describe("composables index", () => {
  it("should export usePluginManager from index", () => {
    expect(usePluginManager).toBe(usePluginManagerOrig);
  });

  it("should export useClipboardManager from index", () => {
    expect(useClipboardManager).toBe(useClipboardManagerOrig);
  });

  it("should export useAppList from index", () => {
    expect(useAppList).toBe(useAppListOrig);
  });
});
