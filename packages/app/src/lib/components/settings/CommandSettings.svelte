<script lang="ts">
  /**
   * CommandSettings Component
   *
   * 指令设置页面 - 使用提取的子组件
   * 状态和逻辑保留在主组件中，确保 Svelte 5 响应式正常工作
   */
  import {
    Tabs,
    TabsList,
    TabsTrigger,
    TabsContent,
  } from "$lib/components/ui/tabs";
  import { ScrollArea } from "$lib/components/ui/scroll-area";
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { toast } from "svelte-sonner";
  import type { Command, Source } from "$lib/type";

  // 子组件
  import CommandSidebar from "./CommandSidebar.svelte";
  import CommandCard from "./CommandCard.svelte";

  // ===== State =====
  let commands = $state<Command[]>([]);
  let selectedPlugin = $state<string | null>(null);
  let selectedCategoryId = $state<string | null>(null);

  // ===== Constants =====
  const sourceNameMap: Record<string, string> = {
    Command: "基础常用",
    Extension: "内置扩展",
    Application: "程序启动",
    FileCommand: "文件启动",
    Plugin: "已安装插件",
    Internal: "页面导航",
  };

  function getPluginName(cmd: Command): string | undefined {
    if (cmd.source === "Plugin") {
      if ("PluginEntry" in cmd.action) {
        return cmd.action.PluginEntry.plugin_id;
      }
      if ("PluginCommand" in cmd.action) {
        return cmd.action.PluginCommand.plugin_id;
      }
    }
    return undefined;
  }

  // ===== Lifecycle =====
  onMount(async () => {
    try {
      commands = await invoke<Command[]>("get_commands");
    } catch (error) {
      console.error("Failed to fetch commands:", error);
    }
  });

  // ===== Methods =====
  async function updateCommand(command: Command) {
    try {
      const commandToUpdate = JSON.parse(JSON.stringify(command));
      await invoke("update_command", { commandToUpdate });
    } catch (error) {
      console.error("Failed to update command:", error);
    }
  }

  function executeCommand(commandName: string) {
    invoke("execute_command", { name: commandName }).catch((error) =>
      console.error("Failed to execute command:", error),
    );
  }

  function handleAddKeyword(commandName: string, keyword: string) {
    const command = commands.find((cmd) => cmd.name === commandName);
    if (!command) return;

    if (!command.keywords.some((k) => k.name === keyword)) {
      command.keywords.push({
        name: keyword,
        disabled: false,
        is_default: false,
      });
      updateCommand(command);
    }
  }

  function handleRemoveKeyword(commandName: string, keyword: string) {
    const command = commands.find((cmd) => cmd.name === commandName);
    if (!command) return;

    command.keywords = command.keywords.filter((k) => k.name !== keyword);
    updateCommand(command);
  }

  function handleToggleKeyword(commandName: string, keywordName: string) {
    const command = commands.find((cmd) => cmd.name === commandName);
    if (!command) return;

    const keyword = command.keywords.find((k) => k.name === keywordName);
    if (keyword) {
      keyword.disabled = !keyword.disabled;
      updateCommand(command);
    }
  }

  function handleSelectCategory(source: string) {
    selectedCategoryId = source;
    selectedPlugin = null;
  }

  function handleSelectPlugin(pluginName: string) {
    selectedPlugin = pluginName;
    selectedCategoryId = null;
  }

  // ===== Computed/Derived =====
  const availableCategories = $derived.by(() => {
    const sources = new Set<Source>(
      commands
        .map((cmd) => cmd.source)
        .filter((source): source is Source => !!source),
    );
    const categoryOrder: Source[] = [
      "Command",
      "Extension",
      "Application",
      "FileCommand",
      "Internal",
    ];
    return categoryOrder
      .filter((source) => sources.has(source))
      .map((source) => ({
        id: source,
        name: sourceNameMap[source] || source,
      }));
  });

  const activeCategory = $derived.by(() => {
    if (selectedPlugin) return null;
    if (selectedCategoryId) {
      return (
        availableCategories.find((c) => c.id === selectedCategoryId) ||
        availableCategories[0] ||
        null
      );
    }
    return availableCategories[0] || null;
  });

  const pluginNames = $derived.by(() => {
    const names = new Set(
      commands
        .filter((cmd) => cmd.source === "Plugin" && getPluginName(cmd))
        .map((cmd) => getPluginName(cmd) as string),
    );
    return Array.from(names);
  });

  // 插件的功能指令（没有 matches 的指令）
  const functionCommands = $derived(
    selectedPlugin
      ? commands.filter(
          (cmd) =>
            getPluginName(cmd) === selectedPlugin &&
            (!cmd.matches || cmd.matches.length === 0),
        )
      : [],
  );

  // 插件的匹配指令（有 matches 的指令）
  const matchCommands = $derived(
    selectedPlugin
      ? commands.filter(
          (cmd) =>
            getPluginName(cmd) === selectedPlugin &&
            cmd.matches &&
            cmd.matches.length > 0,
        )
      : [],
  );

  // 内置分类的功能指令（没有 matches 的指令）
  const builtinFunctionCommands = $derived(
    activeCategory
      ? commands.filter(
          (cmd) =>
            cmd.source === activeCategory.id &&
            (!cmd.matches || cmd.matches.length === 0),
        )
      : [],
  );

  // 内置分类的匹配指令（有 matches 的指令）
  const builtinMatchCommands = $derived(
    activeCategory
      ? commands.filter(
          (cmd) =>
            cmd.source === activeCategory.id &&
            cmd.matches &&
            cmd.matches.length > 0,
        )
      : [],
  );
