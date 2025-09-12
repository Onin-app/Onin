<script lang="ts">
  import { Button, DropdownMenu, Switch, Tabs } from "bits-ui";
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import type { Command } from "$lib/type";

  let commands = $state<Command[]>([]);

  onMount(async () => {
    try {
      commands = await invoke<Command[]>("get_basic_commands");
      console.log("Fetched basic commands:", commands);
    } catch (error) {
      console.error("Failed to fetch basic commands:", error);
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

  const sourceNameMap = {
    Command: "基础常用",
    Application: "程序启动",
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

  let filteredCommands = $derived(
    commands.filter((cmd) => cmd.source === activeCategory?.id),
  );
</script>

<main class="flex h-full">
  <div class="w-36 border-r border-neutral-200 dark:border-neutral-700">
    <h3 class="p-2 text-sm text-gray-500 dark:text-gray-400">指令类型</h3>
    <ul class="flex w-full flex-col justify-center">
      {#each commandCategories as category}
        <li
          class={activeCategory?.id === category.id
            ? "bg-neutral-300 dark:bg-neutral-600"
            : "hover:bg-neutral-200 dark:hover:bg-neutral-700"}
        >
          <Button.Root
            class="h-full w-full cursor-pointer px-2 py-1 text-left"
            onclick={() => (activeCategory = category)}
          >
            {category.name}
          </Button.Root>
        </li>
      {/each}
    </ul>
  </div>
  <div class="flex-1 overflow-y-auto p-3">
    <Tabs.Root value="function">
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
      <Tabs.Content value="function" class="pt-3 select-none">
        {#each filteredCommands as command}
          <div class="mb-4">
            <div class="mb-2 flex items-center">
              <h4 class="font-semibold">{command.title}</h4>
            </div>
            <div class="flex flex-wrap gap-2">
              {#each command.keywords as keyword}
                <DropdownMenu.Root>
                  <DropdownMenu.Trigger
                    class="border-input text-foreground shadow-btn hover:bg-muted inline-flex cursor-pointer items-center justify-center rounded-full border px-2 py-1 text-sm font-medium select-none active:scale-[0.98] {keyword.disabled
                      ? 'bg-neutral-200 text-neutral-500 line-through dark:bg-neutral-700 dark:text-neutral-400'
                      : 'bg-white dark:bg-neutral-800'}"
                  >
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
                          onclick={() =>
                            toggleKeywordDisabled(command.name, keyword.name)}
                        >
                          {keyword.disabled ? "启用指令" : "禁用指令"}
                        </Button.Root>
                      </DropdownMenu.Item>
                    </DropdownMenu.Content>
                  </DropdownMenu.Portal>
                </DropdownMenu.Root>
              {/each}
            </div>
          </div>
        {/each}
      </Tabs.Content>
      <Tabs.Content value="match" class="pt-3 select-none">
        <!-- Placeholder for match commands -->
      </Tabs.Content>
    </Tabs.Root>
  </div>
</main>
