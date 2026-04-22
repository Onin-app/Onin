import * as Sentry from "@sentry/browser";

const GLITCHTIP_DSN = import.meta.env.VITE_GLITCHTIP_DSN?.trim();
const GLITCHTIP_ENVIRONMENT =
  import.meta.env.VITE_GLITCHTIP_ENVIRONMENT ||
  (import.meta.env.DEV ? "development" : "production");

let initialized = false;

export async function initGlitchTip(): Promise<void> {
  if (initialized || typeof window === "undefined") {
    return;
  }

  if (!GLITCHTIP_DSN) {
    return;
  }

  initialized = true;

  const version = import.meta.env.PACKAGE_VERSION || "0.0.0";
  const release = `onin@${version}`;

  Sentry.init({
    dsn: GLITCHTIP_DSN,
    environment: GLITCHTIP_ENVIRONMENT,
    release,
    tracesSampleRate: 0,
    attachStacktrace: true,
    ignoreErrors: [
      "ResizeObserver loop limit exceeded",
      "ResizeObserver loop completed with undelivered notifications.",
    ],
  });

  Sentry.setTag("app", "onin");
  Sentry.setTag("runtime", "tauri-webview");
  Sentry.setTag("layer", "js");
  Sentry.setTag("app_version", version);

  Sentry.setContext("app", {
    version,
  });

  // Double check version from Tauri API if available later
  try {
    const { getVersion } = await import("@tauri-apps/api/app");
    const tauriVersion = await getVersion();
    if (tauriVersion !== version) {
      Sentry.setTag("app_version_native", tauriVersion);
    }
  } catch (error) {
    // Silent fail
  }
}
