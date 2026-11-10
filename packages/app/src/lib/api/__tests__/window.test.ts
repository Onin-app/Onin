import { describe, it, expect, vi, beforeEach } from "vitest";
import {
  acquireWindowCloseLock,
  releaseWindowCloseLock,
  closeMainWindow,
} from "../window";

const mockInvoke = vi.fn();

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (...args: unknown[]) => mockInvoke(...args),
}));

describe("window API", () => {
  beforeEach(() => {
    mockInvoke.mockReset();
  });

  it("acquireWindowCloseLock calls acquire_window_close_lock", async () => {
    mockInvoke.mockResolvedValueOnce(undefined);

    await acquireWindowCloseLock();
    expect(mockInvoke).toHaveBeenCalledWith("acquire_window_close_lock");
  });

  it("releaseWindowCloseLock calls release_window_close_lock", async () => {
    mockInvoke.mockResolvedValueOnce(undefined);

    await releaseWindowCloseLock();
    expect(mockInvoke).toHaveBeenCalledWith("release_window_close_lock");
  });

  it("closeMainWindow calls close_main_window", async () => {
    mockInvoke.mockResolvedValueOnce(undefined);

    await closeMainWindow();
    expect(mockInvoke).toHaveBeenCalledWith("close_main_window");
  });
});
