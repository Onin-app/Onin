<script lang="ts">
  import { Button, DropdownMenu, Tabs } from "bits-ui";
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import type { Command } from "$lib/type";

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

  let activeCategory = $state(commandCategories[0] || null);
  $effect(() => {
    if (!activeCategory && commandCategories.length > 0) {
      activeCategory = commandCategories[0];
    }
  });

  // 过滤出插件启动指令（打开插件本身）和插件功能指令
  let filteredCommands = $derived(
    commands.filter((cmd) => {
      if (cmd.source !== activeCategory?.id) return false;
      // 如果是插件分类，只显示插件启动指令（不显示功能指令）
      if (activeCategory?.id === "Plugin") {
        return cmd.name.startsWith("plugin_") && !cmd.name.includes("plugin_cmd_");
      }
      return true;
    }),
  );

  // Type guard for PluginCommand action
  function isPluginCommandAction(action: any): action is { PluginCommand: { plugin_id: string; command_code: string } } {
    return action && typeof action === "object" && "PluginCommand" in action;
  }

  function isPluginAction(action: any): action is { Plugin: string } {
    return action && typeof action === "object" && "Plugin" in action;
  }

  // 构建插件ID到名称的映射（性能优化：避免重复查找）
  let pluginIdToNameMap = $derived.by(() => {
    const map = new Map<string, string>();
    commands
      .filter((cmd) => cmd.source === "Plugin" && cmd.name.startsWith("plugin_") && !cmd.name.startsWith("plugin_cmd_"))
      .forEach((cmd) => {
        if (isPluginAction(cmd.action)) {
          map.set(cmd.action.Plugin, cmd.title);
        }
      });
    return map;
  });

  // 获取所有插件的名称列表（用于左侧菜单）
  let pluginNames = $derived(
    Array.from(new Set(
      commands
        .filter((cmd) => cmd.source === "Plugin" && cmd.name.startsWith("plugin_cmd_"))
        .map((cmd) => {
          if (isPluginCommandAction(cmd.action)) {
            const pluginId = cmd.action.PluginCommand.plugin_id;
            return pluginIdToNameMap.get(pluginId) || pluginId;
          }
          return null;
        })
        .filter((name): name is string => name !== null)
    ))
  );

  // 获取选中插件的功能指令
  let selectedPluginCommands = $derived(
    selectedPlugin
      ? commands.filter((cmd) => {
          if (cmd.source !== "Plugin" || !cmd.name.startsWith("plugin_cmd_")) return false;
          if (isPluginCommandAction(cmd.action)) {
            const pluginId = cmd.action.PluginCommand.plugin_id;
            return pluginIdToNameMap.get(pluginId) === selectedPlugin;
          }
          return false;
        })
      : []
  );
</script>

