import { writable } from "svelte/store";

/**
 * A store to hold the currently active handler function for the ESC key.
 * Pages can set their own handler on mount and clear it on destroy.
 */
export const escapeHandler = writable<(() => void) | null>(null);
