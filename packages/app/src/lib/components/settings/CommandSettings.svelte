<script lang="ts">
  import { Button, DropdownMenu, Tabs, ScrollArea } from "bits-ui";
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import {
    Command as CommandIcon,
    RocketLaunch,
    File,
    Plugs,
    User,
    PuzzlePiece,
  } from "phosphor-svelte";
  import type { Command } from "$lib/type";

  const iconMap: Record<string, any> = {
    Command: CommandIcon,
    Application: RocketLaunch,
    FileCommand: File,
    Plugin: Plugs,
    Custom: User,
  };

  let commands = $state<Command[]>([]);
  let selectedPlugin = $state<string | null>(null);

  onMount(async () => {
    try {
      commands = await invoke<Command[]>("get_commands");
      console.log("Fetched commands:", commands);
    } catch (error) {
      console.error("Failed to fetch commands:", error);
    }
  });

  async function updateCommand(command: Command) {
    try {
      const commandToUpdate = JSON.parse(JSON.stringify(command));
      await invoke("update_command", { commandToUpdate });
    } catch (error) {
      console.error("Failed to update command:", error);
    }
  }

  async function executeCommand(commandName: string) {
    try {
      await invoke("execute_command", { name: commandName });
      console.log("Command executed:", commandName);
    } catch (error) {
      console.error("Failed to execute command:", error);
    }
  }

  async function executePluginCommand(commandName: string) {
    try {
      await invoke("execute_command", { name: commandName });
      console.log("Plugin command executed:", commandName);
    } catch (error) {
      console.error("Failed to execute plugin command:", error);
    }
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
      }
    }
  }

  const sourceNameMap = {
    Command: "基础常用",
    Application: "程序启动",
    FileCommand: "文件启动",
    Plugin: "已安装插件",
    Custom: "自定义",
  };

  let commandCategories = $derived(
    Array.from(new Set(commands.map((cmd) => cmd.source))).map((source) => ({
      id: source,
      name: sourceNameMap[source] || source,
    })),
  );

  let selectedCategoryId = $state<string | null>(null);

  // 使用 derived 来自动从 selectedCategoryId 获取完整的 category 对象
  let activeCategory = $derived.by(() => {
    if (selectedCategoryId) {
      return (
        commandCategories.find((cat) => cat.id === selectedCategoryId) || null
      );
    }
    // 如果没有选中的，默认返回第一个
    return commandCategories[0] || null;
  });

  // 过滤出插件启动指令（打开插件本身）和插件功能指令
  let filteredCommands = $derived(
    commands.filter((cmd) => {
      if (cmd.source !== activeCategory?.id) return false;
      // 如果是插件分类，只显示插件启动指令（不显示功能指令）
      if (activeCategory?.id === "Plugin") {
        return (
          cmd.name.startsWith("plugin_") && !cmd.name.includes("plugin_cmd_")
        );
      }
      return true;
    }),
  );

  // Type guard for PluginCommand action
  function isPluginCommandAction(
    action: any,
  ): action is { PluginCommand: { plugin_id: string; command_code: string } } {
    return action && typeof action === "object" && "PluginCommand" in action;
  }

  function isPluginAction(action: any): action is { Plugin: string } {
    return action && typeof action === "object" && "Plugin" in action;
  }

  // 构建插件ID到名称的映射（性能优化：避免重复查找）
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

  // 获取所有插件的名称列表（用于左侧菜单）
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

  // 获取选中插件的功能指令
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
</script>

