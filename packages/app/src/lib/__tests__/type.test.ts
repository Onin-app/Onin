import { describe, it, expect } from "vitest";
import {
  Theme,
  type ColorConversion,
  type LaunchableItem,
  type CommandKeyword,
  type CommandMatch,
  type CommandAction,
  type Command,
  type Shortcut,
  type SortMode,
  type CommandUsageStats,
  type AppConfig,
  type ItemType,
  type Source,
  type IconType,
  type AppOrigin,
} from "../type";

describe("Theme", () => {
  it("has correct enum values", () => {
    expect(Theme.LIGHT).toBe("light");
    expect(Theme.DARK).toBe("dark");
    expect(Theme.SYSTEM).toBe("system");
  });
});

describe("ColorConversion", () => {
  it("has correct shape", () => {
    const color: ColorConversion = {
      hex: "#ff0000",
      rgb: "rgb(255, 0, 0)",
      hsl: "hsl(0, 100%, 50%)",
      red: 255,
      green: 0,
      blue: 0,
      alpha: 1,
    };
    expect(color.hex).toBe("#ff0000");
    expect(color.red).toBe(255);
    expect(color.alpha).toBe(1);
  });
});

describe("LaunchableItem", () => {
  it("accepts minimal shape", () => {
    const item: LaunchableItem = {
      name: "test",
      path: "/path/to/app",
      icon: "icon.png",
      icon_type: "Url",
      item_type: "App",
      source: "Application",
      keywords: [],
    };
    expect(item.name).toBe("test");
    expect(item.item_type).toBe("App");
  });

  it("accepts full shape with optional fields", () => {
    const item: LaunchableItem = {
      name: "test",
      path: "/path",
      icon: "icon",
      icon_type: "Base64",
      item_type: "File",
      source: "FileSearch",
      keywords: [{ name: "kw", disabled: false, is_default: true }],
      description: "desc",
      action: "open",
      origin: "Hkey",
      source_display: "Display",
      matches: [],
      modified_time: 123456,
      requires_confirmation: true,
      trigger_mode: "preview",
    };
    expect(item.keywords[0].name).toBe("kw");
    expect(item.requires_confirmation).toBe(true);
  });
});

describe("CommandKeyword", () => {
  it("accepts minimal and full shape", () => {
    const kw1: CommandKeyword = { name: "test" };
    const kw2: CommandKeyword = {
      name: "test",
      disabled: true,
      is_default: false,
    };
    expect(kw1.name).toBe("test");
    expect(kw2.disabled).toBe(true);
    expect(kw2.is_default).toBe(false);
  });
});

describe("CommandMatch", () => {
  it("accepts text match shape", () => {
    const match: CommandMatch = {
      type: "text",
      name: "search text",
      description: "a test search",
      regexp: "^foo$",
      min: 1,
      max: 100,
    };
    expect(match.type).toBe("text");
    expect(match.regexp).toBe("^foo$");
  });

  it("accepts file match shape", () => {
    const match: CommandMatch = {
      type: "file",
      name: "images",
      description: "image files",
      extensions: [".png", ".jpg"],
    };
    expect(match.type).toBe("file");
    expect(match.extensions).toContain(".png");
  });
});

describe("CommandAction", () => {
  it("accepts various action variants", () => {
    const actSystem: CommandAction = { System: "lock" };
    const actApp: CommandAction = { App: "/path/to/app" };
    const actFile: CommandAction = { File: "/path/to/file" };
    const actPluginEntry: CommandAction = {
      PluginEntry: { plugin_id: "plugin1" },
    };
    const actPluginCommand: CommandAction = {
      PluginCommand: { plugin_id: "plugin1", command_code: "cmd" },
    };
    const actExtension: CommandAction = {
      Extension: { extension_id: "ext1", command_code: "cmd" },
    };

    expect(actSystem).toHaveProperty("System");
    expect(actApp).toHaveProperty("App");
    expect(actFile).toHaveProperty("File");
    expect(actPluginEntry).toHaveProperty("PluginEntry");
    expect(actPluginCommand).toHaveProperty("PluginCommand");
    expect(actExtension).toHaveProperty("Extension");
  });
});

describe("Command", () => {
  it("accepts full command shape", () => {
    const cmd: Command = {
      name: "lock-screen",
      title: "Lock Screen",
      english_name: "lock",
      keywords: [{ name: "lock" }],
      icon: "lock.png",
      source: "Application",
      action: { System: "lock" },
      description: "Locks the desktop",
      path: "/usr/bin/lock",
      origin: "Shortcut",
      matches: [{ type: "text", name: "text", description: "text" }],
    };
    expect(cmd.name).toBe("lock-screen");
    expect(cmd.action).toHaveProperty("System");
  });
});

describe("Shortcut", () => {
  it("accepts shortcut shape", () => {
    const shortcut: Shortcut = {
      shortcut: "Ctrl+L",
      command_name: "lock-screen",
      command_title: "Lock Screen",
      readonly: true,
    };
    expect(shortcut.shortcut).toBe("Ctrl+L");
    expect(shortcut.readonly).toBe(true);
  });
});

describe("CommandUsageStats", () => {
  it("accepts stats shape", () => {
    const stats: CommandUsageStats = {
      command_name: "lock-screen",
      usage_count: 5,
      last_used: 1716123456,
    };
    expect(stats.command_name).toBe("lock-screen");
    expect(stats.usage_count).toBe(5);
  });
});

describe("AppConfig", () => {
  it("accepts config shape", () => {
    const config: AppConfig = {
      auto_paste_time_limit: 10,
      auto_clear_time_limit: 60,
      sort_mode: "smart",
      enable_usage_tracking: true,
      marketplace_api_url: "https://api.onin.dev",
      disabled_extension_ids: ["ext1"],
      file_search_excluded_paths: ["/tmp"],
      file_search_include_hidden: false,
    };
    expect(config.sort_mode).toBe("smart");
    expect(config.enable_usage_tracking).toBe(true);
  });
});

describe("Union Types", () => {
  it("validates ItemType", () => {
    const app: ItemType = "App";
    const folder: ItemType = "Folder";
    const file: ItemType = "File";
    expect(app).toBe("App");
    expect(folder).toBe("Folder");
    expect(file).toBe("File");
  });

  it("validates Source", () => {
    const sources: Source[] = [
      "Application",
      "Custom",
      "Command",
      "FileCommand",
      "FileSearch",
      "Plugin",
      "Extension",
    ];
    expect(sources).toContain("Application");
    expect(sources).toContain("Plugin");
  });

  it("validates IconType", () => {
    const types: IconType[] = ["Base64", "Iconfont", "Url"];
    expect(types).toContain("Base64");
  });

  it("validates AppOrigin", () => {
    const origins: AppOrigin[] = ["Hkey", "Shortcut", "Uwp"];
    expect(origins).toContain("Shortcut");
  });

  it("validates SortMode", () => {
    const modes: SortMode[] = ["smart", "frequency", "recent", "default"];
    expect(modes).toContain("smart");
  });
});
