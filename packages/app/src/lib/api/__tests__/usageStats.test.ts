import { describe, it, expect, vi, beforeEach } from "vitest";
import {
  getUsageStats,
  recordCommandUsage,
  clearUsageStats,
} from "../usageStats";
import type { CommandUsageStats } from "$lib/type";

const mockInvoke = vi.fn();

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (...args: unknown[]) => mockInvoke(...args),
}));

describe("usageStats API", () => {
  beforeEach(() => {
    mockInvoke.mockReset();
  });

  const dummyStat: CommandUsageStats = {
    command_name: "test_cmd",
    usage_count: 5,
    last_used: 1700000000,
  };

  it("getUsageStats calls get_usage_stats", async () => {
    mockInvoke.mockResolvedValueOnce([dummyStat]);

    const result = await getUsageStats();
    expect(mockInvoke).toHaveBeenCalledWith("get_usage_stats");
    expect(result).toEqual([dummyStat]);
  });

  it("recordCommandUsage calls record_command_usage with commandName", async () => {
    mockInvoke.mockResolvedValueOnce(undefined);

    await recordCommandUsage("test_cmd");
    expect(mockInvoke).toHaveBeenCalledWith("record_command_usage", {
      commandName: "test_cmd",
    });
  });

  it("clearUsageStats calls clear_usage_stats", async () => {
    mockInvoke.mockResolvedValueOnce(undefined);

    await clearUsageStats();
    expect(mockInvoke).toHaveBeenCalledWith("clear_usage_stats");
  });
});
