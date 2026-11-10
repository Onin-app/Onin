import { describe, it, expect, vi, beforeEach } from "vitest";
import {
  getShortcuts,
  addShortcut,
  removeShortcut,
  getToggleShortcut,
  setToggleShortcut,
  getDetachWindowShortcut,
  setDetachWindowShortcut,
  setShortcutRecording,
} from "../shortcut";
import type { Shortcut } from "$lib/type";

const mockInvoke = vi.fn();

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (...args: unknown[]) => mockInvoke(...args),
}));

describe("shortcut API", () => {
  beforeEach(() => {
    mockInvoke.mockReset();
  });

  const dummyShortcut: Shortcut = {
    shortcut: "CommandOrControl+Shift+Space",
    command_name: "test_command",
    command_title: "Test",
  };

  it("getShortcuts calls get_shortcuts", async () => {
    mockInvoke.mockResolvedValueOnce([dummyShortcut]);

    const result = await getShortcuts();
    expect(mockInvoke).toHaveBeenCalledWith("get_shortcuts");
    expect(result).toEqual([dummyShortcut]);
  });

  it("addShortcut calls add_shortcut with shortcut and oldShortcutStr", async () => {
    mockInvoke.mockResolvedValueOnce(undefined);

    await addShortcut(dummyShortcut, "Alt+Space");
    expect(mockInvoke).toHaveBeenCalledWith("add_shortcut", {
      shortcut: dummyShortcut,
      oldShortcutStr: "Alt+Space",
    });
  });

  it("removeShortcut calls remove_shortcut with shortcutStr", async () => {
    mockInvoke.mockResolvedValueOnce(undefined);

    await removeShortcut("Alt+Space");
    expect(mockInvoke).toHaveBeenCalledWith("remove_shortcut", {
      shortcutStr: "Alt+Space",
    });
  });

  it("handles toggle and detach window shortcuts", async () => {
    mockInvoke.mockResolvedValueOnce("Alt+Space");
    expect(await getToggleShortcut()).toBe("Alt+Space");
    expect(mockInvoke).toHaveBeenCalledWith("get_toggle_shortcut");

    mockInvoke.mockResolvedValueOnce(undefined);
    await setToggleShortcut("Ctrl+Space");
    expect(mockInvoke).toHaveBeenCalledWith("set_toggle_shortcut", {
      shortcutStr: "Ctrl+Space",
    });

    mockInvoke.mockResolvedValueOnce("Alt+D");
    expect(await getDetachWindowShortcut()).toBe("Alt+D");
    expect(mockInvoke).toHaveBeenCalledWith("get_detach_window_shortcut");

    mockInvoke.mockResolvedValueOnce(undefined);
    await setDetachWindowShortcut("Ctrl+D");
    expect(mockInvoke).toHaveBeenCalledWith("set_detach_window_shortcut", {
      shortcutStr: "Ctrl+D",
    });
  });

  it("setShortcutRecording calls set_shortcut_recording", async () => {
    mockInvoke.mockResolvedValueOnce(undefined);
    await setShortcutRecording(true);
    expect(mockInvoke).toHaveBeenCalledWith("set_shortcut_recording", {
      recording: true,
    });
  });
});