<main class="flex h-full">
  <div class="w-36 border-r border-neutral-200 dark:border-neutral-700">
    <h3 class="text-jgray-500 p-2 text-sm dark:text-gray-400">内置指令</h3>
    <ul class="flex w-full flex-col justify-center">
      {#each commandCategories as category}
        <li
          class={activeCategory?.id === category.id
            ? "bg-neutral-300 dark:bg-neutral-600"
            : "hover:bg-neutral-200 dark:hover:bg-neutral-700"}
        >
          <Button.Root
            class="h-full w-full cursor-pointer px-2 py-1 text-left"
            onclick={() => {
              activeCategory = category;
              selectedPlugin = null; // 清除插件选择
            }}
          >
            {category.name}
          </Button.Root>
        </li>
      {/each}
    </ul>
    <h3 class="text-jgray-500 mt-4 p-2 text-sm dark:text-gray-400">插件指令</h3>
    <ul class="flex w-full flex-col justify-center">
      {#each pluginNames as pluginName}
        <li
          class={selectedPlugin === pluginName
            ? "bg-neutral-300 dark:bg-neutral-600"
            : "hover:bg-neutral-200 dark:hover:bg-neutral-700"}
        >
          <Button.Root
            class="h-full w-full cursor-pointer px-2 py-1 text-left"
            onclick={() => {
              selectedPlugin = pluginName;
              activeCategory = null; // 清除内置指令选择
            }}
          >
            {pluginName}
          </Button.Root>
        </li>
      {/each}
    </ul>
  </div>
  <div class="flex-1 overflow-y-auto p-3">
    <Tabs.Root value="function" class="flex h-full flex-col">
      <Tabs.List
        class="rounded-9px bg-dark-10 shadow-mini-inset dark:bg-background grid w-full grid-cols-2 gap-1 p-1 text-sm leading-[0.01em] font-semibold dark:border dark:border-neutral-600/30"
      >
        <Tabs.Trigger
          value="function"
          class="data-[state=active]:shadow-mini dark:data-[state=active]:bg-muted h-8 rounded-[7px] bg-transparent py-2 data-[state=active]:bg-white"
        >
          功能指令
        </Tabs.Trigger>
        <Tabs.Trigger
          value="match"
          class="data-[state=active]:shadow-mini dark:data-[state=active]:bg-muted h-8 rounded-[7px] bg-transparent py-2 data-[state=active]:bg-white"
        >
          匹配指令
        </Tabs.Trigger>
      </Tabs.List>
      <Tabs.Content
        value="function"
        class="flex-1 overflow-y-auto pt-3 select-none"
      >
        {#if selectedPlugin}
          <!-- 显示插件指令 -->
          {#each selectedPluginCommands as command}
            <div class="group/box mb-4">
              <div class="mb-2">
                <h4 class="text-sm font-semibold">
                  {command.title}
                </h4>
              </div>
              <div class="flex flex-wrap gap-2">
                {#each command.keywords as keyword}
                  <div
                    class="group/button border-input text-foreground shadow-btn hover:bg-muted relative inline-flex cursor-pointer items-center justify-center rounded-full border px-2 py-1 text-sm font-medium select-none active:scale-[0.98] {keyword.disabled
                      ? 'bg-neutral-200 text-neutral-500 line-through dark:bg-neutral-700 dark:text-neutral-400'
                      : 'bg-white dark:bg-neutral-800'}"
                  >
                    <DropdownMenu.Root>
                      <DropdownMenu.Trigger>
                        {keyword.name}
                      </DropdownMenu.Trigger>
                      <DropdownMenu.Portal>
                        <DropdownMenu.Content
                          class="border-muted bg-background shadow-popover w-[229px] rounded-xl border px-1 py-1.5 outline-hidden focus-visible:outline-hidden"
                          sideOffset={8}
                          align="start"
                        >
                          <DropdownMenu.Item
                            class="rounded-button data-highlighted:bg-muted flex h-10 items-center py-3 pr-1.5 pl-3 text-sm font-medium ring-0! ring-transparent! select-none focus-visible:outline-none"
                          >
                            <Button.Root
                              class="w-full"
                              onclick={() => {
                                executePluginCommand(command.name);
                              }}
                            >
                              执行指令
                            </Button.Root>
                          </DropdownMenu.Item>
                          <DropdownMenu.Item
                            class="rounded-button data-highlighted:bg-muted flex h-10 items-center py-3 pr-1.5 pl-3 text-sm font-medium ring-0! ring-transparent! select-none focus-visible:outline-none"
                          >
                            <Button.Root
                              class="w-full"
                              onclick={() => {
                                toggleKeywordDisabled(command.name, keyword.name);
                              }}
                            >
                              {keyword.disabled ? "启用指令" : "禁用指令"}
                            </Button.Root>
                          </DropdownMenu.Item>
                        </DropdownMenu.Content>
                      </DropdownMenu.Portal>
                    </DropdownMenu.Root>
                    {#if !keyword.is_default}
                      <Button.Root
                        class="absolute -top-1 -right-1 hidden h-4 w-4 items-center justify-center rounded-full bg-red-500 text-white group-hover/button:flex"
                        onclick={() => {
                          removeKeyword(command.name, keyword.name);
                        }}
                      >
                        &times;
                      </Button.Root>
                    {/if}
                  </div>
                {/each}
                <div class="hidden group-hover/box:block">
                  <input
                    type="text"
                    placeholder="添加关键字"
                    class="border-input bg-background ring-offset-background placeholder:text-muted-foreground focus-visible:ring-ring flex h-7 w-24 rounded-md border px-3 py-2 text-sm file:border-0 file:bg-transparent file:text-sm file:font-medium focus-visible:ring-2 focus-visible:ring-offset-2 focus-visible:outline-none disabled:cursor-not-allowed disabled:opacity-50"
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
          <!-- 显示内置指令 -->
          {#each filteredCommands as command}
            <div class="group/box mb-4">
              <div class="mb-2">
                <h4 class="text-sm font-semibold">
                  {command.title}
                </h4>
              </div>
              <div class="flex flex-wrap gap-2">
                {#each command.keywords as keyword}
                  <div
                    class="group/button border-input text-foreground shadow-btn hover:bg-muted relative inline-flex cursor-pointer items-center justify-center rounded-full border px-2 py-1 text-sm font-medium select-none active:scale-[0.98] {keyword.disabled
                      ? 'bg-neutral-200 text-neutral-500 line-through dark:bg-neutral-700 dark:text-neutral-400'
                      : 'bg-white dark:bg-neutral-800'}"
                  >
                    <DropdownMenu.Root>
                      <DropdownMenu.Trigger>
                        {keyword.name}
                      </DropdownMenu.Trigger>
                      <DropdownMenu.Portal>
                        <DropdownMenu.Content
                          class="border-muted bg-background shadow-popover w-[229px] rounded-xl border px-1 py-1.5 outline-hidden focus-visible:outline-hidden"
                          sideOffset={8}
                          align="start"
                        >
                          <DropdownMenu.Item
                            class="rounded-button data-highlighted:bg-muted flex h-10 items-center py-3 pr-1.5 pl-3 text-sm font-medium ring-0! ring-transparent! select-none focus-visible:outline-none"
                          >
                            <Button.Root
                              class="w-full"
                              onclick={() => {
                                executeCommand(command.name);
                              }}
                            >
                              执行指令
                            </Button.Root>
                          </DropdownMenu.Item>
                          <DropdownMenu.Item
                            class="rounded-button data-highlighted:bg-muted flex h-10 items-center py-3 pr-1.5 pl-3 text-sm font-medium ring-0! ring-transparent! select-none focus-visible:outline-none"
                          >
                            <Button.Root
                              class="w-full"
                              onclick={() => {
                                toggleKeywordDisabled(command.name, keyword.name);
                              }}
                            >
                              {keyword.disabled ? "启用指令" : "禁用指令"}
                            </Button.Root>
                          </DropdownMenu.Item>
                        </DropdownMenu.Content>
                      </DropdownMenu.Portal>
                    </DropdownMenu.Root>
                    {#if !keyword.is_default}
                      <Button.Root
                        class="absolute -top-1 -right-1 hidden h-4 w-4 items-center justify-center rounded-full bg-red-500 text-white group-hover/button:flex"
                        onclick={() => {
                          removeKeyword(command.name, keyword.name);
                        }}
                      >
                        &times;
                      </Button.Root>
                    {/if}
                  </div>
                {/each}
                <div class="hidden group-hover/box:block">
                  <input
                    type="text"
                    placeholder="添加关键字"
                    class="border-input bg-background ring-offset-background placeholder:text-muted-foreground focus-visible:ring-ring flex h-7 w-24 rounded-md border px-3 py-2 text-sm file:border-0 file:bg-transparent file:text-sm file:font-medium focus-visible:ring-2 focus-visible:ring-offset-2 focus-visible:outline-none disabled:cursor-not-allowed disabled:opacity-50"
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
      </Tabs.Content>
      <Tabs.Content value="match" class="pt-3 select-none">
        <!-- Placeholder for match commands -->
      </Tabs.Content>
    </Tabs.Root>
  </div>
</main>
