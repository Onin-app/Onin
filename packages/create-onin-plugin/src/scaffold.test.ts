import { strict as assert } from "node:assert";
import { mkdtemp, readFile, rm } from "node:fs/promises";
import { tmpdir } from "node:os";
import { dirname, join, resolve } from "node:path";
import test from "node:test";
import { fileURLToPath } from "node:url";

import { scaffoldPlugin } from "./scaffold.js";
import type { CliOptions, Framework, Language } from "./types.js";

const TEST_DIR = dirname(fileURLToPath(import.meta.url));

const baseTemplateDirs: Record<Language, string> = {
  ts: resolve(TEST_DIR, "../templates/base/ts"),
  js: resolve(TEST_DIR, "../templates/base/js"),
};

const adapterTemplateDirs: Record<Framework, Partial<Record<Language, string>>> = {
  svelte: {
    ts: resolve(TEST_DIR, "../templates/adapters/svelte/ts"),
  },
  react: {
    ts: resolve(TEST_DIR, "../templates/adapters/react/ts"),
    js: resolve(TEST_DIR, "../templates/adapters/react/js"),
  },
  vue: {
    ts: resolve(TEST_DIR, "../templates/adapters/vue/ts"),
  },
  vanilla: {
    ts: resolve(TEST_DIR, "../templates/adapters/vanilla/ts"),
    js: resolve(TEST_DIR, "../templates/adapters/vanilla/js"),
  },
  solid: {
    ts: resolve(TEST_DIR, "../templates/adapters/solid/ts"),
  },
};

async function createTempDir(prefix: string): Promise<string> {
  return mkdtemp(join(tmpdir(), prefix));
}

function createCliOptions(targetDir: string, framework: Framework, language: Language): CliOptions {
  return {
    targetDir,
    pluginName: "Smoke Plugin",
    pluginId: "com.example.smoke-plugin",
    withSettings: true,
    yes: true,
    framework,
    language,
  };
}

test("scaffoldPlugin creates a vanilla TypeScript project", async () => {
  const tempDir = await createTempDir("create-onin-plugin-scaffold-ts-");
  const targetDir = join(tempDir, "vanilla-ts-plugin");

  try {
    const result = await scaffoldPlugin(
      createCliOptions(targetDir, "vanilla", "ts"),
      baseTemplateDirs,
      adapterTemplateDirs,
    );

    assert.equal(result.targetDir, targetDir);
    assert.match(await readFile(join(targetDir, "package.json"), "utf8"), /"typescript": "\^5\.5\.0"/);
    assert.match(await readFile(join(targetDir, "src", "main.ts"), "utf8"), /Smoke Plugin/);
    assert.match(
      await readFile(join(targetDir, "vite.lifecycle.config.ts"), "utf8"),
      /src\/lifecycle\.ts/,
    );
  } finally {
    await rm(tempDir, { recursive: true, force: true });
  }
});

test("scaffoldPlugin creates a vanilla JavaScript project", async () => {
  const tempDir = await createTempDir("create-onin-plugin-scaffold-js-");
  const targetDir = join(tempDir, "vanilla-js-plugin");

  try {
    const result = await scaffoldPlugin(
      createCliOptions(targetDir, "vanilla", "js"),
      baseTemplateDirs,
      adapterTemplateDirs,
    );

    assert.equal(result.targetDir, targetDir);
    assert.match(await readFile(join(targetDir, "package.json"), "utf8"), /"vite": "\^7\.3\.1"/);
    assert.doesNotMatch(
      await readFile(join(targetDir, "package.json"), "utf8"),
      /"typescript": "\^5\.5\.0"/,
    );
    assert.match(await readFile(join(targetDir, "src", "main.js"), "utf8"), /Smoke Plugin/);
    assert.match(
      await readFile(join(targetDir, "vite.lifecycle.config.js"), "utf8"),
      /src\/lifecycle\.js/,
    );
  } finally {
    await rm(tempDir, { recursive: true, force: true });
  }
});

test("scaffoldPlugin creates a react JavaScript project", async () => {
  const tempDir = await createTempDir("create-onin-plugin-scaffold-react-js-");
  const targetDir = join(tempDir, "react-js-plugin");

  try {
    const result = await scaffoldPlugin(
      createCliOptions(targetDir, "react", "js"),
      baseTemplateDirs,
      adapterTemplateDirs,
    );

    assert.equal(result.targetDir, targetDir);
    const packageJson = await readFile(join(targetDir, "package.json"), "utf8");
    assert.match(packageJson, /"react": "\^18\.3\.1"/);
    assert.match(packageJson, /"react-dom": "\^18\.3\.1"/);
    assert.match(packageJson, /"@vitejs\/plugin-react": "\^4\.3\.4"/);
    assert.doesNotMatch(packageJson, /"typescript": "\^5\.5\.0"/);
    assert.doesNotMatch(packageJson, /"@types\/react"/);
    assert.match(await readFile(join(targetDir, "src", "App.jsx"), "utf8"), /pluginName/);
    assert.match(await readFile(join(targetDir, "src", "main.jsx"), "utf8"), /Smoke Plugin/);
    assert.match(await readFile(join(targetDir, "src", "main.jsx"), "utf8"), /ReactDOM\.createRoot/);
    assert.match(await readFile(join(targetDir, "vite.config.js"), "utf8"), /plugin-react/);
  } finally {
    await rm(tempDir, { recursive: true, force: true });
  }
});

test("scaffoldPlugin rejects unsupported framework and language combinations", async () => {
  await assert.rejects(
    scaffoldPlugin(
      createCliOptions("solid-js-plugin", "solid", "js"),
      baseTemplateDirs,
      adapterTemplateDirs,
    ),
    /Unsupported language for framework: solid\/js/,
  );
});
