import { describe, it, expect, vi, beforeEach } from "vitest";
import {
  getCommands,
  updateCommand,
  refreshCommands,
  executeCommand,
} from "../command";
import type { Command } from "$lib/type";

const mockInvoke = vi.fn();

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (...args: unknown[]) => mockInvoke(...args),
}));

describe("command API", () => {
  beforeEach(() => {
    mockInvoke.mockReset();
  });

  const dummyCommand: Command = {
    name: "calculator",
    title: "计算器",
    description: "简易计算器",
    keywords: [{ name: "calc" }],
    icon: "calc",
    source: "Command",
    action: {
      System: "calc",
    },
    type: "Function",
  };

  it("getCommands calls get_commands", async () => {
    mockInvoke.mockResolvedValueOnce([dummyCommand]);

    const result = await getCommands();
    expect(mockInvoke).toHaveBeenCalledWith("get_commands");
    expect(result).toEqual([dummyCommand]);
  });

  it("updateCommand calls update_command with commandToUpdate", async () => {
    mockInvoke.mockResolvedValueOnce(undefined);

    await updateCommand(dummyCommand);
    expect(mockInvoke).toHaveBeenCalledWith("update_command", {
      commandToUpdate: dummyCommand,
    });
  });

  it("refreshCommands calls refresh_commands", async () => {
    mockInvoke.mockResolvedValueOnce(undefined);

    await refreshCommands();
    expect(mockInvoke).toHaveBeenCalledWith("refresh_commands");
  });

  it("executeCommand calls execute_command with name and args", async () => {
    mockInvoke.mockResolvedValueOnce(undefined);

    await executeCommand("calculator", { input: "1+1" });
    expect(mockInvoke).toHaveBeenCalledWith("execute_command", {
      name: "calculator",
      args: { input: "1+1" },
    });
  });
});