<main class="flex h-full w-full gap-6">
  <!-- Internal Sidebar -->
  <div
    class="flex w-36 shrink-0 flex-col gap-4 border-r border-neutral-100 pr-4 dark:border-neutral-800"
  >
    <div class="flex flex-col gap-1">
      <h3
        class="px-2 py-1 text-[10px] font-semibold tracking-wider text-neutral-400 uppercase dark:text-neutral-500"
      >
        内置指令
      </h3>
      <div class="flex flex-col gap-0.5">
        {#each commandCategories as category}
          <Button.Root
            class="flex w-full items-center justify-start gap-2 rounded-md px-2 py-1 text-sm font-medium transition-colors {activeCategory?.id ===
            category.id
              ? 'bg-neutral-100 text-neutral-900 dark:bg-neutral-800 dark:text-white'
              : 'text-neutral-500 hover:bg-neutral-50 hover:text-neutral-900 dark:text-neutral-400 dark:hover:bg-neutral-800/50 dark:hover:text-neutral-200'}"
            onclick={() => {
              selectedCategoryId = category.id;
              selectedPlugin = null;
            }}
          >
            <svelte:component
              this={iconMap[category.id] || CommandIcon}
              size={15}
            />
            {category.name}
          </Button.Root>
        {/each}
      </div>
    </div>

    {#if pluginNames.length > 0}
      <div class="flex flex-col gap-1">
        <h3
          class="px-2 py-1 text-[10px] font-semibold tracking-wider text-neutral-400 uppercase dark:text-neutral-500"
        >
          插件指令
        </h3>
        <div class="flex flex-col gap-0.5">
          {#each pluginNames as pluginName}
            <Button.Root
              class="flex w-full items-center justify-start gap-2 truncate rounded-md px-2 py-1 text-left text-sm font-medium transition-colors {selectedPlugin ===
              pluginName
                ? 'bg-neutral-100 text-neutral-900 dark:bg-neutral-800 dark:text-white'
                : 'text-neutral-500 hover:bg-neutral-50 hover:text-neutral-900 dark:text-neutral-400 dark:hover:bg-neutral-800/50 dark:hover:text-neutral-200'}"
              onclick={() => {
                selectedPlugin = pluginName;
                selectedCategoryId = null;
              }}
            >
              <PuzzlePiece size={15} class="shrink-0" />
              <span class="truncate">{pluginName}</span>
            </Button.Root>
          {/each}
        </div>
      </div>
    {/if}
  </div>

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
                <!-- 插件指令列表 -->
                {#each selectedPluginCommands as command}
                  <div
                    class="group/card flex flex-col gap-2 rounded-xl border border-neutral-200 bg-white p-3 transition-all hover:border-neutral-300 dark:border-neutral-800 dark:bg-neutral-900 dark:hover:border-neutral-700"
                  >
                    <div class="flex items-center justify-between">
                      <h4
                        class="text-sm font-semibold text-neutral-900 dark:text-neutral-100"
                      >
                        {command.title}
                      </h4>
                    </div>
                    <div class="flex flex-wrap gap-1.5">
                      {#each command.keywords as keyword}
                        <div
                          class="group/chip relative inline-flex items-center rounded-md border border-neutral-200 bg-neutral-50 px-2 py-0.5 text-sm font-medium text-neutral-600 transition-colors dark:border-neutral-700 dark:bg-neutral-800 dark:text-neutral-300
                      {keyword.disabled
                            ? 'line-through opacity-50'
                            : 'hover:bg-neutral-100 dark:hover:bg-neutral-700/80'}"
                        >
                          <DropdownMenu.Root>
                            <DropdownMenu.Trigger
                              class="outline-none select-none"
                            >
                              {keyword.name}
                            </DropdownMenu.Trigger>
                            <DropdownMenu.Portal>
                              <DropdownMenu.Content
                                class="animate-in fade-in-0 zoom-in-95 z-50 min-w-[140px] overflow-hidden rounded-lg border border-neutral-200 bg-white p-1 shadow-lg dark:border-neutral-800 dark:bg-neutral-900"
                                sideOffset={4}
                                align="start"
                              >
                                <DropdownMenu.Item
                                  class="relative flex cursor-pointer items-center rounded-sm px-2 py-1.5 text-xs text-neutral-700 transition-colors outline-none select-none hover:bg-neutral-100 data-[disabled]:pointer-events-none data-[disabled]:opacity-50 dark:text-neutral-200 dark:hover:bg-neutral-800"
                                >
                                  <Button.Root
                                    class="w-full text-left"
                                    onclick={() =>
                                      executePluginCommand(command.name)}
                                  >
                                    执行指令
                                  </Button.Root>
                                </DropdownMenu.Item>
                                <DropdownMenu.Item
                                  class="relative flex cursor-pointer items-center rounded-sm px-2 py-1.5 text-xs text-neutral-700 transition-colors outline-none select-none hover:bg-neutral-100 data-[disabled]:pointer-events-none data-[disabled]:opacity-50 dark:text-neutral-200 dark:hover:bg-neutral-800"
                                >
                                  <Button.Root
                                    class="w-full text-left"
                                    onclick={() =>
                                      toggleKeywordDisabled(
                                        command.name,
                                        keyword.name,
                                      )}
                                  >
                                    {keyword.disabled ? "启用指令" : "禁用指令"}
                                  </Button.Root>
                                </DropdownMenu.Item>
                              </DropdownMenu.Content>
                            </DropdownMenu.Portal>
                          </DropdownMenu.Root>

                          {#if !keyword.is_default}
                            <button
                              class="-mr-0.5 ml-1 rounded-full p-0.5 text-neutral-400 opacity-0 transition-all group-hover/chip:opacity-100 hover:bg-neutral-200 hover:text-red-500 dark:text-neutral-500 dark:hover:bg-neutral-700"
                              onclick={(e) => {
                                e.stopPropagation();
                                removeKeyword(command.name, keyword.name);
                              }}
                            >
                              <svg
                                xmlns="http://www.w3.org/2000/svg"
                                width="10"
                                height="10"
                                viewBox="0 0 24 24"
                                fill="none"
                                stroke="currentColor"
                                stroke-width="2"
                                stroke-linecap="round"
                                stroke-linejoin="round"
                                ><path d="M18 6 6 18" /><path
                                  d="m6 6 12 12"
                                /></svg
                              >
                            </button>
                          {/if}
                        </div>
                      {/each}

                      <div class="relative flex items-center">
                        <input
                          type="text"
                          placeholder="+ 添加"
                          class="h-[24px] w-14 rounded-md border border-dashed border-neutral-300 bg-transparent px-2 text-[10px] text-neutral-500 transition-all placeholder:text-neutral-400 focus:w-20 focus:border-solid focus:border-neutral-400 focus:bg-white focus:text-neutral-900 focus:outline-none dark:border-neutral-700 dark:text-neutral-400 dark:focus:border-neutral-600 dark:focus:bg-neutral-900 dark:focus:text-neutral-100"
                          onkeydown={(e) => {
                            if (e.key === "Enter") {
                              addKeyword(command.name, e.currentTarget.value);
                              e.currentTarget.value = "";
                            }
                          }}
                        />
                      </div>
                    </div>
                  </div>
                {/each}
              {:else}
                <!-- 内置指令列表 -->
                {#each filteredCommands as command}
                  <div
                    class="group/card flex flex-col gap-2 rounded-xl border border-neutral-200 bg-white p-3 transition-all hover:border-neutral-300 dark:border-neutral-800 dark:bg-neutral-900 dark:hover:border-neutral-700"
                  >
                    <div class="flex items-center justify-between">
                      <h4
                        class="text-sm font-semibold text-neutral-900 dark:text-neutral-100"
                      >
                        {command.title}
                      </h4>
                    </div>
                    <div class="flex flex-wrap gap-1.5">
                      {#each command.keywords as keyword}
                        <div
                          class="group/chip relative inline-flex items-center rounded-md border border-neutral-200 bg-neutral-50 px-2 py-0.5 text-sm font-medium text-neutral-600 transition-colors dark:border-neutral-700 dark:bg-neutral-800 dark:text-neutral-300
                      {keyword.disabled
                            ? 'line-through opacity-50'
                            : 'hover:bg-neutral-100 dark:hover:bg-neutral-700/80'}"
                        >
                          <DropdownMenu.Root>
                            <DropdownMenu.Trigger
                              class="cursor-default outline-none select-none"
                            >
                              {keyword.name}
                            </DropdownMenu.Trigger>
                            <DropdownMenu.Portal>
                              <DropdownMenu.Content
                                class="animate-in fade-in-0 zoom-in-95 z-50 min-w-[140px] overflow-hidden rounded-lg border border-neutral-200 bg-white p-1 shadow-lg dark:border-neutral-800 dark:bg-neutral-900"
                                sideOffset={4}
                                align="start"
                              >
                                <DropdownMenu.Item
                                  class="relative flex cursor-pointer items-center rounded-sm px-2 py-1.5 text-xs text-neutral-700 transition-colors outline-none select-none hover:bg-neutral-100 data-[disabled]:pointer-events-none data-[disabled]:opacity-50 dark:text-neutral-200 dark:hover:bg-neutral-800"
                                >
                                  <Button.Root
                                    class="w-full text-left"
                                    onclick={() => executeCommand(command.name)}
                                  >
                                    执行指令
                                  </Button.Root>
                                </DropdownMenu.Item>
                                <DropdownMenu.Item
                                  class="relative flex cursor-pointer items-center rounded-sm px-2 py-1.5 text-xs text-neutral-700 transition-colors outline-none select-none hover:bg-neutral-100 data-[disabled]:pointer-events-none data-[disabled]:opacity-50 dark:text-neutral-200 dark:hover:bg-neutral-800"
                                >
                                  <Button.Root
                                    class="w-full text-left"
                                    onclick={() =>
                                      toggleKeywordDisabled(
                                        command.name,
                                        keyword.name,
                                      )}
                                  >
                                    {keyword.disabled ? "启用指令" : "禁用指令"}
                                  </Button.Root>
                                </DropdownMenu.Item>
                              </DropdownMenu.Content>
                            </DropdownMenu.Portal>
                          </DropdownMenu.Root>

                          {#if !keyword.is_default}
                            <button
                              class="-mr-0.5 ml-1 rounded-full p-0.5 text-neutral-400 opacity-0 transition-all group-hover/chip:opacity-100 hover:bg-neutral-200 hover:text-red-500 dark:text-neutral-500 dark:hover:bg-neutral-700"
                              onclick={(e) => {
                                e.stopPropagation();
                                removeKeyword(command.name, keyword.name);
                              }}
                            >
                              <svg
                                xmlns="http://www.w3.org/2000/svg"
                                width="10"
                                height="10"
                                viewBox="0 0 24 24"
                                fill="none"
                                stroke="currentColor"
                                stroke-width="2"
                                stroke-linecap="round"
                                stroke-linejoin="round"
                                ><path d="M18 6 6 18" /><path
                                  d="m6 6 12 12"
                                /></svg
                              >
                            </button>
                          {/if}
                        </div>
                      {/each}

                      <div class="relative flex items-center">
                        <input
                          type="text"
                          placeholder="+ 添加"
                          class="h-[28px] w-16 rounded-md border border-dashed border-neutral-300 bg-transparent px-2 text-sm text-neutral-500 transition-all placeholder:text-neutral-400 focus:w-24 focus:border-solid focus:border-neutral-400 focus:bg-white focus:text-neutral-900 focus:outline-none dark:border-neutral-700 dark:text-neutral-400 dark:focus:border-neutral-600 dark:focus:bg-neutral-900 dark:focus:text-neutral-100"
                          onkeydown={(e) => {
                            if (e.key === "Enter") {
                              addKeyword(command.name, e.currentTarget.value);
                              e.currentTarget.value = "";
                            }
                          }}
                        />
                      </div>
                    </div>
                  </div>
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
            <div
              class="flex h-full flex-col items-center justify-center text-neutral-400"
            >
              <p>暂无匹配指令</p>
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
