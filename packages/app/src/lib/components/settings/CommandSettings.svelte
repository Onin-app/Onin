<script lang="ts">
  /**
   * CommandSettings Component
   *
   * 指令设置页面 - 使用提取的子组件
   * 状态和逻辑保留在主组件中，确保 Svelte 5 响应式正常工作
   */
  import { Tabs, ScrollArea } from "bits-ui";
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { toast } from "svelte-sonner";
  import type { Command } from "$lib/type";

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
    Application: "程序启动",
    FileCommand: "文件启动",
    Plugin: "已安装插件",
    Custom: "自定义",
  };

  // ===== Lifecycle =====
  onMount(async () => {
    try {
      commands = await invoke<Command[]>("get_commands");
      console.log("Fetched commands:", commands);
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
    invoke("execute_command", { name: commandName })
      .then(() => console.log("Command executed:", commandName))
      .catch((error) => console.error("Failed to execute command:", error));
  }

  function toggleKeywordDisabled(commandName: string, keywordName: string) {
    const command = commands.find((cmd) => cmd.name === commandName);
    if (command) {
      const keyword = command.keywords.find((kw) => kw.name === keywordName);
      if (keyword) {
        keyword.disabled = !keyword.disabled;
        commands = [...commands];
        updateCommand(command);
      }
    }
  }

  function addKeyword(commandName: string, newKeyword: string) {
    if (!newKeyword || !newKeyword.trim()) return;
    const command = commands.find((cmd) => cmd.name === commandName);
    if (command && !command.keywords.some((kw) => kw.name === newKeyword)) {
      command.keywords.push({
        name: newKeyword,
        disabled: false,
        is_default: false,
      });
      commands = [...commands];
      updateCommand(command);
      toast.success(`已添加别名「${newKeyword}」`);
    }
  }

  function removeKeyword(commandName: string, keywordName: string) {
    const command = commands.find((cmd) => cmd.name === commandName);
    if (command) {
      const keywordToRemove = command.keywords.find(
        (kw) => kw.name === keywordName,
      );
      if (keywordToRemove && !keywordToRemove.is_default) {
        command.keywords = command.keywords.filter(
          (kw) => kw.name !== keywordName,
        );
        commands = [...commands];
        updateCommand(command);
        toast.success(`已删除别名「${keywordName}」`);
      }
    }
  }

  function selectCategory(categoryId: string) {
    selectedCategoryId = categoryId;
    selectedPlugin = null;
  }

  function selectPlugin(pluginName: string) {
    selectedPlugin = pluginName;
    selectedCategoryId = null;
  }

  // ===== Derived =====
  let commandCategories = $derived(
    Array.from(new Set(commands.map((cmd) => cmd.source))).map((source) => ({
      id: source,
      name: sourceNameMap[source] || source,
    })),
  );

  let activeCategory = $derived.by(() => {
    if (selectedCategoryId) {
      return (
        commandCategories.find((cat) => cat.id === selectedCategoryId) || null
      );
    }
    return commandCategories[0] || null;
  });

  let filteredCommands = $derived(
    commands.filter((cmd) => {
      if (cmd.source !== activeCategory?.id) return false;
      if (activeCategory?.id === "Plugin") {
        return (
          cmd.name.startsWith("plugin_") && !cmd.name.includes("plugin_cmd_")
        );
      }
      return true;
    }),
  );

  // Type guards
  function isPluginCommandAction(
    action: any,
  ): action is { PluginCommand: { plugin_id: string; command_code: string } } {
    return action && typeof action === "object" && "PluginCommand" in action;
  }

  function isPluginAction(action: any): action is { Plugin: string } {
    return action && typeof action === "object" && "Plugin" in action;
  }

  let pluginIdToNameMap = $derived.by(() => {
    const map = new Map<string, string>();
    commands
      .filter(
        (cmd) =>
          cmd.source === "Plugin" &&
          cmd.name.startsWith("plugin_") &&
          !cmd.name.startsWith("plugin_cmd_"),
      )
      .forEach((cmd) => {
        if (isPluginAction(cmd.action)) {
          map.set(cmd.action.Plugin, cmd.title);
        }
      });
    return map;
  });

  let pluginNames = $derived(
    Array.from(
      new Set(
        commands
          .filter(
            (cmd) =>
              cmd.source === "Plugin" && cmd.name.startsWith("plugin_cmd_"),
          )
          .map((cmd) => {
            if (isPluginCommandAction(cmd.action)) {
              const pluginId = cmd.action.PluginCommand.plugin_id;
              return pluginIdToNameMap.get(pluginId) || pluginId;
            }
            return null;
          })
          .filter((name): name is string => name !== null),
      ),
    ),
  );

  let selectedPluginCommands = $derived(
    selectedPlugin
      ? commands.filter((cmd) => {
          if (cmd.source !== "Plugin" || !cmd.name.startsWith("plugin_cmd_"))
            return false;
          if (isPluginCommandAction(cmd.action)) {
            const pluginId = cmd.action.PluginCommand.plugin_id;
            return pluginIdToNameMap.get(pluginId) === selectedPlugin;
          }
          return false;
        })
      : [],
  );

  // 判断命令是否为匹配指令（有 matches 配置）
  function isMatchCommand(cmd: Command): boolean {
    return cmd.matches != null && cmd.matches.length > 0;
  }

  // 功能指令列表（没有 matches 配置的插件指令）
  let functionCommands = $derived(
    selectedPluginCommands.filter((cmd) => !isMatchCommand(cmd)),
  );

  // 匹配指令列表（有 matches 配置的插件指令）
  let matchCommands = $derived(
    selectedPluginCommands.filter((cmd) => isMatchCommand(cmd)),
  );
</script>

<main class="flex h-full w-full gap-6">
  <!-- Sidebar -->
  <CommandSidebar
    categories={commandCategories}
    {activeCategory}
    {pluginNames}
    {selectedPlugin}
    onSelectCategory={selectCategory}
    onSelectPlugin={selectPlugin}
  />

  <!-- Content Area -->
  <div class="flex h-full min-w-0 flex-1 flex-col">
    <Tabs.Root value="function" class="flex h-full flex-col gap-4">
      <Tabs.List
        class="flex w-full border-b border-neutral-200 dark:border-neutral-800"
      >
        <Tabs.Trigger
          value="function"
          class="relative px-4 py-2 text-sm font-medium text-neutral-500 transition-colors after:absolute after:bottom-0 after:left-0 after:h-0.5 after:w-full 
          after:scale-x-0 after:bg-neutral-900 after:transition-transform after:duration-200 hover:text-neutral-700 data-[state=active]:text-neutral-900 data-[state=active]:after:scale-x-100 dark:text-neutral-400 dark:after:bg-white dark:hover:text-neutral-200 dark:data-[state=active]:text-white"
        >
          功能指令
        </Tabs.Trigger>
        <Tabs.Trigger
          value="match"
          class="relative px-4 py-2 text-sm font-medium text-neutral-500 transition-colors after:absolute after:bottom-0 after:left-0 after:h-0.5 after:w-full
          after:scale-x-0 after:bg-neutral-900 after:transition-transform after:duration-200 hover:text-neutral-700 data-[state=active]:text-neutral-900 data-[state=active]:after:scale-x-100 dark:text-neutral-400 dark:after:bg-white dark:hover:text-neutral-200 dark:data-[state=active]:text-white"
        >
          匹配指令
        </Tabs.Trigger>
      </Tabs.List>

      <Tabs.Content value="function" class="-mr-2 flex-1 overflow-hidden">
        <ScrollArea.Root class="h-full w-full" type="hover">
          <ScrollArea.Viewport class="h-full w-full pr-2">
            <div class="flex flex-col gap-3 pb-8">
              {#if selectedPlugin}
                <!-- 插件功能指令列表 -->
                {#each functionCommands as command}
                  <CommandCard
                    {command}
                    onExecute={executeCommand}
                    onToggleKeyword={toggleKeywordDisabled}
                    onAddKeyword={addKeyword}
                    onRemoveKeyword={removeKeyword}
                  />
                {/each}
                {#if functionCommands.length === 0}
                  <div
                    class="flex h-full flex-col items-center justify-center text-neutral-400"
                  >
                    <p>暂无功能指令</p>
                  </div>
                {/if}
              {:else}
                <!-- 内置指令列表 -->
                {#each filteredCommands as command}
                  <CommandCard
                    {command}
                    onExecute={executeCommand}
                    onToggleKeyword={toggleKeywordDisabled}
                    onAddKeyword={addKeyword}
                    onRemoveKeyword={removeKeyword}
                  />
                {/each}
              {/if}
            </div>
          </ScrollArea.Viewport>
          <ScrollArea.Scrollbar
            orientation="vertical"
            class="bg-muted hover:bg-dark-10 data-[state=visible]:animate-in data-[state=hidden]:animate-out data-[state=hidden]:fade-out-0 data-[state=visible]:fade-in-0 flex w-1.5 touch-none rounded-full border-l border-l-transparent p-px transition-all duration-200 select-none hover:w-3"
          >
            <ScrollArea.Thumb class="bg-muted-foreground flex-1 rounded-full" />
          </ScrollArea.Scrollbar>
          <ScrollArea.Corner />
        </ScrollArea.Root>
      </Tabs.Content>

      <Tabs.Content value="match" class="-mr-2 flex-1 overflow-hidden">
        <ScrollArea.Root class="h-full w-full" type="hover">
          <ScrollArea.Viewport class="h-full w-full pr-2">
            <div class="flex flex-col gap-3 pb-8">
              {#if selectedPlugin}
                <!-- 插件匹配指令列表 -->
                {#each matchCommands as command}
                  <CommandCard
                    {command}
                    onExecute={executeCommand}
                    onToggleKeyword={toggleKeywordDisabled}
                    onAddKeyword={addKeyword}
                    onRemoveKeyword={removeKeyword}
                  />
                {/each}
                {#if matchCommands.length === 0}
                  <div
                    class="flex h-full flex-col items-center justify-center text-neutral-400"
                  >
                    <p>暂无匹配指令</p>
                  </div>
                {/if}
              {:else}
                <div
                  class="flex h-full flex-col items-center justify-center text-neutral-400"
                >
                  <p>请选择一个插件</p>
                </div>
              {/if}
            </div>
          </ScrollArea.Viewport>
          <ScrollArea.Scrollbar
            orientation="vertical"
            class="bg-muted hover:bg-dark-10 data-[state=visible]:animate-in data-[state=hidden]:animate-out data-[state=hidden]:fade-out-0 data-[state=visible]:fade-in-0 flex w-1.5 touch-none rounded-full border-l border-l-transparent p-px transition-all duration-200 select-none hover:w-3"
          >
            <ScrollArea.Thumb class="bg-muted-foreground flex-1 rounded-full" />
          </ScrollArea.Scrollbar>
          <ScrollArea.Corner />
        </ScrollArea.Root>
      </Tabs.Content>
    </Tabs.Root>
  </div>
</main>
