import { describe, it, expect, vi, beforeEach } from "vitest";

const mockInvoke = vi.fn();

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (...args: unknown[]) => mockInvoke(...args),
}));

describe("detachWindowShortcut store", () => {
  beforeEach(() => {
    mockInvoke.mockReset();
    vi.resetModules();
  });

  it("loads shortcut from backend on first subscribe", async () => {
    mockInvoke.mockResolvedValue("Ctrl+Shift+D");
    const { detachWindowShortcut } = await import("../shortcuts");
    let value = "";
    const unsub = detachWindowShortcut.subscribe((v) => {
      value = v;
    });
    await vi.waitFor(
      () => {
        expect(mockInvoke).toHaveBeenCalledWith("get_detach_window_shortcut");
      },
      { timeout: 5000 },
    );
    expect(value).toBe("Ctrl+Shift+D");
    unsub();
  }, 15000);

  it("setShortcut updates backend and store", async () => {
    mockInvoke.mockResolvedValue(undefined);
    const { detachWindowShortcut } = await import("../shortcuts");
    await detachWindowShortcut.setShortcut("Ctrl+Alt+T");
    expect(mockInvoke).toHaveBeenCalledWith("set_detach_window_shortcut", {
      shortcutStr: "Ctrl+Alt+T",
    });
    let value = "";
    const unsub = detachWindowShortcut.subscribe((v) => {
      value = v;
    });
    expect(value).toBe("Ctrl+Alt+T");
    unsub();
  });

  it("setShortcut throws when backend fails", async () => {
    mockInvoke.mockRejectedValue(new Error("backend error"));
    const { detachWindowShortcut } = await import("../shortcuts");
    const consoleSpy = vi.spyOn(console, "error").mockImplementation(() => {});
    await expect(detachWindowShortcut.setShortcut("Ctrl+X")).rejects.toThrow(
      "backend error",
    );
    consoleSpy.mockRestore();
  });

  it("handles backend error on initial load gracefully", async () => {
    mockInvoke.mockRejectedValue(new Error("not found"));
    const consoleSpy = vi.spyOn(console, "error").mockImplementation(() => {});
    const { detachWindowShortcut } = await import("../shortcuts");
    let value: string | undefined;
    const unsub = detachWindowShortcut.subscribe((v) => {
      value = v;
    });
    await vi.waitFor(
      () => {
        expect(consoleSpy).toHaveBeenCalledWith(
          "Failed to load detach window shortcut:",
          expect.any(Error),
        );
      },
      { timeout: 5000 },
    );
    expect(value).toBe("");
    unsub();
    consoleSpy.mockRestore();
  });
});

describe("toggleWindowShortcut store", () => {
  beforeEach(() => {
    mockInvoke.mockReset();
    vi.resetModules();
  });

  it("keeps alt+Space default until backend resolves, then updates", async () => {
    let resolveLoad: (v: string) => void = () => {};
    mockInvoke.mockReturnValue(
      new Promise<string>((resolve) => {
        resolveLoad = resolve;
      }),
    );
    const { toggleWindowShortcut } = await import("../shortcuts");

    // 后端加载未完成时保持默认值，Alt+Space 拦截器的 gate 才能恒可用
    let value = "";
    const unsub = toggleWindowShortcut.subscribe((v) => {
      value = v;
    });
    expect(value).toBe("alt+Space");

    resolveLoad("Ctrl+Alt+K");
    await vi.waitFor(
      () => {
        expect(value).toBe("Ctrl+Alt+K");
      },
      { timeout: 5000 },
    );
    unsub();
  }, 15000);

  it("keeps alt+Space default when backend returns empty", async () => {
    mockInvoke.mockResolvedValue("");
    const { toggleWindowShortcut } = await import("../shortcuts");

    let value = "";
    const unsub = toggleWindowShortcut.subscribe((v) => {
      value = v;
    });
    await vi.waitFor(
      () => {
        expect(mockInvoke).toHaveBeenCalledWith("get_toggle_shortcut");
      },
      { timeout: 5000 },
    );
    expect(value).toBe("alt+Space");
    unsub();
  }, 15000);

  it("setShortcut invokes backend and updates store", async () => {
    mockInvoke.mockResolvedValue(undefined);
    const { toggleWindowShortcut } = await import("../shortcuts");
    await toggleWindowShortcut.setShortcut("Alt+F1");
    expect(mockInvoke).toHaveBeenCalledWith("set_toggle_shortcut", {
      shortcutStr: "Alt+F1",
    });
    let value = "";
    const unsub = toggleWindowShortcut.subscribe((v) => {
      value = v;
    });
    expect(value).toBe("Alt+F1");
    unsub();
  });
});
