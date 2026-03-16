# create-onin-plugin

Monorepo-local CLI for bootstrapping Onin plugins with a marketplace-safe
release layout.

## Usage

Inside the monorepo:

```bash
pnpm create:plugin my-plugin
```

Direct package execution:

```bash
pnpm --filter create-onin-plugin start my-plugin
```

After publish:

```bash
npx create-onin-plugin my-plugin
```

## Current template

The first version ships a single `svelte-view` template with:

- `src/main.ts`
- `src/lifecycle.ts`
- `vite.lifecycle.config.ts`
- `pnpm build`
- `pnpm pack`
