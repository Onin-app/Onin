import { describe, it, expect, vi, beforeEach } from "vitest";
import {
  getFileCommands,
  addFileCommands,
  removeFileCommand,
} from "../fileCommand";
import type { LaunchableItem } from "$lib/type";

const mockInvoke = vi.fn();

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (...args: unknown[]) => mockInvoke(...args),
}));

describe("fileCommand API", () => {
  beforeEach(() => {
    mockInvoke.mockReset();
  });

  it("getFileCommands calls get_file_commands invoke handler", async () => {
    const mockItems: LaunchableItem[] = [
      {
        name: "Test Doc",
        path: "/path/to/test.doc",
        icon: "",
        icon_type: "Base64",
        item_type: "File",
        source: "FileCommand",
        keywords: [],
        type: "Function",
      },
    ];
    mockInvoke.mockResolvedValue(mockItems);

    const result = await getFileCommands();
    expect(mockInvoke).toHaveBeenCalledTimes(1);
    expect(mockInvoke).toHaveBeenCalledWith("get_file_commands");
    expect(result).toEqual(mockItems);
  });

  it("addFileCommands calls add_file_commands with paths", async () => {
    const paths = ["/path/1", "/path/2"];
    const mockItems: LaunchableItem[] = [
      {
        name: "1",
        path: "/path/1",
        icon: "",
        icon_type: "Base64",
        item_type: "File",
        source: "FileCommand",
        keywords: [],
        type: "Function",
      },
      {
        name: "2",
        path: "/path/2",
        icon: "",
        icon_type: "Base64",
        item_type: "File",
        source: "FileCommand",
        keywords: [],
        type: "Function",
      },
    ];
    mockInvoke.mockResolvedValue(mockItems);

    const result = await addFileCommands(paths);
    expect(mockInvoke).toHaveBeenCalledTimes(1);
    expect(mockInvoke).toHaveBeenCalledWith("add_file_commands", { paths });
    expect(result).toEqual(mockItems);
  });

  it("removeFileCommand calls remove_file_command with path", async () => {
    const targetPath = "/path/1";
    mockInvoke.mockResolvedValue([]);

    const result = await removeFileCommand(targetPath);
    expect(mockInvoke).toHaveBeenCalledTimes(1);
    expect(mockInvoke).toHaveBeenCalledWith("remove_file_command", {
      path: targetPath,
    });
    expect(result).toEqual([]);
  });
});
