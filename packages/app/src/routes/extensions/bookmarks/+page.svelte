<script lang="ts">
  /**
   * Bookmarks Extension Page
   *
   * 独立的浏览器书签检索与过滤页面
   */
  import { onMount } from "svelte";
  import { goto } from "$app/navigation";
  import { page } from "$app/stores";
  import { invoke } from "@tauri-apps/api/core";
  import AppScrollArea from "$lib/components/AppScrollArea.svelte";
  import ExtensionHeader from "$lib/components/ExtensionHeader.svelte";
  import PhosphorIcon from "$lib/components/PhosphorIcon.svelte";
  import { toast } from "svelte-sonner";

  interface BookmarkItem {
    title: string;
    url: string;
    browser: string;
    folder: string;
  }

  let searchQuery = $state("");
  let selectedBrowser = $state("all"); // "all", "chrome", "safari", "edge", "brave", "arc"
  let allBookmarks = $state<BookmarkItem[]>([]);
  let selectedIndex = $state(0);
  let isLoading = $state(true);
  let headerRef = $state<ExtensionHeader>(null!);
  let isRefreshingBookmarks = $state(false);
  let isMac = $state(false);

  // 核心书签过滤纯函数：提取高内聚的过滤算法，杜绝逻辑重复，同时向前端衍生和同步时序判断提供统一支撑
  function filterBookmarksList(
    list: BookmarkItem[],
    query: string,
    filter: string,
  ): BookmarkItem[] {
    const q = query.trim().toLowerCase();
    const f = filter.trim().toLowerCase();

    return list.filter((item) => {
      // 浏览器过滤
      if (f !== "all" && item.browser.toLowerCase() !== f) {
        return false;
      }
      // 搜索词过滤
      if (q) {
        const titleMatch = item.title.toLowerCase().includes(q);
        const urlMatch = item.url.toLowerCase().includes(q);
        return titleMatch || urlMatch;
      }
      return true;
    });
  }

  // 在前端进行极速的响应式过滤，实现 0 延迟切换 Tab 和模糊搜索。
  // 通过限制最多渲染 100 条数据，彻底消除了过量 DOM 重绘导致的点击卡顿和卡肉感。
  let filteredBookmarks = $derived.by(() => {
    return filterBookmarksList(
      allBookmarks,
      searchQuery,
      selectedBrowser,
    ).slice(0, 100);
  });

  // 支持的浏览器配置映射
  const BROWSER_CONFIGS: Record<
    string,
    { name: string; icon: string; color: string }
  > = {
    chrome: {
      name: "Chrome",
      icon: "googleLogo",
      color: "text-blue-500 dark:text-blue-400",
    },
    safari: {
      name: "Safari",
      icon: "compass",
      color: "text-sky-500 dark:text-sky-400",
    },
    edge: {
      name: "Edge",
      icon: "browser",
      color: "text-teal-500 dark:text-teal-400",
    },
    brave: {
      name: "Brave",
      icon: "shieldCheck",
      color: "text-orange-500 dark:text-orange-400",
    },
    arc: {
      name: "Arc",
      icon: "sparkle",
      color: "text-violet-500 dark:text-violet-400",
    },
    firefox: {
      name: "Firefox",
      icon: "globe",
      color: "text-orange-600 dark:text-orange-500",
    },
    opera: {
      name: "Opera",
      icon: "appWindow",
      color: "text-red-500 dark:text-red-400",
    },
    vivaldi: {
      name: "Vivaldi",
      icon: "globe",
      color: "text-rose-500 dark:text-rose-400",
    },
  };

  // 动态活跃浏览器列表，通过 allBookmarks 响应式衍生得出，不仅杜绝了重复扫描，而且保证了任何时候 Tab 栏绝对实时、不过时！
  let activeBrowsers = $derived.by(() => {
    // 提取所有在书签中真实存在的浏览器名称，并去重
    const detected = Array.from(
      new Set(allBookmarks.map((b) => b.browser.toLowerCase())),
    );
    detected.sort();

    const dynamicBrowsers = detected.map((id) => {
      const config = BROWSER_CONFIGS[id] || {
        name: id.charAt(0).toUpperCase() + id.slice(1),
        icon: "bookmark",
        color: "text-neutral-500",
      };
      return {
        id,
        name: config.name,
        icon: config.icon,
        color: config.color,
      };
    });

    return [
      { id: "all", name: "全部", icon: "bookmark", color: "text-neutral-500" },
      ...dynamicBrowsers,
    ];
  });

  // 从 URL 参数获取初始搜索值
  const initialQuery = $derived($page.url.searchParams.get("q") || "");

  // 从后端一次性获取全量书签数据，交由前端在内存中快速过滤
  async function loadAllBookmarks(force: boolean = false) {
    try {
      isLoading = true;
      const data = await invoke<BookmarkItem[]>("get_bookmarks_data", {
        searchQuery: "",
        browserFilter: "all",
        forceReload: force,
      });
      allBookmarks = data || [];

      // 模拟过滤得到准确的当前长度，通过统一函数调用避免重复过滤逻辑，且切断 $derived 异步 Tick 时序依赖
      const currentFilteredCount = filterBookmarksList(
        allBookmarks,
        searchQuery,
        selectedBrowser,
      ).slice(0, 100).length;

      // 重置选中索引，防止越界
      if (selectedIndex >= currentFilteredCount) {
        selectedIndex = Math.max(0, currentFilteredCount - 1);
      }
    } catch (error) {
      console.error("[Bookmarks] Failed to load bookmarks:", error);
    } finally {
      isLoading = false;
    }
  }

  // 手动强制同步书签
  async function handleRefresh() {
    if (isRefreshingBookmarks) return;
    try {
      isRefreshingBookmarks = true;
      await loadAllBookmarks(true);
      toast.success("已成功同步最新书签");
    } catch (e) {
      console.error("[Bookmarks] Failed to refresh bookmarks:", e);
      toast.error("同步书签失败");
    } finally {
      isRefreshingBookmarks = false;
      headerRef?.focus();
    }
  }

  // 切换浏览器过滤
  function selectBrowser(browserId: string) {
    selectedBrowser = browserId;
    selectedIndex = 0;
    // 数据在前端过滤，瞬时更新，无需调用后端 API
  }

  // 处理搜索输入
  const handleSearch = (_value: string) => {
    selectedIndex = 0;
    // 搜索词已双向绑定至 searchQuery，此处仅需重置选中索引，自动触发前端极速响应式过滤
  };

  // 返回主页面
  const handleBack = () => {
    goto("/");
  };

  // 键盘操作处理：支持上下方向键和回车键
  const handleKeyDown = (e: KeyboardEvent) => {
    if (e.key === "Backspace" && searchQuery === "") {
      handleBack();
      return;
    }

    if (filteredBookmarks.length === 0) return;

    if (e.key === "ArrowDown") {
      e.preventDefault();
      selectedIndex = (selectedIndex + 1) % filteredBookmarks.length;
      scrollToSelected();
    } else if (e.key === "ArrowUp") {
      e.preventDefault();
      selectedIndex =
        (selectedIndex - 1 + filteredBookmarks.length) %
        filteredBookmarks.length;
      scrollToSelected();
    } else if (e.key === "Enter") {
      e.preventDefault();
      openBookmark(filteredBookmarks[selectedIndex]);
    } else if (e.key === "c" && (e.metaKey || e.ctrlKey)) {
      // 如果输入框内有选中的文本，允许默认的复制行为，不进行拦截
      const activeEl = document.activeElement as HTMLInputElement;
      if (
        activeEl &&
        (activeEl.tagName === "INPUT" || activeEl.tagName === "TEXTAREA") &&
        activeEl.selectionStart !== activeEl.selectionEnd
      ) {
        return;
      }
      e.preventDefault();
      copyBookmarkUrl(filteredBookmarks[selectedIndex]);
    }
  };

  // 打开书签
  async function openBookmark(item: BookmarkItem) {
    try {
      // 使用 web 扩展的 open_url 命令在默认浏览器中打开网址
      await invoke("execute_extension", {
        extensionId: "web",
        commandCode: "open_url",
        input: item.url,
      });
      // 成功打开后关闭主窗口
      invoke("close_main_window");
    } catch (error) {
      console.error("[Bookmarks] Failed to open bookmark:", error);
    }
  }

  // 复制链接
  async function copyBookmarkUrl(item: BookmarkItem) {
    try {
      await navigator.clipboard.writeText(item.url);
      toast.success("已复制书签网址到剪贴板");
    } catch (e) {
      console.error("[Bookmarks] Failed to copy URL:", e);
      toast.error("复制网址失败");
    }
  }

  // 自动滚动到选中的项目
  const scrollToSelected = () => {
    setTimeout(() => {
      document
        .getElementById(`bookmark-item-${selectedIndex}`)
        ?.scrollIntoView({ block: "nearest" });
    }, 0);
  };

  // 获取浏览器对应的图标名称
  function getBrowserIcon(browserName: string): string {
    const lower = browserName.toLowerCase();
    return BROWSER_CONFIGS[lower]?.icon || "bookmark";
  }

  // 获取浏览器图标的特有颜色
  function getBrowserColorClass(browserName: string): string {
    const lower = browserName.toLowerCase();
    return BROWSER_CONFIGS[lower]?.color || "text-neutral-500";
  }

  // 根据 URL 获取高分辨率站点 Favicon 图标
  function getFaviconUrl(urlStr: string): string {
    try {
      const url = new URL(urlStr);
      // 使用 Google API 并获取 64x64 高清图标
      return `https://www.google.com/s2/favicons?domain=${url.hostname}&sz=64`;
    } catch (e) {
      return "";
    }
  }

  onMount(async () => {
    isMac = /macintosh|mac os x/i.test(navigator.userAgent);
    // 使用 URL 参数初始化搜索值
    searchQuery = initialQuery;
    // 每次进入页面时强制进行一次全量扫描，保证初始数据最新
    await loadAllBookmarks(true);

    // 自动聚焦头部输入框
    headerRef?.focus();
  });
