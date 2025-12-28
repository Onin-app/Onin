import { writable } from "svelte/store";

/**
 * A store to trigger input focus requests.
 * Increment the counter to request focus.
 */
export const focusInputTrigger = writable<number>(0);

export function requestInputFocus() {
  focusInputTrigger.update((n) => n + 1);
}
