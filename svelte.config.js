// Tauri doesn't have a Node.js server to do proper SSR
// so we will use adapter-static to prerender the app (SSG)
// See: https://v2.tauri.app/start/frontend/sveltekit/ for more info
import adapter from "@sveltejs/adapter-static";
import { vitePreprocess } from "@sveltejs/vite-plugin-svelte";

/** @type {import('@sveltejs/kit').Config} */
const config = {
  preprocess: vitePreprocess(),
  kit: {
    // adapter: adapter(),

    // 对于 Tauri 应用，我们通常构建一个单页应用 (SPA)。
    // adapter-static 是一个常见的选择，因为它将 SvelteKit 应用构建为静态文件。
    adapter: adapter({
      // 设置 fallback 为 'index.html'，确保所有未匹配的路由都回退到 index.html，
      // 这对于 SPA 模式至关重要，因为 Tauri 仅加载一个 HTML 文件。
      fallback: "index.html",
    }),
    prerender: {
      // 禁用所有路由的预渲染。
      // 对于 Tauri 应用，通常不需要预渲染，因为路由由客户端处理。
      entries: [], // 一个空数组意味着不会预渲染任何路由
    },
  },
};

export default config;