</script>

{#snippet right()}
  <button
    class="flex h-10 w-10 cursor-pointer items-center justify-center rounded-lg text-neutral-600 transition-colors hover:bg-neutral-200 dark:text-neutral-400 dark:hover:bg-neutral-700"
    onclick={handleRefresh}
    title="同步书签"
    aria-label="同步书签"
  >
    <PhosphorIcon
      icon="arrowsClockwise"
      size={20}
      class={isRefreshingBookmarks ? "animate-spin" : ""}
    />
  </button>
{/snippet}

<div class="flex h-full w-full flex-col overflow-hidden">
  <!-- 头部搜索栏 -->
  <ExtensionHeader
    bind:this={headerRef}
    placeholder="输入名称或网址搜索书签..."
    bind:value={searchQuery}
    onInput={handleSearch}
    onBack={handleBack}
    onKeyDown={handleKeyDown}
    {right}
  />

  <!-- 浏览器源分类 Tab 栏 -->
  <div
    class="flex flex-shrink-0 gap-1 border-b border-neutral-200/50 px-4 py-2 dark:border-neutral-700/50"
  >
    {#each activeBrowsers as browser (browser.id)}
      <button
        class="flex items-center gap-1.5 rounded-lg px-3 py-1.5 text-xs font-medium transition-all duration-200 focus:outline-none {selectedBrowser ===
        browser.id
          ? 'bg-neutral-200 text-neutral-900 shadow-sm dark:bg-neutral-700 dark:text-neutral-100'
          : 'text-neutral-500 hover:bg-neutral-100 dark:hover:bg-neutral-800'}"
        onclick={() => selectBrowser(browser.id)}
      >
        <PhosphorIcon icon={browser.icon} size={14} class={browser.color} />
        <span>{browser.name}</span>
      </button>
    {/each}
  </div>

  <!-- 内容展示区域 -->
  <div class="flex-1 overflow-hidden">
    {#if isLoading && filteredBookmarks.length === 0}
      <div
        class="flex h-full items-center justify-center text-sm text-neutral-500"
      >
        <div class="flex flex-col items-center gap-3">
          <div
            class="h-6 w-6 animate-spin rounded-full border-2 border-neutral-300 border-t-neutral-600 dark:border-neutral-600 dark:border-t-neutral-300"
          ></div>
          <span>读取书签中...</span>
        </div>
      </div>
    {:else if filteredBookmarks.length === 0}
      <div class="flex h-full items-center justify-center px-6">
        <div
          class="flex max-w-sm flex-col items-center gap-4 text-center text-sm text-neutral-500"
        >
          <div
            class="flex h-14 w-14 items-center justify-center rounded-2xl bg-neutral-100 text-neutral-400 dark:bg-neutral-800"
          >
            <PhosphorIcon icon="bookmarkSimple" size={28} />
          </div>
          <div class="space-y-1">
            <div class="font-semibold text-neutral-800 dark:text-neutral-200">
              没有找到匹配的书签
            </div>
            <div class="text-xs text-neutral-400">
              请检查关键词或尝试切换顶部的浏览器分类过滤
            </div>
          </div>
        </div>
      </div>
    {:else}
      <AppScrollArea class="h-full w-full" viewportClass="h-full w-full p-2">
        <div class="flex flex-col gap-1.5">
          {#each filteredBookmarks as item, index (item.url + index)}
            <button
              id="bookmark-item-{index}"
              class="group flex w-full items-center gap-3.5 rounded-xl border border-transparent px-3 py-2.5 text-left text-sm transition-all duration-150 {selectedIndex ===
              index
                ? 'bg-neutral-200/90 shadow-sm dark:bg-neutral-700/60'
                : 'hover:bg-neutral-150/60 dark:hover:bg-neutral-800/40'}"
              onclick={() => (selectedIndex = index)}
              ondblclick={() => openBookmark(item)}
            >
              <!-- 书签图标：优先显示网站的高清 Favicon，失败时自动退化为所属浏览器图标 -->
              <div
                class="relative flex h-9 w-9 flex-shrink-0 items-center justify-center text-neutral-600 transition-all group-hover:scale-105 dark:text-neutral-300"
              >
                {#if getFaviconUrl(item.url)}
                  <img
                    src={getFaviconUrl(item.url)}
                    alt=""
                    class="h-5 w-5 rounded object-contain transition-all"
                    onload={(e) => {
                      (e.currentTarget as HTMLImageElement).style.opacity = "1";
                    }}
                    onerror={(e) => {
                      const img = e.currentTarget as HTMLImageElement;
                      img.style.display = "none";
                      const fallback =
                        img.nextElementSibling as HTMLElement | null;
                      if (fallback) fallback.style.display = "flex";
                    }}
                    style="opacity: 0; transition: opacity 0.2s;"
                  />
                {/if}
                <div
                  class="flex items-center justify-center"
                  style={getFaviconUrl(item.url) ? "display: none;" : ""}
                >
                  <PhosphorIcon
                    icon={getBrowserIcon(item.browser)}
                    size={20}
                    class={getBrowserColorClass(item.browser)}
                  />
                </div>
              </div>

              <!-- 书签详细信息 -->
              <div class="min-w-0 flex-1">
                <div class="flex items-center gap-2">
                  <span
                    class="truncate font-semibold text-neutral-900 dark:text-neutral-100"
                    title={item.title}
                  >
                    {item.title || "无标题书签"}
                  </span>
                  {#if item.folder}
                    <span
                      class="flex-shrink-0 rounded bg-neutral-100 px-1.5 py-0.5 text-[10px] font-medium text-neutral-500 dark:bg-neutral-800 dark:text-neutral-400"
                    >
                      {item.folder}
                    </span>
                  {/if}
                </div>
                <div
                  class="mt-1 truncate font-mono text-xs text-neutral-400 dark:text-neutral-500"
                  title={item.url}
                >
                  {item.url}
                </div>
              </div>

              <!-- 快捷指令提示：精细化键帽与平台自适应设计 -->
              {#if selectedIndex === index}
                <div
                  class="animate-in fade-in text-xxs flex flex-shrink-0 items-center gap-1.5 font-normal text-neutral-400/80 duration-200 dark:text-neutral-500"
                >
                  <kbd
                    class="rounded border border-neutral-200 bg-neutral-100 px-1.5 py-0.5 font-mono text-[9px] text-neutral-500 shadow-[0_1px_0_rgba(0,0,0,0.05)] dark:border-neutral-700/60 dark:bg-neutral-800 dark:text-neutral-400"
                    >Enter</kbd
                  >
                  <span class="scale-90 opacity-80">打开</span>

                  <span class="text-neutral-300 dark:text-neutral-700">|</span>

                  <div class="flex items-center gap-0.5">
                    {#if isMac}
                      <kbd
                        class="rounded border border-neutral-200 bg-neutral-100 px-1.5 py-0.5 font-mono text-[9px] text-neutral-500 shadow-[0_1px_0_rgba(0,0,0,0.05)] dark:border-neutral-700/60 dark:bg-neutral-800 dark:text-neutral-400"
                        >⌘</kbd
                      >
                      <kbd
                        class="rounded border border-neutral-200 bg-neutral-100 px-1.5 py-0.5 font-mono text-[9px] text-neutral-500 shadow-[0_1px_0_rgba(0,0,0,0.05)] dark:border-neutral-700/60 dark:bg-neutral-800 dark:text-neutral-400"
                        >C</kbd
                      >
                    {:else}
                      <kbd
                        class="rounded border border-neutral-200 bg-neutral-100 px-1.5 py-0.5 font-mono text-[9px] text-neutral-500 shadow-[0_1px_0_rgba(0,0,0,0.05)] dark:border-neutral-700/60 dark:bg-neutral-800 dark:text-neutral-400"
                        >Ctrl</kbd
                      >
                      <kbd
                        class="rounded border border-neutral-200 bg-neutral-100 px-1.5 py-0.5 font-mono text-[9px] text-neutral-500 shadow-[0_1px_0_rgba(0,0,0,0.05)] dark:border-neutral-700/60 dark:bg-neutral-800 dark:text-neutral-400"
                        >C</kbd
                      >
                    {/if}
                  </div>
                  <span class="scale-90 opacity-80">复制</span>
                </div>
              {/if}
            </button>
          {/each}
        </div>
      </AppScrollArea>
    {/if}
  </div>
</div>
