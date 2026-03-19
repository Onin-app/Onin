import { lifecycle } from "./api/lifecycle";

export interface PluginSetupContext {}

export interface PluginUiMountContext {
  target: HTMLElement;
}

export type PluginCleanup = () => void | Promise<void>;
export type PluginMountResult = void | PluginCleanup;
type PluginRuntimeEvent = "show" | "hide" | "focus" | "blur" | "cleanup";

export interface OninPluginDefinition {
  setup?: (context: PluginSetupContext) => void | Promise<void>;
  mount?: (
    context: PluginUiMountContext,
  ) => PluginMountResult | Promise<PluginMountResult>;
}

export function definePlugin<T extends OninPluginDefinition>(plugin: T): T {
  return plugin;
}

export function setupPlugin(
  plugin: OninPluginDefinition,
  context: PluginSetupContext = {},
): void {
  if (plugin.setup) {
    lifecycle.onLoad(() => plugin.setup?.(context));
  }
}

export async function mountPlugin(
  plugin: OninPluginDefinition,
  target: HTMLElement,
): Promise<PluginCleanup | undefined> {
  if (!plugin.mount) {
    throw new Error("Plugin mount is not defined.");
  }

  const result = await plugin.mount({ target });
  if (typeof result !== "function") {
    return undefined;
  }

  let settled = false;
  let removeListeners = () => {};

  const cleanupOnce: PluginCleanup = async () => {
    if (settled) {
      return;
    }

    settled = true;
    removeListeners();
    await result();
  };

  if (typeof window !== "undefined") {
    const onRuntimeMessage = (event: MessageEvent) => {
      const data = event.data;
      if (
        !data ||
        typeof data !== "object" ||
        data.type !== "plugin-lifecycle-event"
      ) {
        return;
      }

      const runtimeEvent = data.event as PluginRuntimeEvent;
      if (runtimeEvent === "cleanup") {
        void cleanupOnce();
      }
    };
    const onPageHide = () => {
      void cleanupOnce();
    };
    const onBeforeUnload = () => {
      void cleanupOnce();
    };

    window.addEventListener("message", onRuntimeMessage);
    window.addEventListener("pagehide", onPageHide, { once: true });
    window.addEventListener("beforeunload", onBeforeUnload, { once: true });

    removeListeners = () => {
      window.removeEventListener("message", onRuntimeMessage);
      window.removeEventListener("pagehide", onPageHide);
      window.removeEventListener("beforeunload", onBeforeUnload);
    };
  }

  return cleanupOnce;
}
