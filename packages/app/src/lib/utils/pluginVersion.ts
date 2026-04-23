export function isValidPluginVersion(
  version?: string | null,
): version is string {
  if (!version) {
    return false;
  }

  const normalized = version.trim();
  if (!normalized) {
    return false;
  }

  return normalized.toUpperCase() !== "N/A";
}

export function formatPluginVersion(version?: string | null): string {
  if (!isValidPluginVersion(version)) {
    return "";
  }

  const normalized = version.trim();
  if (/^[vV]/.test(normalized)) {
    return normalized;
  }

  return /^\d/.test(normalized) ? `v${normalized}` : normalized;
}

interface ParsedPluginVersion {
  major: number;
  minor: number;
  patch: number;
  prerelease: string[];
}

function parsePluginVersion(
  version?: string | null,
): ParsedPluginVersion | null {
  if (!isValidPluginVersion(version)) {
    return null;
  }

  const normalized = version.trim().replace(/^[vV]/, "");
  const match = normalized.match(
    /^(\d+)(?:\.(\d+))?(?:\.(\d+))?(?:-([0-9A-Za-z.-]+))?$/,
  );

  if (!match) {
    return null;
  }

  return {
    major: Number.parseInt(match[1] ?? "0", 10),
    minor: Number.parseInt(match[2] ?? "0", 10),
    patch: Number.parseInt(match[3] ?? "0", 10),
    prerelease: match[4]?.split(".").filter(Boolean) ?? [],
  };
}

function comparePrereleaseIdentifiers(left: string[], right: string[]): number {
  if (left.length === 0 && right.length === 0) {
    return 0;
  }

  if (left.length === 0) {
    return 1;
  }

  if (right.length === 0) {
    return -1;
  }

  const maxLength = Math.max(left.length, right.length);

  for (let index = 0; index < maxLength; index += 1) {
    const leftPart = left[index];
    const rightPart = right[index];

    if (leftPart === undefined) {
      return -1;
    }

    if (rightPart === undefined) {
      return 1;
    }

    const leftNumeric = /^\d+$/.test(leftPart);
    const rightNumeric = /^\d+$/.test(rightPart);

    if (leftNumeric && rightNumeric) {
      const diff =
        Number.parseInt(leftPart, 10) - Number.parseInt(rightPart, 10);
      if (diff !== 0) {
        return diff;
      }
      continue;
    }

    if (leftNumeric !== rightNumeric) {
      return leftNumeric ? -1 : 1;
    }

    if (leftPart !== rightPart) {
      return leftPart.localeCompare(rightPart);
    }
  }

  return 0;
}

export function comparePluginVersions(
  leftVersion?: string | null,
  rightVersion?: string | null,
): number {
  const left = parsePluginVersion(leftVersion);
  const right = parsePluginVersion(rightVersion);

  if (!left && !right) {
    return 0;
  }

  if (!left) {
    return -1;
  }

  if (!right) {
    return 1;
  }

  const numericDiff =
    left.major - right.major ||
    left.minor - right.minor ||
    left.patch - right.patch;

  if (numericDiff !== 0) {
    return numericDiff;
  }

  return comparePrereleaseIdentifiers(left.prerelease, right.prerelease);
}
