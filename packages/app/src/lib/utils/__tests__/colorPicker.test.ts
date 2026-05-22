import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

vi.mock("@tauri-apps/api/event", () => ({
  listen: vi.fn(),
}));

vi.mock("svelte-sonner", () => ({
  toast: {
    success: vi.fn(),
    error: vi.fn(),
    info: vi.fn(),
  },
}));

vi.mock("$lib/utils/toastOverlay", () => ({
  showToastOverlay: vi.fn().mockReturnValue(Promise.resolve()),
}));

import { startColorPickerFlow, resetColorPickerState } from "../colorPicker";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { toast } from "svelte-sonner";
import { showToastOverlay } from "$lib/utils/toastOverlay";

const mockInvoke = vi.mocked(invoke);
const mockListen = vi.mocked(listen);
const mockToast = vi.mocked(toast);
const mockShowToastOverlay = vi.mocked(showToastOverlay);

const fakeUnlisten = vi.fn();

function fireColorPickerResult(payload: unknown) {
  const cb = mockListen.mock.calls[0][1] as (event: {
    payload: unknown;
  }) => void | Promise<void>;
  return cb({ payload });
}

function defaultMockInvoke() {
  mockInvoke.mockImplementation(async (cmd: string) => {
    if (cmd === "start_color_picker") return;
    if (cmd === "plugin_clipboard_write_text") return;
    if (cmd === "close_main_window") return;
  });
}

