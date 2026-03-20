import { command, lifecycle__SETTINGS_IMPORT__ } from "onin-sdk";

lifecycle.onLoad(async () => {
__SETTINGS_BLOCK__  await command.handle(async (code) => {
    if (code === "open") {
      return {
        ok: true,
      };
    }

    return null;
  });
});
