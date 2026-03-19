import { command, definePlugin__SETTINGS_IMPORT__ } from "onin-sdk";

export const setup = async () => {
__SETTINGS_BLOCK__    await command.handle(async (code) => {
    if (code === "open") {
      return {
        ok: true,
      };
    }

    return null;
  });
};

export const mount = async ({ target }) => {
  const { mountPluginUi } = await import("./ui");
  return mountPluginUi({
    target,
    pluginName: "__PLUGIN_NAME__",
    pluginId: "__PLUGIN_ID__",
  });
};

export default definePlugin({
  setup,
  mount,
});
