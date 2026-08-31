<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { get } from "svelte/store";
  import { invoke, convertFileSrc } from "@tauri-apps/api/core";
  import { Button } from "$lib/components/ui/button";
  import { Badge } from "$lib/components/ui/badge";
  import { Card } from "$lib/components/ui/card";
  import { ScrollArea } from "$lib/components/ui/scroll-area";
  import { toast } from "svelte-sonner";
  import {
    Database,
    FolderOpen,
    Copy,
    FileCode,
    Cpu,
    Plugs,
    PuzzlePiece,
    ArrowClockwise,
    CaretRight,
  } from "phosphor-svelte";
  import { Theme } from "$lib/type";
  import { getTheme, theme } from "$lib/utils/theme";

  interface AppDataFileInfo {
    id: string;
    name: string;
    category: string; // "main" | "plugin" | "extension"
    rel_path: string;
    absolute_path: string;
    size_bytes: number;
    is_json: boolean;
    is_image: boolean;
    is_text: boolean;
  }

  let dataDirPath = $state<string>("加载中...");
  let filesList = $state<AppDataFileInfo[]>([]);
  let loadingList = $state<boolean>(true);

  let selectedFile = $state<AppDataFileInfo | null>(null);
  let selectedFileContent = $state<string>("");
  let selectedFileDisplay = $state<string>(""); // 脱敏后的展示内容
  let imageUrl = $state<string>(""); // 图片文件的本地展示 URL
  let loadingContent = $state<boolean>(false);
  let fileTooLarge = $state<boolean>(false);

  let highlightedHtml = $state<string | null>(null);
  let isHighlighting = $state<boolean>(false);
  let highlightRequestId = 0;
  let fileSelectRequestId = 0;
  let currentResolvedTheme = $state<Theme.DARK | Theme.LIGHT>(
    getTheme(get(theme)),
  );
  let unsubscribeTheme: (() => void) | null = null;

  // 格式化文件大小
  const formatSize = (bytes: number) => {
    if (bytes === 0) return "0 B";
    const k = 1024;
    const sizes = ["B", "KB", "MB", "GB", "TB", "PB"];
    const i = Math.min(
      Math.floor(Math.log(bytes) / Math.log(k)),
      sizes.length - 1,
    );
    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + " " + sizes[i];
  };

  // 校验敏感字段 key，排除 monkey、keyboard 等误伤词
  const isSensitiveKey = (key: string): boolean => {
    const k = key.toLowerCase();
    const exclusions = [
      "monkey",
      "donkey",
      "turkey",
      "whiskey",
      "keyboard",
      "keyword",
    ];
    if (exclusions.some((exclude) => k.includes(exclude))) {
      return false;
    }
    return (
      k.includes("key") ||
      k.includes("password") ||
      k.includes("secret") ||
      k.includes("token")
    );
  };

  // 敏感字段和长文本截断处理
  const processDataForPreview = (val: any, keyName?: string): any => {
    if (val === null || val === undefined) {
      return val;
    }
    if (typeof val === "string") {
      // 1. 敏感字段脱敏
      if (keyName && isSensitiveKey(keyName)) {
        if (val.length <= 8) {
          return "••••••••";
        } else {
          return val.slice(0, 4) + "••••" + val.slice(-4);
        }
      }
      // 2. 长文本截断（如 base64 数据）
      if (val.length > 1000) {
        return (
          val.slice(0, 50) +
          ` ... [已截断，共 ${val.length} 字符，完整数据可复制或打开文件查看] ... ` +
          val.slice(-10)
        );
      }
      return val;
    }
    if (Array.isArray(val)) {
      return val.map((item) => processDataForPreview(item, undefined));
    }
    if (typeof val === "object") {
      const newObj: any = {};
      for (const [k, v] of Object.entries(val)) {
        newObj[k] = processDataForPreview(v, k);
      }
      return newObj;
    }
    return val;
  };

  const truncateText = (text: string, maxLength: number): string => {
    if (text.length <= maxLength) return text;
    return (
      text.slice(0, maxLength) +
      `\n\n... [文件过大，已截断展示前 ${formatSize(maxLength)} 内容，完整数据可复制或打开文件查看] ...`
    );
  };

  // 初始化加载数据路径和文件列表
  const loadDataInfo = async () => {
    loadingList = true;
    try {
      dataDirPath = await invoke<string>("get_app_data_dir_path");
      filesList = await invoke<AppDataFileInfo[]>("list_app_data_files");

      // 加载内置扩展列表以获取友好名称
      try {
        loadedExtensions = await invoke<any[]>("get_extensions");
      } catch (err) {
        console.error("加载扩展列表失败", err);
      }

      // 加载已安装插件列表以获取友好名称
      try {
        loadedPlugins = await invoke<any[]>("get_loaded_plugins");
      } catch (err) {
        console.error("加载插件列表失败", err);
      }
    } catch (e) {
      console.error(e);
      toast.error("加载数据目录信息失败：" + String(e));
    } finally {
      loadingList = false;
    }
  };

  // 打开系统数据目录
  const handleOpenDataDir = async () => {
    try {
      await invoke("open_app_data_dir");
    } catch (e) {
      toast.error("打开文件夹失败：" + String(e));
    }
  };

  // 选择并读取文件内容
  const handleSelectFile = async (file: AppDataFileInfo) => {
    fileSelectRequestId += 1;
    const requestId = fileSelectRequestId;

    selectedFile = file;

    // 自动展开选中的分组
    if (file.category === "extension") {
      const extId = getExtensionId(file);
      expandedExtensions[extId] = true;
    } else if (file.category === "plugin") {
      const pluginId = getPluginId(file);
      expandedPlugins[pluginId] = true;
    }

    fileTooLarge = false;
    selectedFileContent = "";
    selectedFileDisplay = "";
    imageUrl = "";
    highlightedHtml = null;

    // 限制最大读取大小为 10MB
    const MAX_PREVIEW_SIZE = 10 * 1024 * 1024;
    if (file.size_bytes > MAX_PREVIEW_SIZE && file.is_text) {
      fileTooLarge = true;
      return;
    }

    loadingContent = true;

    // 1. 如果是图片文件，直接用 convertFileSrc 显示
    if (file.is_image) {
      try {
        imageUrl = convertFileSrc(file.absolute_path);
      } catch (e) {
        if (requestId !== fileSelectRequestId) return;
        toast.error("转换图片路径失败：" + String(e));
      } finally {
        if (requestId === fileSelectRequestId) {
          loadingContent = false;
        }
      }
      return;
    }

    // 2. 如果不是文本文件，直接停止加载， 在 UI 上显示二进制不可预览提示
    if (!file.is_text) {
      if (requestId === fileSelectRequestId) {
        loadingContent = false;
      }
      return;
    }

    // 3. 读取文本或 JSON 内容
    try {
      const raw = await invoke<string>("read_app_data_file_content", {
        relPath: file.rel_path,
      });
      if (requestId !== fileSelectRequestId) return;

      selectedFileContent = raw;

      if (file.is_json) {
        try {
          const parsed = JSON.parse(raw);
          const processed = processDataForPreview(parsed);
          selectedFileDisplay = JSON.stringify(processed, null, 2);
        } catch {
          selectedFileDisplay = truncateText(raw, 50000);
        }
      } else {
        selectedFileDisplay = truncateText(raw, 50000);
      }
    } catch (e) {
      if (requestId !== fileSelectRequestId) return;
      toast.error("读取文件内容失败：" + String(e));
    } finally {
      if (requestId === fileSelectRequestId) {
        loadingContent = false;
      }
    }
  };

  // 语法高亮
  const highlightText = async (
    content: string,
    isJson: boolean,
    resolvedTheme: Theme.DARK | Theme.LIGHT,
    requestId: number,
  ) => {
    isHighlighting = true;

    try {
      const { codeToHtml } = await import("shiki");
      const html = await codeToHtml(content, {
        lang: isJson ? "json" : "text",
        theme: resolvedTheme === Theme.DARK ? "github-dark" : "github-light",
      });

      if (requestId !== highlightRequestId) return;
      highlightedHtml = html;
    } catch (error) {
      console.warn("Failed to highlight JSON preview:", error);
      if (requestId !== highlightRequestId) return;
      highlightedHtml = null;
    } finally {
      if (requestId === highlightRequestId) {
        isHighlighting = false;
      }
    }
  };

  $effect(() => {
    highlightRequestId += 1;
    const requestId = highlightRequestId;

    if (!selectedFileDisplay || !selectedFile) {
      highlightedHtml = null;
      isHighlighting = false;
      return;
    }

    void highlightText(
      selectedFileDisplay,
      selectedFile.is_json,
      currentResolvedTheme,
      requestId,
    );
  });

  // 复制未脱敏的原生配置内容
  const handleCopyContent = async () => {
    if (!selectedFileContent) return;
    try {
      await navigator.clipboard.writeText(selectedFileContent);
      toast.success("已复制完整配置数据（包含密钥），请妥善保管");
    } catch (e) {
      toast.error("复制失败：" + String(e));
    }
  };

  onMount(() => {
    loadDataInfo();
    // 预加载 shiki，优化首次打开性能
    void import("shiki").catch((err) => {
      console.warn("预加载 shiki 失败:", err);
    });

    unsubscribeTheme = theme.subscribe((value) => {
      const resolved = getTheme(value);
      if (currentResolvedTheme !== resolved) {
        currentResolvedTheme = resolved;
      }
    });
  });

  onDestroy(() => {
    unsubscribeTheme?.();
    fileSelectRequestId = -1;
    highlightRequestId = -1;
  });

  // 按分类计算文件
  const mainFiles = $derived(filesList.filter((f) => f.category === "main"));
  const pluginFiles = $derived(
    filesList.filter((f) => f.category === "plugin"),
  );
  const extensionFiles = $derived(
    filesList.filter((f) => f.category === "extension"),
  );

  interface ExtensionItemInfo {
    id: string;
    name: string;
    description?: string;
    icon?: string;
    enabled?: boolean;
  }

  interface LoadedPluginInfo {
    id: string;
    name: string;
    dir_name: string;
    enabled?: boolean;
  }

  let loadedExtensions = $state<ExtensionItemInfo[]>([]);
  let loadedPlugins = $state<LoadedPluginInfo[]>([]);
  let expandedExtensions = $state<Record<string, boolean>>({});
  let expandedPlugins = $state<Record<string, boolean>>({});

  const getExtensionId = (file: AppDataFileInfo) => {
    const parts = file.rel_path.split("/");
    if (parts.length > 1 && parts[0] === "extensions") {
      return parts[1];
    }
    return "unknown";
  };

  const getPluginId = (file: AppDataFileInfo) => {
    if (file.rel_path.startsWith("plugin_settings/")) {
      return file.rel_path
        .substring("plugin_settings/".length)
        .replace(/\.json$/i, "");
    }
    if (file.rel_path.startsWith("plugin_data/")) {
      const sub = file.rel_path.substring("plugin_data/".length);
      const parts = sub.split("/");
      if (parts.length > 0) {
        return parts[0];
      }
    }
    return "unknown";
  };

  const getExtensionName = (id: string) => {
    const ext = loadedExtensions.find((x) => x.id === id);
    return ext?.name || id;
  };

  const getPluginName = (id: string) => {
    if (id === "global") return "全局插件状态";
    const p = loadedPlugins.find((x) => x.id === id || x.dir_name === id);
    return p?.name || id;
  };

  const getFriendlyFileName = (file: AppDataFileInfo) => {
    if (file.rel_path.startsWith("plugin_settings/")) {
      return "插件设置 (.json)";
    }
    const parts = file.rel_path.split("/");
    return parts[parts.length - 1];
  };

  const toggleExtensionExpand = (id: string) => {
    expandedExtensions[id] = !expandedExtensions[id];
  };

  const togglePluginExpand = (id: string) => {
    expandedPlugins[id] = !expandedPlugins[id];
  };

  const groupedExtensions = $derived(
    (() => {
      const groupsMap = new Map<string, AppDataFileInfo[]>();
      for (const file of extensionFiles) {
        const extId = getExtensionId(file);
        if (!groupsMap.has(extId)) {
          groupsMap.set(extId, []);
        }
        groupsMap.get(extId)!.push(file);
      }
      return Array.from(groupsMap.entries()).map(([id, files]) => ({
        entityId: id,
        entityName: getExtensionName(id),
        files,
      }));
    })(),
  );

  const groupedPlugins = $derived(
    (() => {
      const groupsMap = new Map<string, AppDataFileInfo[]>();
      for (const file of pluginFiles) {
        const pluginId = getPluginId(file);
        if (!groupsMap.has(pluginId)) {
          groupsMap.set(pluginId, []);
        }
        groupsMap.get(pluginId)!.push(file);
      }
      return Array.from(groupsMap.entries()).map(([id, files]) => ({
        entityId: id,
        entityName: getPluginName(id),
        files,
      }));
    })(),
  );

  const getSelectedFileDisplayName = (file: AppDataFileInfo) => {
    if (file.category === "extension") {
      const extId = getExtensionId(file);
      const extName = getExtensionName(extId);
      return `${extName} - ${getFriendlyFileName(file)}`;
    }
    if (file.category === "plugin") {
      const pluginId = getPluginId(file);
      const pluginName = getPluginName(pluginId);
      return `${pluginName} - ${getFriendlyFileName(file)}`;
    }
    return file.name;
  };
