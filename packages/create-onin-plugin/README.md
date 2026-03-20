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

- `src/background.ts`
- `manifest.json`
- `vite.background.config.ts`
- `pnpm pack:plugin`

Framework-specific starters add their own UI entry files such as `src/main.ts`, `src/main.tsx`, and framework app components.

The release zip contains:

- `manifest.json`
- `icon.svg`
- `dist/index.html`
- `dist/background.js`
