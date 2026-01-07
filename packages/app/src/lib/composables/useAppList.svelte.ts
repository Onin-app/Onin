/**
 * App List Manager Composable
 *
 * 管理应用列表的状态和逻辑
 * 遵循单一职责原则，只处理应用列表相关功能
 */
import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { WebviewWindow } from "@tauri-apps/api/webviewWindow";
import type { LaunchableItem, CommandUsageStats, AppConfig } from "$lib/type";
import { fuzzyMatch } from "$lib/utils/fuzzyMatch";
import { sortByUsage } from "$lib/utils/sortByUsage";

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
    // State (reactive)
    state: AppListState;
    // Methods
    fetchApps: () => Promise<void>;
    handleInput: (value: string) => void;
    openApp: (
        app: LaunchableItem,
        args: OpenAppArgs,
        onSuccess: () => void
    ) => Promise<void>;
    handleKeyDown: (
        e: KeyboardEvent,
        displayList: LaunchableItem[],
        onEnter: (app: LaunchableItem) => void
    ) => void;
    resetSelection: () => void;
    resetToOriginList: () => void;
    loadConfig: () => Promise<void>;
    // Lifecycle
    setupListeners: () => Promise<UnlistenFn>;
}

/**
 * 创建应用列表管理器
 *
 * 使用 Svelte 5 runes 管理应用列表状态
 */
export function useAppList(): AppListManagerReturn {
    // ===== State =====
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

    // ===== Methods =====

    /**
     * 加载配置和使用统计
     */
    const loadConfig = async () => {
        try {
            state.appConfig = await invoke<AppConfig>("get_app_config");
            state.usageStats = await invoke<CommandUsageStats[]>("get_usage_stats");
            console.log("Loaded config and usage stats:", {
                appConfig: state.appConfig,
                usageStats: state.usageStats,
            });
        } catch (error) {
            console.error("Failed to load config or usage stats:", error);
        }
    };

    /**
     * 获取应用列表
     */
    const fetchApps = async () => {
        try {
            console.log("Fetching all launchable items...");
            const res = await invoke<LaunchableItem[]>("get_all_launchable_items");
            console.log("本机软件列表: ", res);
            if (res) {
                state.originAppList = res;
                state.appList = res;
            }
            console.log(`Got ${state.appList.length} apps.`);
        } catch (error) {
            console.error("Failed to get all launchable items:", error);
        }
    };

    /**
     * 处理搜索输入
     */
    const handleInput = (value: string) => {
        let apps = fuzzyMatch(value, state.originAppList);
        apps = sortByUsage(
            apps,
            state.usageStats,
            state.appConfig.sort_mode,
            state.appConfig.enable_usage_tracking
        );
        state.appList = apps;
        state.selectedIndex = 0;
    };

    /**
     * 打开应用
     */
    const openApp = async (
        app: LaunchableItem,
        args: OpenAppArgs,
        onSuccess: () => void
    ) => {
        try {
            if (app.action) {
                await invoke("execute_command", {
                    name: app.action,
                    window: await WebviewWindow.getCurrent(),
                    args: Object.keys(args).length > 0 ? args : null,
                });

                // 刷新使用统计和列表（异步，不阻塞）
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
                    window: await WebviewWindow.getCurrent(),
                });
            }

            onSuccess();
        } catch (error) {
            console.error("Failed to open app:", error);
        }
    };

    /**
     * 处理键盘导航
     */
    const handleKeyDown = (
        e: KeyboardEvent,
        displayList: LaunchableItem[],
        onEnter: (app: LaunchableItem) => void
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

        // 保持选中项在可见范围内
        const container = document.querySelector(".app-list");
        const selectedItem = container?.children[state.selectedIndex];
        if (selectedItem) {
            selectedItem.scrollIntoView({
                behavior: "auto",
                block: "nearest",
            });
        }
    };

    /**
     * 重置选中索引
     */
    const resetSelection = () => {
        state.selectedIndex = 0;
    };

    /**
     * 重置为原始列表
     */
    const resetToOriginList = () => {
        state.appList = state.originAppList;
        state.selectedIndex = 0;
    };

    /**
     * 设置事件监听器
     */
    const setupListeners = async (): Promise<UnlistenFn> => {
        // 监听应用更新事件
        const unlistenAppsUpdated = await listen("apps_updated", () => {
            console.log("Received apps_updated event from backend. Refetching list...");
            fetchApps();
        });

        // 监听命令就绪事件
        const unlistenCommandsReady = await listen("commands_ready", () => {
            console.log("Received commands_ready event from backend. Refetching list...");
            fetchApps();
        });

        // 监听刷新开始事件
        const unlistenRefreshStarted = await listen("refresh_started", () => {
            console.log("Refresh started");
            state.isRefreshing = true;
        });

        // 监听刷新结束事件
        const unlistenRefreshEnded = await listen<{
            previous_count: number;
            current_count: number;
            added: number;
        }>("commands_refreshed", async (event) => {
            console.log("Refresh ended", event.payload);
            state.isRefreshing = false;
            fetchApps();

            // 显示通知
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
