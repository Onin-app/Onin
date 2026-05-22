import { defineConfig } from 'vitest/config';
import { resolve } from 'node:path';
import { svelte } from '@sveltejs/vite-plugin-svelte';

export default defineConfig({
  plugins: [
    svelte({
      compilerOptions: {
        dev: true,
      },
    }),
  ],
  resolve: {
    alias: {
      $lib: resolve(__dirname, 'src/lib'),
      $app: resolve(__dirname, 'src/mocks/sveltekit'),
    },
    conditions: ['browser'],
  },
  test: {
    globals: true,
    environment: 'jsdom',
    include: ['src/**/*.test.ts'],
    setupFiles: ['src/lib/mocks/vitest-setup.ts'],
    coverage: {
      provider: 'v8',
      reporter: ['text', 'json', 'html'],
      include: ['src/lib/**/*.ts'],
      exclude: [
        'src/**/*.test.ts',
        'src/**/__tests__/**',
        'src/lib/mocks/**',
      ],
    },
  },
});
