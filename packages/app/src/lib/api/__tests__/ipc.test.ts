import { describe, it, expect, vi, beforeEach } from "vitest";
import { safeInvoke } from "../ipc";

const mockInvoke = vi.fn();

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (...args: unknown[]) => mockInvoke(...args),
}));

describe("IPC layer (safeInvoke)", () => {
  beforeEach(() => {
    mockInvoke.mockReset();
  });

  it("invokes commands without arguments correctly", async () => {
    mockInvoke.mockResolvedValueOnce({ auto_paste_time_limit: 10 });

    const result = await safeInvoke("get_app_config");
    expect(mockInvoke).toHaveBeenCalledTimes(1);
    expect(mockInvoke).toHaveBeenCalledWith("get_app_config");
    expect(result).toEqual({ auto_paste_time_limit: 10 });
  });

  it("invokes commands with arguments correctly", async () => {
    mockInvoke.mockResolvedValueOnce([]);

    await safeInvoke("remove_file_command", { path: "/test/path" });
    expect(mockInvoke).toHaveBeenCalledTimes(1);
    expect(mockInvoke).toHaveBeenCalledWith("remove_file_command", {
      path: "/test/path",
    });
  });

  it("propagates backend rejection errors", async () => {
    mockInvoke.mockRejectedValueOnce(new Error("Command failed in backend"));

    await expect(safeInvoke("get_shortcuts")).rejects.toThrow(
      "Command failed in backend",
    );
  });
});
