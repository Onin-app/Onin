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

Create a Vanilla TypeScript starter instead:

```bash
npx create-onin-plugin my-plugin --framework vanilla
```

Create a Vanilla JavaScript starter instead:

```bash
npx create-onin-plugin my-plugin --framework vanilla --language js
```

Create a React JavaScript starter instead:

```bash
npx create-onin-plugin my-plugin --framework react --language js
```

Create a Vue JavaScript starter instead:

```bash
npx create-onin-plugin my-plugin --framework vue --language js
```

Create a Svelte JavaScript starter instead:

```bash
npx create-onin-plugin my-plugin --framework svelte --language js
```

Create a Solid JavaScript starter instead:

```bash
npx create-onin-plugin my-plugin --framework solid --language js
```

Create a Solid starter instead:

```bash
npx create-onin-plugin my-plugin --framework solid
```

## Frameworks

The CLI currently supports:

- `svelte` (default)
- `react`
- `vue`
- `vanilla`
- `solid`

## Languages

- `ts` (default)
- `js` for `vanilla`, `react`, `vue`, `svelte`, and `solid`

## Generated project

Every generated plugin includes:

- `src/plugin.ts` or `src/plugin.js`
- `src/background.ts` or `src/background.js`
- `manifest.json`
- `scripts/build.mjs`
- `pnpm pack:plugin`

Framework-specific starters add their own UI mount helpers and app components, while `src/main.*` stays as a thin shared wrapper.

The release zip contains:

- `manifest.json`
- `icon.svg`
- `dist/index.html`
- `dist/lifecycle.js`