describe("startColorPickerFlow", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mockListen.mockResolvedValue(fakeUnlisten);
    defaultMockInvoke();
  });

  afterEach(async () => {
    resetColorPickerState();
  });

  it("calls beforeStart hook", async () => {
    const beforeStart = vi.fn();
    await startColorPickerFlow({ beforeStart });
    expect(beforeStart).toHaveBeenCalled();
  });

  it("registers listener for color_picker_result and starts picker", async () => {
    await startColorPickerFlow();
    expect(mockListen).toHaveBeenCalledWith(
      "color_picker_result",
      expect.any(Function),
    );
    expect(mockInvoke).toHaveBeenCalledWith("start_color_picker", {
      restoreMainWindow: true,
    });
  });

  it("ignores subsequent calls while already picking", async () => {
    await startColorPickerFlow();
    await startColorPickerFlow();
    expect(mockListen).toHaveBeenCalledTimes(1);
  });

  it("handles cancel (null payload) showing info toast", async () => {
    await startColorPickerFlow();
    await fireColorPickerResult(null);
    expect(mockToast.info).toHaveBeenCalledWith("已取消取色");
  });

  it("calls onCancel when payload is null", async () => {
    const onCancel = vi.fn();
    await startColorPickerFlow({ onCancel });
    await fireColorPickerResult(null);
    expect(onCancel).toHaveBeenCalled();
  });

  it("copies hex to clipboard and shows success toast", async () => {
    await startColorPickerFlow();
    await fireColorPickerResult({
      hex: "#ff0000",
      rgb: "rgb(255,0,0)",
      hsl: "hsl(0,100%,50%)",
      red: 255,
      green: 0,
      blue: 0,
      alpha: 1,
    });
    expect(mockInvoke).toHaveBeenCalledWith("plugin_clipboard_write_text", {
      options: { text: "#ff0000" },
    });
    expect(mockToast.success).toHaveBeenCalledWith("#ff0000 已复制");
    expect(mockInvoke).toHaveBeenCalledWith("close_main_window");
  });

  it("calls onSuccess with the color payload", async () => {
    const onSuccess = vi.fn();
    const payload = {
      hex: "#00ff00",
      rgb: "rgb(0,255,0)",
      hsl: "hsl(120,100%,50%)",
      red: 0,
      green: 255,
      blue: 0,
      alpha: 1,
    };
    await startColorPickerFlow({ onSuccess });
    await fireColorPickerResult(payload);
    expect(onSuccess).toHaveBeenCalledWith(payload);
  });

  it("does not close main window when closeOnSuccess is false", async () => {
    await startColorPickerFlow({ closeOnSuccess: false });
    await fireColorPickerResult({
      hex: "#000",
      rgb: "rgb(0,0,0)",
      hsl: "hsl(0,0%,0%)",
      red: 0,
      green: 0,
      blue: 0,
      alpha: 1,
    });
    expect(mockInvoke).not.toHaveBeenCalledWith("close_main_window");
  });

  it("shows error toast when clipboard write fails", async () => {
    mockInvoke.mockImplementation(async (cmd: string) => {
      if (cmd === "start_color_picker") return;
      if (cmd === "plugin_clipboard_write_text")
        throw new Error("clipboard error");
      if (cmd === "close_main_window") return;
    });
    await startColorPickerFlow();
    await fireColorPickerResult({
      hex: "#fff",
      rgb: "rgb(255,255,255)",
      hsl: "hsl(0,0%,100%)",
      red: 255,
      green: 255,
      blue: 255,
      alpha: 1,
    });
    expect(mockToast.error).toHaveBeenCalledWith("#fff 已取色，复制失败");
  });

  it("shows toast overlay when useToastOverlay is true", async () => {
    const orig = window.setTimeout;
    try {
      window.setTimeout = ((fn: () => void) => {
        fn();
      }) as never;
      await startColorPickerFlow({ useToastOverlay: true });
      await fireColorPickerResult({
        hex: "#333",
        rgb: "rgb(51,51,51)",
        hsl: "hsl(0,0%,20%)",
        red: 51,
        green: 51,
        blue: 51,
        alpha: 1,
      });
      expect(mockShowToastOverlay).toHaveBeenCalledWith("#333 已复制", {
        kind: "success",
        duration: 1400,
      });
    } finally {
      window.setTimeout = orig;
    }
  });

  it("shows error toast overlay when clipboard fails and useToastOverlay is true", async () => {
    const orig = window.setTimeout;
    try {
      window.setTimeout = ((fn: () => void) => {
        fn();
      }) as never;
      mockInvoke.mockImplementation(async (cmd: string) => {
        if (cmd === "start_color_picker") return;
        if (cmd === "plugin_clipboard_write_text") throw new Error("fail");
        if (cmd === "close_main_window") return;
      });
      await startColorPickerFlow({ useToastOverlay: true });
      await fireColorPickerResult({
        hex: "#333",
        rgb: "rgb(51,51,51)",
        hsl: "hsl(0,0%,20%)",
        red: 51,
        green: 51,
        blue: 51,
        alpha: 1,
      });
      expect(mockShowToastOverlay).toHaveBeenCalledWith(
        "#333 已取色，复制失败",
        {
          kind: "error",
          duration: 1800,
        },
      );
    } finally {
      window.setTimeout = orig;
    }
  });

  it("handles start_color_picker invoke error", async () => {
    const consoleSpy = vi.spyOn(console, "error").mockImplementation(() => {});
    try {
      mockInvoke.mockRejectedValue(new Error("picker error"));
      await startColorPickerFlow();
      expect(mockToast.error).toHaveBeenCalledWith("picker error");
    } finally {
      consoleSpy.mockRestore();
    }
  });

  it("calls cleanupListener on start_color_picker error", async () => {
    const consoleSpy = vi.spyOn(console, "error").mockImplementation(() => {});
    try {
      mockInvoke.mockRejectedValue(new Error("fail"));
      await startColorPickerFlow();
      expect(fakeUnlisten).toHaveBeenCalledTimes(1);
    } finally {
      consoleSpy.mockRestore();
    }
  });

  it("calls restoreMainWindow with custom value", async () => {
    await startColorPickerFlow({ restoreMainWindow: false });
    expect(mockInvoke).toHaveBeenCalledWith("start_color_picker", {
      restoreMainWindow: false,
    });
  });
});
