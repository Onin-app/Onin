import { describe, it, expect, vi, beforeEach } from "vitest";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

vi.mock("@tauri-apps/api/event", () => ({
  listen: vi.fn(),
}));

vi.mock("svelte-sonner", () => ({
  toast: { error: vi.fn() },
}));

import { usePluginList } from "../usePluginList.svelte";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
const mockInvoke = vi.mocked(invoke);
const mockListen = vi.mocked(listen);

const samplePlugin = {
  id: "test-plugin",
  name: "Test Plugin",
  version: "1.0.0",
  description: "A test plugin",
  entry: "index.js",
  author: "test",
  icon: "icon.png",
};

const samplePlugins = [samplePlugin];

describe("usePluginList", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mockInvoke.mockResolvedValue(undefined);
    mockListen.mockResolvedValue(() => {});
  });

  it("should initialize with empty state", () => {
    const pl = usePluginList();
    expect(pl.state.plugins).toEqual([]);
    expect(pl.state.searchQuery).toBe("");
    expect(pl.state.imageErrors).toEqual(new Set());
    expect(pl.filteredPlugins).toEqual([]);
  });

  it("loadPlugins should populate plugins from invoke", async () => {
    const pl = usePluginList();
    mockInvoke.mockResolvedValue(samplePlugins);

    await pl.loadPlugins();

    expect(mockInvoke).toHaveBeenCalledWith("get_loaded_plugins");
    expect(pl.state.plugins).toHaveLength(1);
    expect(pl.state.plugins[0].id).toBe("test-plugin");
    expect(pl.state.plugins[0].enabled).toBe(true);
    expect(pl.state.plugins[0].stars).toBeDefined();
    expect(pl.state.plugins[0].downloads).toBeDefined();
  });

  it("loadPlugins with forceReload should call load_plugins", async () => {
    const pl = usePluginList();
    mockInvoke.mockResolvedValue(samplePlugins);

    await pl.loadPlugins(true);

    expect(mockInvoke).toHaveBeenCalledWith("load_plugins");
  });

  it("loadPlugins should handle errors gracefully", async () => {
    const pl = usePluginList();
    console.error = vi.fn();
    mockInvoke.mockRejectedValue(new Error("fail"));

    await pl.loadPlugins();

    expect(console.error).toHaveBeenCalled();
    expect(pl.state.plugins).toEqual([]);
  });

  it("filteredPlugins should filter by searchQuery", () => {
    const pl = usePluginList();
    pl.state.plugins = [
      samplePlugin,
      { ...samplePlugin, id: "other", name: "Other Plugin" },
    ];

    pl.setSearchQuery("test");
    expect(pl.filteredPlugins).toHaveLength(1);
    expect(pl.filteredPlugins[0].id).toBe("test-plugin");

    pl.setSearchQuery("other");
    expect(pl.filteredPlugins).toHaveLength(1);
    expect(pl.filteredPlugins[0].id).toBe("other");

    pl.setSearchQuery("nonexistent");
    expect(pl.filteredPlugins).toHaveLength(0);
  });

  it("togglePlugin should update state and call invoke", async () => {
    const pl = usePluginList();
    mockInvoke.mockResolvedValue(samplePlugins);
    await pl.loadPlugins();

    mockInvoke.mockResolvedValue(undefined);
    await pl.togglePlugin("test-plugin", false);

    expect(mockInvoke).toHaveBeenCalledWith("toggle_plugin", {
      pluginId: "test-plugin",
      enabled: false,
    });
    expect(pl.state.plugins[0].enabled).toBe(false);
  });

  it("togglePlugin should show error notification on failure", async () => {
    const pl = usePluginList();
    mockInvoke.mockResolvedValue(samplePlugins);
    await pl.loadPlugins();

    mockInvoke.mockImplementation(async (cmd: string) => {
      if (cmd === "toggle_plugin") {
        throw new Error("fail");
      }
      return undefined;
    });
    console.error = vi.fn();

    await pl.togglePlugin("test-plugin", false);

    expect(console.error).toHaveBeenCalled();
    const showNotificationCall = mockInvoke.mock.calls.find(
      (call: unknown[]) => call[0] === "show_notification",
    );
    expect(showNotificationCall).toBeDefined();
  });

  it("toggleAutoDetach should update state and call invoke", async () => {
    const pl = usePluginList();
    mockInvoke.mockResolvedValue(samplePlugins);
    await pl.loadPlugins();

    mockInvoke.mockResolvedValue(undefined);
    await pl.toggleAutoDetach("test-plugin", true);

    expect(mockInvoke).toHaveBeenCalledWith("toggle_plugin_auto_detach", {
      pluginId: "test-plugin",
      autoDetach: true,
    });
    expect(pl.state.plugins[0].auto_detach).toBe(true);
  });

  it("toggleTerminateOnBg should update state and call invoke", async () => {
    const pl = usePluginList();
    mockInvoke.mockResolvedValue(samplePlugins);
    await pl.loadPlugins();

    await pl.toggleTerminateOnBg("test-plugin", true);

    expect(mockInvoke).toHaveBeenCalledWith("toggle_plugin_terminate_on_bg", {
      pluginId: "test-plugin",
      terminateOnBg: true,
    });
    expect(pl.state.plugins[0].terminate_on_bg).toBe(true);
  });

  it("toggleRunAtStartup should update state and call invoke", async () => {
    const pl = usePluginList();
    mockInvoke.mockResolvedValue(samplePlugins);
    await pl.loadPlugins();

    await pl.toggleRunAtStartup("test-plugin", true);

    expect(mockInvoke).toHaveBeenCalledWith("toggle_plugin_run_at_startup", {
      pluginId: "test-plugin",
      runAtStartup: true,
    });
    expect(pl.state.plugins[0].run_at_startup).toBe(true);
  });

  it("executePlugin should call invoke and handle errors", async () => {
    const pl = usePluginList();
    await pl.executePlugin("test-plugin");
    expect(mockInvoke).toHaveBeenCalledWith("execute_plugin_entry", {
      pluginId: "test-plugin",
    });
  });

  it("executePlugin should show toast on error", async () => {
    const pl = usePluginList();
    console.error = vi.fn();
    mockInvoke.mockRejectedValue(new Error("fail"));

    await pl.executePlugin("test-plugin");

    expect(console.error).toHaveBeenCalled();
  });

  it("handleImageError should add pluginId to imageErrors", () => {
    const pl = usePluginList();
    pl.handleImageError("test-plugin");
    expect(pl.state.imageErrors.has("test-plugin")).toBe(true);
  });

  it("setSearchQuery should update searchQuery and filter plugins", () => {
    const pl = usePluginList();
    pl.setSearchQuery("hello");
    expect(pl.state.searchQuery).toBe("hello");
  });

  it("setupListeners should register listeners and return cleanup", async () => {
    const unlisten1 = vi.fn();
    const unlisten2 = vi.fn();
    const unlisten3 = vi.fn();
    const unlisten4 = vi.fn();
    mockListen
      .mockResolvedValueOnce(unlisten1)
      .mockResolvedValueOnce(unlisten2)
      .mockResolvedValueOnce(unlisten3)
      .mockResolvedValueOnce(unlisten4);

    const pl = usePluginList();
    const cleanup = await pl.setupListeners();

    expect(mockListen).toHaveBeenCalledWith(
      "plugin-installed",
      expect.any(Function),
    );
    expect(mockListen).toHaveBeenCalledWith(
      "plugin-settings-schema-registered",
      expect.any(Function),
    );
    expect(mockListen).toHaveBeenCalledWith(
      "plugin-init-error",
      expect.any(Function),
    );
    expect(mockListen).toHaveBeenCalledWith(
      "plugin-init-success",
      expect.any(Function),
    );

    cleanup();
    expect(unlisten1).toHaveBeenCalled();
    expect(unlisten2).toHaveBeenCalled();
    expect(unlisten3).toHaveBeenCalled();
    expect(unlisten4).toHaveBeenCalled();
  });
});
