<script setup lang="ts">
defineProps<{
  pluginName: string;
  pluginId: string;
}>();
</script>

<template>
  <main class="shell">
    <section class="hero">
      <p class="eyebrow">Onin Plugin</p>
      <h1>{{ pluginName }}</h1>
      <p class="lede">
        This starter includes a dedicated background build, release pack command,
        and a manifest wired for marketplace-safe output.
      </p>
    </section>

    <section class="card">
      <h2>What is ready</h2>
      <ul>
        <li>Vite app build to <code>dist/</code></li>
        <li>Standalone <code>background.js</code> build</li>
        <li><code>pnpm pack</code> for release zip creation</li>
        <li>Manifest and background path already aligned</li>
      </ul>
    </section>

    <section class="card">
      <h2>Plugin ID</h2>
      <code>{{ pluginId }}</code>
    </section>
  </main>
</template>

<style scoped>
:global(body) {
  margin: 0;
  font-family:
    "IBM Plex Sans", "Segoe UI", sans-serif;
  background:
    radial-gradient(circle at top left, rgba(59, 130, 246, 0.18), transparent 35%),
    linear-gradient(180deg, #f7f7f5 0%, #eceae3 100%);
  color: #161616;
}

code {
  font-family:
    "IBM Plex Mono", "Cascadia Code", monospace;
}

.shell {
  min-height: 100vh;
  padding: 28px;
  display: grid;
  gap: 18px;
  align-content: start;
}

.hero,
.card {
  background: rgba(255, 255, 255, 0.78);
  border: 1px solid rgba(22, 22, 22, 0.08);
  border-radius: 24px;
  padding: 24px;
  box-shadow: 0 18px 50px rgba(22, 22, 22, 0.08);
  backdrop-filter: blur(14px);
}

.eyebrow {
  margin: 0 0 8px;
  font-size: 12px;
  letter-spacing: 0.16em;
  text-transform: uppercase;
  color: #5f6368;
}

h1,
h2,
p,
ul {
  margin: 0;
}

h1 {
  font-size: 32px;
  line-height: 1.05;
}

h2 {
  font-size: 18px;
  margin-bottom: 12px;
}

.lede {
  margin-top: 12px;
  max-width: 48ch;
  color: #4b5563;
  line-height: 1.6;
}

ul {
  padding-left: 18px;
  color: #374151;
  line-height: 1.7;
}

@media (max-width: 640px) {
  .shell {
    padding: 16px;
  }

  h1 {
    font-size: 28px;
  }
}
</style>
