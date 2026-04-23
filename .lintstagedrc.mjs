export default {
  '*.{js,cjs,mjs,ts,svelte,json,css,md,html,yml,yaml}': ['prettier --write'],
  'packages/app/src-tauri/**/*.{rs,toml}': () => [
    'cargo fmt --manifest-path packages/app/src-tauri/Cargo.toml',
  ],
};
