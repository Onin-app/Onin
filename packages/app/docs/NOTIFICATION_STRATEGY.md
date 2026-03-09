# Notification Strategy (macOS)

## Current behavior

- `macOS + tauri dev`: use `osascript` (`display notification ...`)
- All other cases (including packaged macOS builds): use `tauri-plugin-notification`

Implementation: `packages/app/src-tauri/src/plugin_api/notification.rs`

## Why this split exists

- In local dev mode on macOS, notification behavior through the plugin can be inconsistent.
- `osascript` is a reliable debug-time path and helps isolate app logic from dev-runtime differences.
- For packaged builds, we still use the official plugin to keep behavior aligned with cross-platform architecture.

## Maintenance rules

1. Do not remove the macOS dev fallback unless local dev notification reliability is verified.
2. Do not switch packaged builds to `osascript`; production path should stay on the plugin.
3. If notification issues recur, always test both paths separately:
   - dev path (`tauri dev`)
   - packaged path (`tauri build` output)
