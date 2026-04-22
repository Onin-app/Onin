import { initGlitchTip } from "$lib/glitchtip";

// Initialize Sentry/GlitchTip as early as possible on the client
initGlitchTip().catch((err) => {
  console.error("Failed to initialize GlitchTip:", err);
});