</script>

<div class="flex h-full w-full flex-col gap-4 overflow-hidden pr-2">
  <!-- 顶部路径卡片 -->
  <Card
    class="border-border/60 bg-card flex items-center justify-between gap-4 rounded-2xl p-4 shadow-2xs"
  >
    <div class="flex min-w-0 flex-1 flex-col gap-1 overflow-hidden">
      <span
        class="text-muted-foreground text-xs font-semibold tracking-wider uppercase"
      >
        本地数据存储路径
      </span>
      <span
        class="text-foreground/90 truncate font-mono text-xs"
        title={dataDirPath}
      >
        {dataDirPath}
      </span>
    </div>
    <div class="flex shrink-0 gap-2">
      <Button
        variant="outline"
        size="sm"
        class="h-8 cursor-pointer gap-1.5 rounded-xl text-xs font-medium transition-[transform,background-color] duration-120 active:scale-95"
        onclick={loadDataInfo}
        disabled={loadingList}
      >
        <ArrowClockwise size={14} class={loadingList ? "animate-spin" : ""} />
        刷新
      </Button>
      <Button
        variant="default"
        size="sm"
        class="h-8 cursor-pointer gap-1.5 rounded-xl text-xs font-medium transition-[transform,background-color] duration-120 active:scale-95"
        onclick={handleOpenDataDir}
      >
        <FolderOpen size={14} />
        打开文件夹
      </Button>
    </div>
  </Card>

  <!-- 双栏展示区 -->
  <div class="flex flex-1 gap-4 overflow-hidden">
    <!-- 左侧列表 -->
    <div
      class="border-border/60 bg-card flex w-60 flex-col overflow-hidden rounded-2xl border shadow-2xs"
    >
      <ScrollArea
        class="h-full w-full"
        viewportClass="h-full w-full p-3 flex flex-col gap-4"
      >
        {#if loadingList}
          <div
            class="text-muted-foreground flex flex-1 items-center justify-center py-8 text-xs"
          >
            加载列表中...
          </div>
        {:else}
          <!-- 核心配置文件 -->
          {#if mainFiles.length > 0}
            <div class="flex flex-col gap-1">
              <div
                class="text-muted-foreground flex items-center gap-1.5 px-2 py-1 text-[11px] font-semibold tracking-wider uppercase"
              >
                <Cpu size={12} />
                核心配置
              </div>
              {#each mainFiles as file (file.id)}
                {@const isActive = selectedFile?.id === file.id}
                <button
                  class="flex w-full cursor-pointer flex-col gap-1 rounded-xl px-2.5 py-1.5 text-left transition-[transform,background-color,color] duration-120 active:scale-[0.98] {isActive
                    ? 'bg-muted text-foreground border-border/50 border font-medium shadow-2xs'
                    : 'text-muted-foreground hover:bg-muted/50 hover:text-foreground'}"
                  onclick={() => handleSelectFile(file)}
                >
                  <span class="truncate text-xs font-medium">{file.name}</span>
                  <div
                    class="text-muted-foreground/75 flex w-full items-center justify-between text-[9px]"
                  >
                    <span
                      class="max-w-[110px] truncate font-mono"
                      title={file.rel_path.split("/").pop()}
                      >{file.rel_path.split("/").pop()}</span
                    >
                    <span class="shrink-0 font-mono"
                      >{formatSize(file.size_bytes)}</span
                    >
                  </div>
                </button>
              {/each}
            </div>
          {/if}

          <!-- 扩展配置文件 -->
          {#if groupedExtensions.length > 0}
            <div class="flex flex-col gap-1.5">
              <div
                class="text-muted-foreground flex items-center gap-1.5 px-2 py-1 text-[11px] font-semibold tracking-wider uppercase"
              >
                <PuzzlePiece size={12} />
                扩展数据
              </div>
              {#each groupedExtensions as group (group.entityId)}
                {@const isExpanded = !!expandedExtensions[group.entityId]}
                {@const hasActiveFile = group.files.some(
                  (f) => selectedFile?.id === f.id,
                )}
                <div
                  class="border-border/40 bg-muted/20 flex flex-col gap-0.5 overflow-hidden rounded-xl border"
                >
                  <button
                    class="hover:bg-muted/50 flex w-full cursor-pointer items-center justify-between px-3 py-2 text-left text-xs font-semibold transition-[background-color,color] duration-120 {hasActiveFile
                      ? 'bg-muted/60 text-foreground'
                      : 'text-muted-foreground'}"
                    onclick={() => toggleExtensionExpand(group.entityId)}
                  >
                    <div
                      class="flex flex-col items-start gap-0.5 overflow-hidden"
                    >
                      <span class="truncate">{group.entityName}</span>
                      {#if group.entityId !== group.entityName}
                        <span
                          class="text-muted-foreground/70 truncate font-mono text-[9px] font-normal"
                        >
                          ID: {group.entityId}
                        </span>
                      {/if}
                    </div>
                    <span
                      class="text-muted-foreground transition-transform duration-140 ease-[cubic-bezier(0.23,1,0.32,1)] {isExpanded
                        ? 'rotate-90'
                        : ''}"
                    >
                      <CaretRight size={12} />
                    </span>
                  </button>
                  {#if isExpanded}
                    <div
                      class="border-border/30 flex flex-col gap-0.5 border-t py-1 pr-1 pl-2"
                    >
                      {#each group.files as file (file.id)}
                        {@const isActive = selectedFile?.id === file.id}
                        <button
                          class="flex w-full cursor-pointer flex-col gap-1 rounded-lg px-2.5 py-1.5 text-left transition-[transform,background-color,color] duration-120 active:scale-[0.98] {isActive
                            ? 'bg-muted text-foreground border-border/50 border font-medium shadow-2xs'
                            : 'text-muted-foreground hover:bg-muted/50 hover:text-foreground'}"
                          onclick={() => handleSelectFile(file)}
                        >
                          <span class="truncate text-[11px] font-medium"
                            >{getFriendlyFileName(file)}</span
                          >
                          <div
                            class="text-muted-foreground/75 flex w-full items-center justify-between text-[9px]"
                          >
                            <span
                              class="max-w-[100px] truncate font-mono"
                              title={file.rel_path}
                              >{file.rel_path.split("/").pop()}</span
                            >
                            <span class="shrink-0 font-mono"
                              >{formatSize(file.size_bytes)}</span
                            >
                          </div>
                        </button>
                      {/each}
                    </div>
                  {/if}
                </div>
              {/each}
            </div>
          {/if}

          <!-- 插件配置文件 -->
          {#if groupedPlugins.length > 0}
            <div class="flex flex-col gap-1.5">
              <div
                class="text-muted-foreground flex items-center gap-1.5 px-2 py-1 text-[11px] font-semibold tracking-wider uppercase"
              >
                <Plugs size={12} />
                插件数据
              </div>
              {#each groupedPlugins as group (group.entityId)}
                {@const isExpanded = !!expandedPlugins[group.entityId]}
                {@const hasActiveFile = group.files.some(
                  (f) => selectedFile?.id === f.id,
                )}
                <div
                  class="border-border/40 bg-muted/20 flex flex-col gap-0.5 overflow-hidden rounded-xl border"
                >
                  <button
                    class="hover:bg-muted/50 flex w-full cursor-pointer items-center justify-between px-3 py-2 text-left text-xs font-semibold transition-[background-color,color] duration-120 {hasActiveFile
                      ? 'bg-muted/60 text-foreground'
                      : 'text-muted-foreground'}"
                    onclick={() => togglePluginExpand(group.entityId)}
                  >
                    <div
                      class="flex flex-col items-start gap-0.5 overflow-hidden"
                    >
                      <span class="truncate">{group.entityName}</span>
                      {#if group.entityId !== group.entityName}
                        <span
                          class="text-muted-foreground/70 truncate font-mono text-[9px] font-normal"
                        >
                          ID: {group.entityId}
                        </span>
                      {/if}
                    </div>
                    <span
                      class="text-muted-foreground transition-transform duration-140 ease-[cubic-bezier(0.23,1,0.32,1)] {isExpanded
                        ? 'rotate-90'
                        : ''}"
                    >
                      <CaretRight size={12} />
                    </span>
                  </button>
                  {#if isExpanded}
                    <div
                      class="border-border/30 flex flex-col gap-0.5 border-t py-1 pr-1 pl-2"
                    >
                      {#each group.files as file (file.id)}
                        {@const isActive = selectedFile?.id === file.id}
                        <button
                          class="flex w-full cursor-pointer flex-col gap-1 rounded-lg px-2.5 py-1.5 text-left transition-[transform,background-color,color] duration-120 active:scale-[0.98] {isActive
                            ? 'bg-muted text-foreground border-border/50 border font-medium shadow-2xs'
                            : 'text-muted-foreground hover:bg-muted/50 hover:text-foreground'}"
                          onclick={() => handleSelectFile(file)}
                        >
                          <span class="truncate text-[11px] font-medium"
                            >{getFriendlyFileName(file)}</span
                          >
                          <div
                            class="text-muted-foreground/75 flex w-full items-center justify-between text-[9px]"
                          >
                            <span
                              class="max-w-[100px] truncate font-mono"
                              title={file.rel_path}
                              >{file.rel_path.split("/").pop()}</span
                            >
                            <span class="shrink-0 font-mono"
                              >{formatSize(file.size_bytes)}</span
                            >
                          </div>
                        </button>
                      {/each}
                    </div>
                  {/if}
                </div>
              {/each}
            </div>
          {/if}
        {/if}
      </ScrollArea>
    </div>

    <!-- 右侧内容区域 -->
    <div
      class="border-border/60 bg-card flex flex-1 flex-col overflow-hidden rounded-2xl border shadow-2xs"
    >
      {#if !selectedFile}
        <div
          class="text-muted-foreground/75 flex flex-1 flex-col items-center justify-center gap-3"
        >
          <Database size={40} weight="light" />
          <span class="text-xs">请在左侧选择要预览的文件</span>
        </div>
      {:else}
        <!-- 详情顶部面板 -->
        <div
          class="border-border/50 flex items-center justify-between border-b px-4 py-3"
        >
          <div class="flex flex-col gap-0.5 overflow-hidden">
            <span
              class="text-foreground truncate text-xs font-semibold tracking-tight"
            >
              {getSelectedFileDisplayName(selectedFile)}
            </span>
            <span
              class="text-muted-foreground/70 truncate font-mono text-[10px]"
            >
              相对路径: {selectedFile.rel_path}
            </span>
          </div>
          <div class="flex items-center gap-2">
            {#if selectedFile.is_json}
              <Button
                variant="outline"
                size="sm"
                class="h-7 cursor-pointer gap-1 rounded-lg px-2.5 text-[11px] font-medium transition-[transform,background-color] duration-120 active:scale-95"
                onclick={handleCopyContent}
                disabled={loadingContent || !selectedFileContent}
              >
                <Copy size={12} />
                复制
              </Button>
            {/if}
          </div>
        </div>

        <!-- 详情内容区 -->
        <ScrollArea
          class="min-h-0 w-full flex-1"
          viewportClass="h-full w-full p-4 bg-muted/10 flex flex-col"
        >
          {#if loadingContent}
            <div
              class="text-muted-foreground flex flex-1 items-center justify-center text-xs"
            >
              读取数据内容中...
            </div>
          {:else if imageUrl}
            <div class="flex flex-1 items-center justify-center p-2">
              <img
                src={imageUrl}
                alt={getSelectedFileDisplayName(selectedFile)}
                class="border-border/60 bg-background max-h-[380px] max-w-full rounded-xl border p-1.5 shadow-sm"
              />
            </div>
          {:else if fileTooLarge}
            <div
              class="text-muted-foreground flex flex-1 flex-col items-center justify-center gap-3 p-6 text-center"
            >
              <FileCode size={40} class="text-muted-foreground/60" />
              <span class="text-foreground text-xs font-semibold">
                文件过大，已禁用在应用内直接读取预览
              </span>
              <p
                class="text-muted-foreground/80 max-w-md text-[11px] leading-relaxed"
              >
                当前文件大小为 <strong class="text-foreground"
                  >{formatSize(selectedFile?.size_bytes || 0)}</strong
                >，已超过系统预览限制 (10
                MB)。为了避免解析大文件导致应用卡死，我们限制了其直接预览。
              </p>
              <div class="mt-2 flex gap-2">
                <Button
                  variant="outline"
                  size="sm"
                  class="h-8 cursor-pointer gap-1.5 rounded-xl text-xs font-medium transition-[transform,background-color] duration-120 active:scale-95"
                  onclick={handleOpenDataDir}
                >
                  <FolderOpen size={14} />
                  在系统资源管理器中打开目录
                </Button>
              </div>
            </div>
          {:else if !selectedFileDisplay}
            <div
              class="text-muted-foreground/75 flex flex-1 flex-col items-center justify-center gap-2"
            >
              <FileCode size={32} />
              <span class="text-xs">当前文件是二进制文件或无法直接预览</span>
            </div>
          {:else}
            <!-- 带有微动画和等宽字体的代码高亮 -->
            <div class="file-preview-code relative min-h-0 w-full">
              {#if isHighlighting}
                <div
                  class="bg-muted/80 text-muted-foreground border-border/40 absolute top-3 right-3 flex items-center gap-1.5 rounded-lg border px-2 py-1 text-[10px] font-medium shadow-2xs backdrop-blur-xs"
                >
                  <span
                    class="bg-primary h-1.5 w-1.5 animate-pulse rounded-full"
                  ></span>
                  正在高亮...
                </div>
              {/if}
              {#if highlightedHtml}
                {@html highlightedHtml}
              {:else}
                <pre
                  class="border-border/60 bg-background text-foreground overflow-x-auto rounded-xl border p-3.5 font-mono text-xs break-all whitespace-pre-wrap shadow-2xs select-text">{selectedFileDisplay}</pre>
              {/if}
            </div>
          {/if}
        </ScrollArea>
      {/if}
    </div>
  </div>
</div>

<style>
  :global(.file-preview-code pre.shiki) {
    min-height: 100%;
    margin: 0;
    padding: 1rem;
    overflow: visible;
    font-family:
      ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, "Liberation Mono",
      "Courier New", monospace;
    font-size: 0.75rem;
    line-height: 1.625;
    white-space: pre-wrap;
    word-break: break-word;
    background: transparent !important;
  }

  :global(.file-preview-code code) {
    font-family: inherit;
  }
</style>
