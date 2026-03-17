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

Create a React starter instead:

```bash
npx create-onin-plugin my-plugin --framework react
```

Create a Vue starter instead:

```bash
npx create-onin-plugin my-plugin --framework vue
```

## Frameworks

The CLI currently supports:

- `svelte` (default)
- `react`
- `vue`

## Generated project

Every generated plugin includes:

- `src/lifecycle.ts`
- `manifest.json`
- `vite.lifecycle.config.ts`
- `pnpm pack:plugin`

Framework-specific starters add their own UI entry files such as `src/main.ts`, `src/main.tsx`, and framework app components.

The release zip contains:

- `manifest.json`
- `icon.svg`
- `dist/index.html`
- `dist/lifecycle.js`
