import { describe, it, expect, vi, beforeEach } from "vitest";
import { getAppConfig, updateAppConfig } from "../config";
import type { AppConfig } from "$lib/type";

const mockInvoke = vi.fn();

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (...args: unknown[]) => mockInvoke(...args),
}));

describe("config API", () => {
  beforeEach(() => {
    mockInvoke.mockReset();
  });

  const dummyConfig: AppConfig = {
    auto_paste_time_limit: 5,
    auto_clear_time_limit: 10,
    sort_mode: "frequency",
    enable_usage_tracking: true,
  };

  it("getAppConfig calls get_app_config", async () => {
    mockInvoke.mockResolvedValueOnce(dummyConfig);

    const result = await getAppConfig();
    expect(mockInvoke).toHaveBeenCalledWith("get_app_config");
    expect(result).toEqual(dummyConfig);
  });

  it("updateAppConfig calls update_app_config with config payload", async () => {
    mockInvoke.mockResolvedValueOnce(undefined);

    await updateAppConfig(dummyConfig);
    expect(mockInvoke).toHaveBeenCalledWith("update_app_config", {
      config: dummyConfig,
    });
  });
});
