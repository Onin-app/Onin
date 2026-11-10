import { describe, it, expect, vi, beforeEach } from "vitest";
import {
  openAppDataDir,
  setTrayVisibility,
  isTrayVisible,
  showNotification,
} from "../system";

const mockInvoke = vi.fn();

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (...args: unknown[]) => mockInvoke(...args),
}));

describe("system API", () => {
  beforeEach(() => {
    mockInvoke.mockReset();
  });

  it("openAppDataDir calls open_app_data_dir", async () => {
    mockInvoke.mockResolvedValueOnce(undefined);

    await openAppDataDir();
    expect(mockInvoke).toHaveBeenCalledWith("open_app_data_dir");
  });

  it("setTrayVisibility calls set_tray_visibility with visible", async () => {
    mockInvoke.mockResolvedValueOnce(undefined);

    await setTrayVisibility(true);
    expect(mockInvoke).toHaveBeenCalledWith("set_tray_visibility", {
      visible: true,
    });
  });

  it("isTrayVisible calls is_tray_visible", async () => {
    mockInvoke.mockResolvedValueOnce(true);

    const result = await isTrayVisible();
    expect(mockInvoke).toHaveBeenCalledWith("is_tray_visible");
    expect(result).toBe(true);
  });

  it("showNotification calls show_notification with options", async () => {
    mockInvoke.mockResolvedValueOnce(undefined);

    await showNotification({ title: "Test Title", body: "Test Body" });
    expect(mockInvoke).toHaveBeenCalledWith("show_notification", {
      options: { title: "Test Title", body: "Test Body" },
    });
  });
});
