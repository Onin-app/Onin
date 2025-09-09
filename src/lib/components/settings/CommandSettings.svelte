<script lang="ts">
  import { Button, Command, DropdownMenu, Tabs } from "bits-ui";

  // 定义关键词类型
  type Keyword = {
    name: string;
    disabled: boolean;
  };

  // 定义命令类型
  type CommandType = {
    title: string;
    command: string;
    keywords: Keyword[];
  };

  // 基础指令
  let basicCommands = $state({
    // 功能指令
    function: [
      {
        title: "屏幕截图",
        command: "screenshot",
        keywords: [
          {
            name: "截图",
            disabled: false,
          },
          {
            name: "Screenshot",
            disabled: true,
          },
        ],
      },
      {
        title: "屏幕颜色拾取",
        command: "pick-color",
        keywords: [
          {
            name: "取色",
            disabled: false,
          },
          {
            name: "Pick Color",
            disabled: false,
          },
        ],
      },
    ],
    // 匹配指令
    match: [],
  });

  const builtInCommandList = [
    {
      name: "基础常用",
      id: "basic",
      commandType: basicCommands,
    },
  ];

  let activeSetting = $state(builtInCommandList[0]);

  // 切换关键词的启用/禁用状态
  function toggleKeywordDisabled(commandTitle: string, keywordName: string) {
    // 直接修改状态并强制触发响应式更新
    const command = activeSetting.commandType.function.find(
      (cmd) => cmd.title === commandTitle,
    );
    if (command) {
      const keyword = command.keywords.find((kw) => kw.name === keywordName);
      if (keyword) {
        keyword.disabled = !keyword.disabled;
        // 通过重新赋值触发响应式更新
        activeSetting = { ...activeSetting };
      }
    }
  }
</script>

<main class="flex h-full">
  <div class="w-36 border-r border-neutral-200 dark:border-neutral-700">
    <h3 class="p-2 text-sm text-gray-500 dark:text-gray-400">内置指令</h3>
    <ul class="flex w-full flex-col justify-center">
      {#each builtInCommandList as command}
        <li
          class={activeSetting.id === command.id
            ? "bg-neutral-300 dark:bg-neutral-600"
            : "hover:bg-neutral-200 dark:hover:bg-neutral-700"}
        >
          <Button.Root
            class="h-full cursor-pointer px-2 py-1"
            onclick={() => (activeSetting = command)}
          >
            {command.name}
          </Button.Root>
        </li>
      {/each}
    </ul>
  </div>
  <div class="flex-1 overflow-scroll p-3">
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
        {#each activeSetting.commandType.function as command}
          <div class="mb-2">
            <h4 class="mb-1">{command.title}</h4>
            {#each command.keywords as keyword}
              <DropdownMenu.Root>
                <DropdownMenu.Trigger
                  class="border-input text-foreground shadow-btn hover:bg-muted inline-flex cursor-pointer items-center justify-center rounded-full border px-2 py-1 text-sm font-medium select-none active:scale-[0.98] {keyword.disabled
                    ? 'bg-neutral-200 text-neutral-500 dark:bg-neutral-700 dark:text-neutral-400'
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
                      class="rounded-button data-highlighted:bg-muted flex h-10 items-center py-3 pr-1.5 pl-3 text-sm font-medium ring-0! ring-transparent! select-none focus-visible:outline-none {keyword.disabled
                        ? 'cursor-not-allowed opacity-50'
                        : ''}"
                    >
                      <div class="flex items-center">打开指令</div>
                    </DropdownMenu.Item>
                    <DropdownMenu.Item
                      class="rounded-button data-highlighted:bg-muted flex h-10 items-center py-3 pr-1.5 pl-3 text-sm font-medium ring-0! ring-transparent! select-none focus-visible:outline-none"
                      onclick={() =>
                        toggleKeywordDisabled(command.title, keyword.name)}
                    >
                      <div class="flex items-center">
                        {keyword.disabled ? "启用指令" : "禁用指令"}
                      </div>
                    </DropdownMenu.Item>
                  </DropdownMenu.Content>
                </DropdownMenu.Portal>
              </DropdownMenu.Root>
            {/each}
          </div>
        {/each}
      </Tabs.Content>
      <Tabs.Content value="match" class="pt-3 select-none">
        {#each activeSetting.commandType.match as command}
          <div>
            <!-- <span>{command.name}</span>
            <span>{command.description}</span> -->
          </div>
        {/each}
      </Tabs.Content>
    </Tabs.Root>
  </div>
</main>
