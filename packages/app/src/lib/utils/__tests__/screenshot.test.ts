import { beforeEach, describe, expect, it, vi } from "vitest";

const { mockInvoke, mockShowToastOverlay } = vi.hoisted(() => ({
  mockInvoke: vi.fn(),
  mockShowToastOverlay: vi.fn(),
}));

vi.mock("@tauri-apps/api/core", () => ({ invoke: mockInvoke }));
vi.mock("$lib/utils/toastOverlay", () => ({
  showToastOverlay: mockShowToastOverlay,
}));

import { takeScreenshot } from "../screenshot";

describe("takeScreenshot", () => {
  beforeEach(() => {
    mockInvoke.mockReset();
    mockShowToastOverlay.mockReset();
    mockInvoke.mockResolvedValue(undefined);
    mockShowToastOverlay.mockResolvedValue(undefined);
  });

  it("captures the screen and reports success", async () => {
    await expect(takeScreenshot()).resolves.toBe(true);

    expect(mockInvoke).toHaveBeenCalledWith("take_screenshot");
    expect(mockShowToastOverlay).toHaveBeenCalledWith("截图已复制到剪贴板", {
      kind: "success",
    });
  });

  it("handles capture failures and reports them", async () => {
    mockInvoke.mockRejectedValue(new Error("capture failed"));

    await expect(takeScreenshot()).resolves.toBe(false);

    expect(mockShowToastOverlay).toHaveBeenCalledWith("截图失败，请重试", {
      kind: "error",
    });
  });
});
