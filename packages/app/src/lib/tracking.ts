import { invoke } from "@tauri-apps/api/core";
import { getVersion } from "@tauri-apps/api/app";

const STORAGE_KEY_LAST_ACTIVE = "onin_last_active_date";
const STORAGE_KEY_OPEN_COUNT = "onin_today_open_count";
const STORAGE_KEY_LAST_VERSION = "onin_last_app_version";
const STORAGE_KEY_COMMAND_STATS = "onin_command_stats";
const STORAGE_KEY_PENDING_OPENS = "onin_pending_opens";
const STORAGE_KEY_PENDING_COMMAND_STATS = "onin_pending_command_stats";

interface CommandStats {
  total: number;
  sources: Record<string, number>;
}

let versionCache = "";

async function getCachedVersion(): Promise<string> {
  if (versionCache) return versionCache;
  try {
    versionCache = await getVersion();
  } catch {
    versionCache = "unknown";
  }
  return versionCache;
}

export async function trackEvent(
  name: string,
  props?: Record<string, string | number | boolean>,
): Promise<boolean> {
  try {
    await invoke("plugin:aptabase|track_event", { name, props });
    return true;
  } catch (err) {
    console.error("[Aptabase] track error:", err);
    return false;
  }
}

async function detectVersionChange(): Promise<{
  currentVersion: string;
  previousVersion: string | null;
  isFirstLaunch: boolean;
  isUpgrade: boolean;
}> {
  const currentVersion = await getCachedVersion();
  const previousVersion = localStorage.getItem(STORAGE_KEY_LAST_VERSION);
  const isFirstLaunch = !previousVersion;
  const isUpgrade = !!previousVersion && previousVersion !== currentVersion;

  if (currentVersion !== "unknown" && currentVersion !== previousVersion) {
    localStorage.setItem(STORAGE_KEY_LAST_VERSION, currentVersion);
  }

  return { currentVersion, previousVersion, isFirstLaunch, isUpgrade };
}

// 每次冷启动触发一次，携带完整版本上下文与是否可见标志
export async function trackAppStarted(visible: boolean = true): Promise<void> {
  const { currentVersion, previousVersion, isFirstLaunch, isUpgrade } =
    await detectVersionChange();

  const props: Record<string, string | number | boolean> = {
    app_version: currentVersion,
    is_first_launch: isFirstLaunch,
    is_upgrade: isUpgrade,
    visible,
  };
  if (previousVersion) {
    props.previous_version = previousVersion;
  }

  await trackEvent("app_started", props);
}

// 命令使用本地累积，避免高频上报消耗额度，次日由 daily_active 汇总上报
export function accumulateCommandStat(sourceType: string): void {
  try {
    const raw = localStorage.getItem(STORAGE_KEY_COMMAND_STATS);
    const stats: CommandStats = raw
      ? JSON.parse(raw)
      : { total: 0, sources: {} };

    stats.total += 1;
    stats.sources[sourceType] = (stats.sources[sourceType] || 0) + 1;

    localStorage.setItem(STORAGE_KEY_COMMAND_STATS, JSON.stringify(stats));
  } catch (err) {
    console.error("[Aptabase] accumulate stat error:", err);
  }
}

function collectAndClearCommandStats(): CommandStats {
  try {
    const raw = localStorage.getItem(STORAGE_KEY_COMMAND_STATS);
    localStorage.removeItem(STORAGE_KEY_COMMAND_STATS);
    if (raw) {
      const parsed = JSON.parse(raw);
      if (
        parsed &&
        typeof parsed.total === "number" &&
        parsed.sources &&
        typeof parsed.sources === "object" &&
        !Array.isArray(parsed.sources)
      ) {
        return parsed;
      }
    }
  } catch {
    /* ignore corrupt data */
  }
  return { total: 0, sources: {} };
}

function restorePendingCommandStats(stats: CommandStats): void {
  try {
    const raw = localStorage.getItem(STORAGE_KEY_PENDING_COMMAND_STATS);
    const existing: CommandStats = raw
      ? JSON.parse(raw)
      : { total: 0, sources: {} };

    existing.total += stats.total;
    for (const [source, count] of Object.entries(stats.sources)) {
      existing.sources[source] = (existing.sources[source] || 0) + count;
    }

    localStorage.setItem(
      STORAGE_KEY_PENDING_COMMAND_STATS,
      JSON.stringify(existing),
    );
  } catch {
    localStorage.setItem(
      STORAGE_KEY_PENDING_COMMAND_STATS,
      JSON.stringify(stats),
    );
  }
}

