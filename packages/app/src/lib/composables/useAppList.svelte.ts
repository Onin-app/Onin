/**
 * App List Manager Composable
 *
 * 管理应用列表的状态和逻辑
 * 遵循单一职责原则，只处理应用列表相关功能
 */
import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type { LaunchableItem, CommandUsageStats, AppConfig } from "$lib/type";
import { fuzzyMatch } from "$lib/utils/fuzzyMatch";
import { toast } from "svelte-sonner";

export interface AppListState {
  originAppList: LaunchableItem[];
  appList: LaunchableItem[];
  selectedIndex: number;
  usageStats: CommandUsageStats[];
  appConfig: AppConfig;
  isRefreshing: boolean;
}

export interface OpenAppArgs {
  input?: string;
  text?: string;
  images?: any[];
  textFiles?: any[];
  files?: any[];
  folders?: any[];
}

export interface AppListManagerReturn {
  state: AppListState;
  fetchApps: () => Promise<void>;
  handleInput: (value: string) => void;
  openApp: (
    app: LaunchableItem,
    args: OpenAppArgs,
    onSuccess: () => void,
  ) => Promise<void>;
  handleKeyDown: (
    e: KeyboardEvent,
    displayList: LaunchableItem[],
    onEnter: (app: LaunchableItem) => void,
  ) => void;
  resetSelection: () => void;
  resetToOriginList: () => void;
  loadConfig: () => Promise<void>;
  setupListeners: () => Promise<UnlistenFn>;
}

export function useAppList(): AppListManagerReturn {
  let state = $state<AppListState>({
    originAppList: [],
    appList: [],
    selectedIndex: 0,
    usageStats: [],
    appConfig: {
      auto_paste_time_limit: 5,
      auto_clear_time_limit: 0,
      sort_mode: "smart",
      enable_usage_tracking: true,
    },
    isRefreshing: false,
  });

  const loadConfig = async () => {
    try {
      state.appConfig = await invoke<AppConfig>("get_app_config");
      state.usageStats = await invoke<CommandUsageStats[]>("get_usage_stats");
    } catch (error) {
      console.error("Failed to load config or usage stats:", error);
    }
  };

  const fetchApps = async () => {
    try {
      const res = await invoke<LaunchableItem[]>("get_all_launchable_items");
      if (res) {
        state.originAppList = res;
        state.appList = res;
      }
    } catch (error) {
      console.error("Failed to get all launchable items:", error);
    }
  };

  const handleInput = (value: string) => {
    const apps = fuzzyMatch(value, state.originAppList);
    state.appList = apps;
    state.selectedIndex = 0;
  };

  const openApp = async (
    app: LaunchableItem,
    args: OpenAppArgs,
    onSuccess: () => void,
  ) => {
    try {
      if (app.action) {
        await invoke("execute_command", {
          name: app.action,
          args: Object.keys(args).length > 0 ? args : null,
        });

        Promise.all([
          invoke<CommandUsageStats[]>("get_usage_stats"),
          invoke<LaunchableItem[]>("get_all_launchable_items"),
        ])
          .then(([stats, items]) => {
            state.usageStats = stats;
            state.originAppList = items;
          })
          .catch((err) => console.error("Failed to refresh data:", err));
      } else if (app.source === "FileCommand") {
        await invoke("open_app", {
          path: app.path,
        });
      }

      onSuccess();
    } catch (error) {
      console.error("Failed to open app:", error);
      toast.error("运行失败", {
        description: String(error),
        duration: 6000,
      });
    }
  };

  const handleKeyDown = (
    e: KeyboardEvent,
    displayList: LaunchableItem[],
    onEnter: (app: LaunchableItem) => void,
  ) => {
    if (e.key === "ArrowDown" || (e.key === "Tab" && !e.shiftKey)) {
      e.preventDefault();
      state.selectedIndex =
        state.selectedIndex === displayList.length - 1
          ? 0
          : state.selectedIndex + 1;
    } else if (e.key === "ArrowUp" || (e.key === "Tab" && e.shiftKey)) {
      e.preventDefault();
      state.selectedIndex =
        state.selectedIndex === 0
          ? displayList.length - 1
          : state.selectedIndex - 1;
    } else if (e.key === "Enter" && displayList.length > 0) {
      e.preventDefault();
      onEnter(displayList[state.selectedIndex]);
    }

    const container = document.querySelector(".app-list");
    const selectedItem = container?.children[state.selectedIndex];
    if (selectedItem) {
      selectedItem.scrollIntoView({
        behavior: "auto",
        block: "nearest",
      });
    }
  };

  const resetSelection = () => {
    state.selectedIndex = 0;
  };

  const resetToOriginList = () => {
    state.appList = state.originAppList;
    state.selectedIndex = 0;
  };

  const setupListeners = async (): Promise<UnlistenFn> => {
    const unlistenAppsUpdated = await listen("apps_updated", () => {
      fetchApps();
    });

    const unlistenCommandsReady = await listen("commands_ready", () => {
      fetchApps();
    });

    const unlistenRefreshStarted = await listen("refresh_started", () => {
      state.isRefreshing = true;
    });

    const unlistenRefreshEnded = await listen<{
      previous_count: number;
      current_count: number;
      added: number;
    }>("commands_refreshed", async (event) => {
      state.isRefreshing = false;
      fetchApps();

      const { current_count, added } = event.payload;
      let message = `共 ${current_count} 项`;
      if (added > 0) {
        message += `，新增 ${added} 项`;
      } else if (added < 0) {
        message += `，减少 ${Math.abs(added)} 项`;
      }
      await invoke("show_notification", {
        options: {
          title: "刷新成功",
          body: message,
        },
      });
    });

    return () => {
      unlistenAppsUpdated();
      unlistenCommandsReady();
      unlistenRefreshStarted();
      unlistenRefreshEnded();
    };
  };

  return {
    get state() {
      return state;
    },
    fetchApps,
    handleInput,
    openApp,
    handleKeyDown,
    resetSelection,
    resetToOriginList,
    loadConfig,
    setupListeners,
  };
}
