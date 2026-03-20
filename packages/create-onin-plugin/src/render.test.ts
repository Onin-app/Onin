import { strict as assert } from "node:assert";
import { mkdtemp, mkdir, readFile, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import test from "node:test";

import { copyTemplateDir, renderPackageJson } from "./render.js";
import type { TemplateContext } from "./types.js";

const context: TemplateContext = {
  packageName: "sample-plugin",
  pluginName: "Sample Plugin",
  pluginId: "com.example.sample-plugin",
  pluginDescription: "Sample Plugin plugin for Onin",
  keyword: "sample-plugin",
  settingsImport: ", settings",
  settingsBlock: "  await settings.useSettingsSchema([]);\n",
  settingsNote: "Settings enabled.",
};

async function createTempDir(prefix: string): Promise<string> {
  return mkdtemp(join(tmpdir(), prefix));
}

test("copyTemplateDir copies nested template files and keeps nested skipped names", async () => {
  const sourceDir = await createTempDir("create-onin-plugin-source-");
  const targetDir = await createTempDir("create-onin-plugin-target-");

  try {
    await mkdir(join(sourceDir, "src"), { recursive: true });
    await writeFile(join(sourceDir, "package.fragment.json"), '{"ignored":true}', "utf8");
    await writeFile(join(sourceDir, "src", "package.fragment.json.tpl"), "__PLUGIN_NAME__", "utf8");
    await writeFile(join(sourceDir, "src", "main.ts.tpl"), "export const name = '__PLUGIN_NAME__';", "utf8");

    await copyTemplateDir(sourceDir, targetDir, context, new Set(["package.fragment.json"]));

    assert.equal(await readFile(join(targetDir, "src", "package.fragment.json"), "utf8"), "Sample Plugin");
    assert.equal(
      await readFile(join(targetDir, "src", "main.ts"), "utf8"),
      "export const name = 'Sample Plugin';",
    );
  } finally {
    await Promise.all([
      rm(sourceDir, { recursive: true, force: true }),
      rm(targetDir, { recursive: true, force: true }),
    ]);
  }
});

test("renderPackageJson renders adapter fragments and omits empty record fields", async () => {
  const tempDir = await createTempDir("create-onin-plugin-package-");
  const basePath = join(tempDir, "package.base.json");
  const adapterPath = join(tempDir, "package.fragment.json");
  const targetPath = join(tempDir, "package.json");

  try {
    await writeFile(
      basePath,
      JSON.stringify({
        name: "__PACKAGE_NAME__",
        scripts: {
          build: "vite build",
        },
      }),
      "utf8",
    );
    await writeFile(
      adapterPath,
      JSON.stringify({
        dependencies: {
          "__PACKAGE_NAME__-runtime": "^1.0.0",
        },
      }),
      "utf8",
    );

    await renderPackageJson(basePath, adapterPath, targetPath, context);

    const rendered = JSON.parse(await readFile(targetPath, "utf8")) as {
      name: string;
      scripts?: Record<string, string>;
      dependencies?: Record<string, string>;
      devDependencies?: Record<string, string>;
    };

    assert.deepEqual(rendered, {
      name: "sample-plugin",
      scripts: {
        build: "vite build",
      },
      dependencies: {
        "sample-plugin-runtime": "^1.0.0",
      },
    });
    assert.ok(!("devDependencies" in rendered));
  } finally {
    await rm(tempDir, { recursive: true, force: true });
  }
});

test("renderPackageJson merges vanilla starter scripts without framework runtime deps", async () => {
  const tempDir = await createTempDir("create-onin-plugin-vanilla-package-");
  const basePath = join(tempDir, "package.base.json");
  const adapterPath = join(tempDir, "package.fragment.json");
  const targetPath = join(tempDir, "package.json");

  try {
    await writeFile(
      basePath,
      JSON.stringify({
        name: "__PACKAGE_NAME__",
        scripts: {
          build: "npm run build:index && npm run build:background",
        },
        dependencies: {
          "onin-sdk": "^1.0.0",
        },
      }),
      "utf8",
    );
    await writeFile(
      adapterPath,
      JSON.stringify({
        scripts: {
          dev: "vite",
          "build:index": "vite build",
        },
        devDependencies: {
          typescript: "^5.5.0",
          vite: "^7.3.1",
        },
      }),
      "utf8",
    );

    await renderPackageJson(basePath, adapterPath, targetPath, context);

    const rendered = JSON.parse(await readFile(targetPath, "utf8")) as {
      scripts?: Record<string, string>;
      dependencies?: Record<string, string>;
      devDependencies?: Record<string, string>;
    };

    assert.deepEqual(rendered.scripts, {
      build: "npm run build:index && npm run build:background",
      dev: "vite",
      "build:index": "vite build",
    });
    assert.deepEqual(rendered.dependencies, {
      "onin-sdk": "^1.0.0",
    });
    assert.deepEqual(rendered.devDependencies, {
      typescript: "^5.5.0",
      vite: "^7.3.1",
    });
  } finally {
    await rm(tempDir, { recursive: true, force: true });
  }
});

test("renderPackageJson removes empty record fields left by base or adapter fragments", async () => {
  const tempDir = await createTempDir("create-onin-plugin-empty-records-");
  const basePath = join(tempDir, "package.base.json");
  const adapterPath = join(tempDir, "package.fragment.json");
  const targetPath = join(tempDir, "package.json");

  try {
    await writeFile(
      basePath,
      JSON.stringify({
        name: "__PACKAGE_NAME__",
        scripts: {},
        dependencies: {},
      }),
      "utf8",
    );
    await writeFile(
      adapterPath,
      JSON.stringify({
        devDependencies: {},
      }),
      "utf8",
    );

    await renderPackageJson(basePath, adapterPath, targetPath, context);

    const rendered = JSON.parse(await readFile(targetPath, "utf8")) as {
      name: string;
      scripts?: Record<string, string>;
      dependencies?: Record<string, string>;
      devDependencies?: Record<string, string>;
    };

    assert.deepEqual(rendered, {
      name: "sample-plugin",
    });
  } finally {
    await rm(tempDir, { recursive: true, force: true });
  }
});
