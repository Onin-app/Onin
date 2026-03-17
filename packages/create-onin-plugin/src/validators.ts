export function toTitleCase(value: string): string {
  return value
    .split(/[-_.\s]+/)
    .filter(Boolean)
    .map((part) => part.charAt(0).toUpperCase() + part.slice(1))
    .join(" ");
}

export function slugify(value: string): string {
  return value
    .trim()
    .toLowerCase()
    .replace(/[^a-z0-9.-]+/g, "-")
    .replace(/^-+|-+$/g, "")
    .replace(/-{2,}/g, "-");
}

export function isValidPluginId(value: string): boolean {
  return /^[a-z0-9][a-z0-9.-]*[a-z0-9]$/.test(value) && !value.includes("..");
}

export function isValidPackageName(value: string): boolean {
  return /^[a-z0-9][a-z0-9.-]*[a-z0-9]$/.test(value);
}

export function isNodeError(error: unknown): error is NodeJS.ErrnoException {
  return (
    typeof error === "object" &&
    error !== null &&
    "code" in error &&
    typeof (error as { code?: unknown }).code === "string"
  );
}