function collectAndClearPendingCommandStats(): CommandStats {
  try {
    const raw = localStorage.getItem(STORAGE_KEY_PENDING_COMMAND_STATS);
    localStorage.removeItem(STORAGE_KEY_PENDING_COMMAND_STATS);
    if (raw) {
      const parsed = JSON.parse(raw);
      if (
        parsed &&
        typeof parsed.total === "number" &&
        parsed.sources &&
        typeof parsed.sources === "object" &&
        !Array.isArray(parsed.sources)
      ) {
        return parsed;
      }
    }
  } catch {
    /* ignore corrupt data */
  }
  return { total: 0, sources: {} };
}

function collectAndClearPendingOpens(): number {
  try {
    const raw = localStorage.getItem(STORAGE_KEY_PENDING_OPENS);
    localStorage.removeItem(STORAGE_KEY_PENDING_OPENS);
    if (raw) {
      const parsed = parseInt(raw, 10);
      return isNaN(parsed) ? 0 : parsed;
    }
  } catch {
    /* ignore */
  }
  return 0;
}

// 日活心跳：每个自然日最多触发一次，跨天时结算昨日统计
let dailyActiveLock = false;

export async function trackDailyActive(): Promise<void> {
  if (dailyActiveLock) return;
  dailyActiveLock = true;

  try {
    const now = new Date();
    const todayStr = `${now.getFullYear()}-${String(now.getMonth() + 1).padStart(2, "0")}-${String(now.getDate()).padStart(2, "0")}`;
    const lastDate = localStorage.getItem(STORAGE_KEY_LAST_ACTIVE);

    const appVersion = await getCachedVersion();

    if (lastDate !== todayStr) {
      const savedCount = localStorage.getItem(STORAGE_KEY_OPEN_COUNT);
      const parsed = savedCount !== null ? parseInt(savedCount, 10) : 0;
      const previousDayOpens = isNaN(parsed) ? 0 : parsed;

      // 如果有之前的活跃日期记录，说明是真实的跨天结算，把上一天的打开次数和命令统计封存进 pending
      if (lastDate) {
        const existingPendingOpens =
          parseInt(
            localStorage.getItem(STORAGE_KEY_PENDING_OPENS) || "0",
            10,
          ) || 0;
        localStorage.setItem(
          STORAGE_KEY_PENDING_OPENS,
          String(existingPendingOpens + previousDayOpens),
        );

        // 只有在真实的跨天结算时，才封存并清空前一天的命令，避免午夜过后的高频操作在跨天触发前被误划归至前一天
        const cmdStats = collectAndClearCommandStats();
        restorePendingCommandStats(cmdStats);
      }

      // 立刻切换状态为新的一天，保证后续激活只做累加而不会重复触发网络请求或结算逻辑
      localStorage.setItem(STORAGE_KEY_LAST_ACTIVE, todayStr);
      localStorage.setItem(STORAGE_KEY_OPEN_COUNT, "1");
    } else {
      const cur = localStorage.getItem(STORAGE_KEY_OPEN_COUNT);
      const next = cur ? parseInt(cur, 10) + 1 : 1;
      localStorage.setItem(STORAGE_KEY_OPEN_COUNT, String(next));
    }

    // 尝试发送累积的 pending 活跃统计
    const pendingOpens = collectAndClearPendingOpens();
    const pendingCmds = collectAndClearPendingCommandStats();

    if (pendingOpens > 0 || pendingCmds.total > 0) {
      const props: Record<string, string | number | boolean> = {
        app_version: appVersion,
        previous_day_opens: pendingOpens,
        total_commands: pendingCmds.total,
      };

      for (const [source, count] of Object.entries(pendingCmds.sources)) {
        const propKey = `cmd_${source.toLowerCase()}`;
        if (count > 0) props[propKey] = count;
      }

      const success = await trackEvent("daily_active", props);

      if (success) {
        // 上报成功，挂起的 counts 和命令均已处理完毕（已在 collect 时清空，不需额外处理）
      } else {
        // 上报失败，将数据退回到 pending 中，留待下次重试
        const currentPendingOpens =
          parseInt(
            localStorage.getItem(STORAGE_KEY_PENDING_OPENS) || "0",
            10,
          ) || 0;
        localStorage.setItem(
          STORAGE_KEY_PENDING_OPENS,
          String(currentPendingOpens + pendingOpens),
        );
        restorePendingCommandStats(pendingCmds);
      }
    }
  } catch (err) {
    console.error("[Aptabase] daily_active error:", err);
  } finally {
    dailyActiveLock = false;
  }
}
