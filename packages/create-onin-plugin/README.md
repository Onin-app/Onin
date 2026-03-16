# create-onin-plugin

CLI for bootstrapping Onin plugins with a marketplace-safe release layout.

## Usage

Published usage:

```bash
npx create-onin-plugin my-plugin
```

Monorepo usage:

```bash
pnpm create:plugin my-plugin
```

Direct package execution inside the repo:

```bash
pnpm --filter create-onin-plugin start my-plugin
```

Interactive mode:

```bash
npx create-onin-plugin
```

## Current template

The first version ships a single `svelte-view` template with:

- `src/main.ts`
- `src/lifecycle.ts`
- `vite.lifecycle.config.ts`
- `pnpm build`
- `pnpm pack:plugin`

## Generated project

The generated plugin includes:

- `src/main.ts` for the UI entry
- `src/lifecycle.ts` for settings, commands, and startup initialization
- `manifest.json` wired to `dist/lifecycle.js`
- `pnpm pack:plugin` to create `plugin.zip`

The release zip contains:

- `manifest.json`
- `icon.svg`
- `dist/index.html`
- `dist/lifecycle.js`