</script>

<div class="flex h-full w-full gap-4">
  <!-- 左侧：分类和插件列表 -->
  <CommandSidebar
    categories={availableCategories}
    {activeCategory}
    {pluginNames}
    {selectedPlugin}
    onSelectCategory={handleSelectCategory}
    onSelectPlugin={handleSelectPlugin}
  />

  <!-- 右侧：指令列表 -->
  <div class="flex flex-1 flex-col overflow-hidden">
    {#if selectedPlugin}
      <!-- 插件分类：显示功能指令和匹配指令 Tabs -->
      <Tabs value="function" class="flex h-full flex-col">
        <TabsList class="mb-3 w-fit">
          <TabsTrigger value="function">
            功能指令 ({functionCommands.length})
          </TabsTrigger>
          <TabsTrigger value="match">
            匹配指令 ({matchCommands.length})
          </TabsTrigger>
        </TabsList>

        <TabsContent value="function" class="mt-0 flex-1 overflow-hidden">
          <ScrollArea
            class="h-full w-full"
            viewportClass="h-full w-full pr-2 pb-8"
          >
            <div class="flex flex-col gap-2">
              {#each functionCommands as command (command.name)}
                <CommandCard
                  {command}
                  onAddKeyword={handleAddKeyword}
                  onRemoveKeyword={handleRemoveKeyword}
                  onToggleKeyword={handleToggleKeyword}
                  onExecute={executeCommand}
                />
              {/each}
              {#if functionCommands.length === 0}
                <div class="text-muted-foreground py-8 text-center text-sm">
                  暂无功能指令
                </div>
              {/if}
            </div>
          </ScrollArea>
        </TabsContent>

        <TabsContent value="match" class="mt-0 flex-1 overflow-hidden">
          <ScrollArea
            class="h-full w-full"
            viewportClass="h-full w-full pr-2 pb-8"
          >
            <div class="flex flex-col gap-2">
              {#each matchCommands as command (command.name)}
                <CommandCard
                  {command}
                  onAddKeyword={handleAddKeyword}
                  onRemoveKeyword={handleRemoveKeyword}
                  onToggleKeyword={handleToggleKeyword}
                  onExecute={executeCommand}
                />
              {/each}
              {#if matchCommands.length === 0}
                <div class="text-muted-foreground py-8 text-center text-sm">
                  暂无匹配指令
                </div>
              {/if}
            </div>
          </ScrollArea>
        </TabsContent>
      </Tabs>
    {:else if activeCategory}
      <!-- 内置分类：如果同时有功能指令和匹配指令，显示 Tabs；否则直接显示列表 -->
      {#if builtinFunctionCommands.length > 0 && builtinMatchCommands.length > 0}
        <Tabs value="function" class="flex h-full flex-col">
          <TabsList class="mb-3 w-fit">
            <TabsTrigger value="function">
              功能指令 ({builtinFunctionCommands.length})
            </TabsTrigger>
            <TabsTrigger value="match">
              匹配指令 ({builtinMatchCommands.length})
            </TabsTrigger>
          </TabsList>

          <TabsContent value="function" class="mt-0 flex-1 overflow-hidden">
            <ScrollArea
              class="h-full w-full"
              viewportClass="h-full w-full pr-2 pb-8"
            >
              <div class="flex flex-col gap-2">
                {#each builtinFunctionCommands as command (command.name)}
                  <CommandCard
                    {command}
                    onAddKeyword={handleAddKeyword}
                    onRemoveKeyword={handleRemoveKeyword}
                    onToggleKeyword={handleToggleKeyword}
                    onExecute={executeCommand}
                  />
                {/each}
              </div>
            </ScrollArea>
          </TabsContent>

          <TabsContent value="match" class="mt-0 flex-1 overflow-hidden">
            <ScrollArea
              class="h-full w-full"
              viewportClass="h-full w-full pr-2 pb-8"
            >
              <div class="flex flex-col gap-2">
                {#each builtinMatchCommands as command (command.name)}
                  <CommandCard
                    {command}
                    onAddKeyword={handleAddKeyword}
                    onRemoveKeyword={handleRemoveKeyword}
                    onToggleKeyword={handleToggleKeyword}
                    onExecute={executeCommand}
                  />
                {/each}
              </div>
            </ScrollArea>
          </TabsContent>
        </Tabs>
      {:else}
        <!-- 只有一种类型的指令，直接显示列表 -->
        <ScrollArea
          class="h-full w-full"
          viewportClass="h-full w-full pr-2 pb-8"
        >
          <div class="flex flex-col gap-2">
            {#each commands.filter((cmd) => cmd.source === activeCategory.id) as command (command.name)}
              <CommandCard
                {command}
                onAddKeyword={handleAddKeyword}
                onRemoveKeyword={handleRemoveKeyword}
                onToggleKeyword={handleToggleKeyword}
                onExecute={executeCommand}
              />
            {/each}
            {#if commands.filter((cmd) => cmd.source === activeCategory.id).length === 0}
              <div class="text-muted-foreground py-8 text-center text-sm">
                暂无指令
              </div>
            {/if}
          </div>
        </ScrollArea>
      {/if}
    {/if}
  </div>
</div>
