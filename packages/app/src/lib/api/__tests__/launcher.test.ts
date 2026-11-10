import { describe, it, expect, vi, beforeEach } from "vitest";
import { getAllLaunchableItems, openApp } from "../launcher";
import type { LaunchableItem } from "$lib/type";

const mockInvoke = vi.fn();

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (...args: unknown[]) => mockInvoke(...args),
}));

describe("launcher API", () => {
  beforeEach(() => {
    mockInvoke.mockReset();
  });

  const dummyItem: LaunchableItem = {
    name: "Test App",
    path: "/Applications/Test.app",
    icon: "icon",
    icon_type: "Iconfont",
    item_type: "App",
    source: "Application",
    keywords: [],
    type: "Function",
  };

  it("getAllLaunchableItems calls get_all_launchable_items", async () => {
    mockInvoke.mockResolvedValueOnce([dummyItem]);

    const result = await getAllLaunchableItems();
    expect(mockInvoke).toHaveBeenCalledWith("get_all_launchable_items");
    expect(result).toEqual([dummyItem]);
  });

  it("openApp calls open_app with path", async () => {
    mockInvoke.mockResolvedValueOnce(undefined);

    await openApp("/test/path");
    expect(mockInvoke).toHaveBeenCalledWith("open_app", {
      path: "/test/path",
    });
  });
});
