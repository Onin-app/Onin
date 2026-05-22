import { describe, it, expect } from "vitest";
import { UPDATE_CONFIG } from "../constants";

describe("UPDATE_CONFIG", () => {
  it("is an object with correct structure and types", () => {
    expect(UPDATE_CONFIG).toBeTypeOf("object");
    expect(UPDATE_CONFIG).toHaveProperty("GITHUB_OWNER");
    expect(UPDATE_CONFIG).toHaveProperty("GITHUB_REPO");
    expect(UPDATE_CONFIG).toHaveProperty("LATEST_RELEASE_URL");

    expect(UPDATE_CONFIG.GITHUB_OWNER).toBeTypeOf("string");
    expect(UPDATE_CONFIG.GITHUB_REPO).toBeTypeOf("string");
    expect(UPDATE_CONFIG.LATEST_RELEASE_URL).toBeTypeOf("string");
  });

  it("has correct GitHub owner and repo values", () => {
    expect(UPDATE_CONFIG.GITHUB_OWNER).toBe("b-yp");
    expect(UPDATE_CONFIG.GITHUB_REPO).toBe("baize");
  });

  it("generates correct LATEST_RELEASE_URL dynamically", () => {
    expect(UPDATE_CONFIG.LATEST_RELEASE_URL).toBe(
      "https://api.github.com/repos/b-yp/baize/releases/latest",
    );
  });
});
