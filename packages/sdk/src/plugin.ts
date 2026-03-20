import { lifecycle } from "./api/lifecycle";

export interface PluginUiMountContext {
  target: HTMLElement;
}

export interface PluginUiDefinition {
  mount: (context: PluginUiMountContext) => unknown | Promise<unknown>;
}

export interface OninPluginDefinition {
  background?: () => void | Promise<void>;
  ui?: PluginUiDefinition;
}

export function definePlugin<T extends OninPluginDefinition>(plugin: T): T {
  return plugin;
}

export function registerPluginBackground(
  background?: OninPluginDefinition["background"],
): void {
  if (background) {
    lifecycle.onLoad(background);
  }
}

export async function mountPluginUi(
  ui: OninPluginDefinition["ui"],
  target: HTMLElement,
): Promise<unknown> {
  if (!ui?.mount) {
    throw new Error("Plugin UI mount is not defined.");
  }

  return ui.mount({ target });
}
