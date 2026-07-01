/**
 * Extension 执行动作注册表
 *
 * 每个 Extension 命令声明自己的执行策略，主逻辑只负责查表分发。
 * 新增 Extension 时只需在此文件中注册，无需修改 +page.svelte。
 *
 * 三种执行策略：
 * - navigate:    跳转到指定路由（可携带 query 参数）
 * - execute:     调用 extensionManager.execute，结果可复制，然后关闭窗口
 * - color-pick:  启动特殊拾色流程
 */

// ============================================================================
// 类型定义
// ============================================================================

/** Extension 执行上下文 */
export interface ExtensionContext {
  /** 有效文本（粘贴文本 > 输入框文本） */
  effectiveText: string;
  /** 触发模式 */
  triggerMode?: "matched" | "preview";
}

/**
 * 导航策略
 *
 * route 函数接收上下文，返回目标路由字符串。
 * 若 queryOnlyWhenMatched 为 true，则只有在 matched/preview 模式下才将
 * effectiveText 作为 query 参数传入，功能指令直接打开时不携带。
 */
type NavigateAction = {
  type: "navigate";
  route: (ctx: ExtensionContext) => string;
  /** true 时仅 matched/preview 模式才带 query（区分功能指令和匹配指令） */
  queryOnlyWhenMatched?: boolean;
};

/**
 * 执行策略
 *
 * 调用 extensionManager.execute，若有可复制结果则写入剪贴板，然后关闭窗口。
 */
type ExecuteAction = {
  type: "execute";
};

/**
 * 拾色策略
 *
 * 启动颜色拾取流程（特殊 UI 流程，无法用通用策略覆盖）。
 */
type ColorPickAction = {
  type: "color-pick";
};

export type ExtensionAction = NavigateAction | ExecuteAction | ColorPickAction;

// ============================================================================
// 注册表
//
// key 格式："{extensionId}:{commandCode}"
// ============================================================================

const encode = encodeURIComponent;

export const EXTENSION_ACTION_MAP: Record<string, ExtensionAction> = {
  // ── 文件搜索 ──────────────────────────────────────────────────────────────
  "file_search:search": {
    type: "navigate",
    route: () => "/extensions/filesearch",
  },

  // ── AI ────────────────────────────────────────────────────────────────────
  "ai:chat": {
    type: "navigate",
    route: () => "/extensions/ai",
  },

  // ── Emoji ─────────────────────────────────────────────────────────────────
  "emoji:search": {
    type: "navigate",
    route: () => "/extensions/emoji",
  },

  // ── 书签 ──────────────────────────────────────────────────────────────────
  "bookmarks:search": {
    type: "navigate",
    route: (ctx) =>
      ctx.effectiveText
        ? `/extensions/bookmarks?q=${encode(ctx.effectiveText)}`
        : "/extensions/bookmarks",
    queryOnlyWhenMatched: true,
  },

  // ── 剪贴板 ────────────────────────────────────────────────────────────────
  "clipboard:history": {
    type: "navigate",
    route: () => "/extensions/clipboard",
  },

  // ── 颜色 ──────────────────────────────────────────────────────────────────
  "color:pick": {
    type: "color-pick",
  },
  "color:convert": {
    type: "navigate",
    route: (ctx) =>
      ctx.effectiveText
        ? `/extensions/color?q=${encode(ctx.effectiveText)}`
        : "/extensions/color",
    queryOnlyWhenMatched: true,
  },

  // ── 翻译 ──────────────────────────────────────────────────────────────────
  // 翻译在后端打开新窗口，只需 execute 即可，不需要前端路由跳转
  "translator:open": {
    type: "execute",
  },

  // ── Web ───────────────────────────────────────────────────────────────────
  // web extension 的命令（open_url / search_google / search_bing / search_baidu）
  // 均走 execute 路径，由后端打开浏览器
  "web:open_url": { type: "execute" },
  "web:search_google": { type: "execute" },
  "web:search_bing": { type: "execute" },
  "web:search_baidu": { type: "execute" },

  // ── 计算器 ────────────────────────────────────────────────────────────────
  // calculator 通常以预览项出现，点击后执行命令并复制结果
  "calculator:calculate": { type: "execute" },

  // ── 文字识别 (OCR) ────────────────────────────────────────────────────────
  "ocr:recognize": {
    type: "navigate",
    route: () => "/extensions/ocr",
  },

  // ── 录屏 ──────────────────────────────────────────────────────────────────
  "screen_recorder:record": {
    type: "navigate",
    route: () => "/extensions/screen-recorder",
  },
};

// ============================================================================
// 工具函数
// ============================================================================

/**
 * 根据 extensionId + commandCode 查找动作配置
 */
export function resolveExtensionAction(
  extensionId: string,
  commandCode: string,
): ExtensionAction | null {
  const key = `${extensionId}:${commandCode}`;
  return EXTENSION_ACTION_MAP[key] ?? null;
}

/**
 * 为 navigate 策略计算最终路由
 *
 * 若 queryOnlyWhenMatched 为 true，且当前不是 matched/preview 模式，
 * 则不传入 effectiveText（功能指令直接打开，不携带搜索词）。
 */
export function buildNavigateRoute(
  action: NavigateAction,
  ctx: ExtensionContext,
): string {
  const effectiveCtx: ExtensionContext =
    action.queryOnlyWhenMatched &&
    ctx.triggerMode !== "matched" &&
    ctx.triggerMode !== "preview"
      ? { ...ctx, effectiveText: "" }
      : ctx;

  return action.route(effectiveCtx);
}
