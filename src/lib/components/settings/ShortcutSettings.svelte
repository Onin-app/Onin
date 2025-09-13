<script lang="ts">
  import { Button, Tabs } from "bits-ui";
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";

  // 指令别名相关状态
  let commandAliases = $state<Array<{
    id: string;
    alias: string;
    command: string;
    enabled: boolean;
  }>>([]);

  onMount(async () => {
    try {
      // 加载指令别名
      const aliases = await invoke<typeof commandAliases>("get_command_aliases");
      // 为从后端加载的别名添加已保存标记
      commandAliases = aliases.map(alias => ({
        ...alias,
        id: 'saved_' + alias.id
      }));
    } catch (error) {
      console.error("Failed to load command aliases:", error);
      // 如果API调用失败，使用空数组作为默认值
      commandAliases = [];
    }
  });

  // 添加新的空行
  function addNewRow() {
    const newAlias = {
      id: Date.now().toString(),
      alias: "",
      command: "",
      enabled: true,
    };
    commandAliases = [...commandAliases, newAlias];
  }

  // 删除指令别名
  async function removeAlias(id: string) {
    try {
      // 如果是已保存的别名，需要从后端删除
      if (id.includes('saved_')) {
        const realId = id.replace('saved_', '');
        await invoke("remove_command_alias", { id: realId });
      }
      // 从前端列表中移除
      commandAliases = commandAliases.filter(a => a.id !== id);
    } catch (error) {
      console.error("Failed to remove alias:", error);
    }
  }

  // 更新别名
  async function updateAlias(id: string, field: 'alias' | 'command', value: string) {
    const alias = commandAliases.find(a => a.id === id);
    if (alias) {
      alias[field] = value;
      commandAliases = [...commandAliases];
      
      // 只有当别名和指令都不为空时才保存到后端
      if (alias.alias.trim() && alias.command.trim()) {
        try {
          // 检查是否是新创建的别名（还没有保存到后端）
          const isNewAlias = !alias.id.includes('saved_');
          
          if (isNewAlias) {
            // 新别名，使用add_command_alias
            await invoke("add_command_alias", { alias });
            // 更新ID标记为已保存
            alias.id = 'saved_' + alias.id;
            commandAliases = [...commandAliases];
          } else {
            // 已存在的别名，使用update_command_alias
            await invoke("update_command_alias", { alias });
          }
        } catch (error) {
          console.error("Failed to save alias:", error);
        }
      }
    }
  }
</script>

<main class="h-full p-4">
  <Tabs.Root value="aliases" class="flex h-full flex-col">
    <Tabs.List
      class="rounded-9px bg-dark-10 shadow-mini-inset dark:bg-background grid w-full grid-cols-2 gap-1 p-1 text-sm leading-[0.01em] font-semibold dark:border dark:border-neutral-600/30"
    >
      <Tabs.Trigger
        value="shortcuts"
        class="data-[state=active]:shadow-mini dark:data-[state=active]:bg-muted h-8 rounded-[7px] bg-transparent py-2 data-[state=active]:bg-white"
      >
        全局快捷键
      </Tabs.Trigger>
      <Tabs.Trigger
        value="aliases"
        class="data-[state=active]:shadow-mini dark:data-[state=active]:bg-muted h-8 rounded-[7px] bg-transparent py-2 data-[state=active]:bg-white"
      >
        指令别名
      </Tabs.Trigger>
    </Tabs.List>

    <!-- 全局快捷键 Tab -->
    <Tabs.Content value="shortcuts" class="flex-1 overflow-y-auto pt-4">
      <div class="flex flex-col items-center justify-center py-12 text-center">
        <div class="mb-4 flex h-16 w-16 items-center justify-center rounded-full bg-gray-100 dark:bg-gray-800">
          <span class="text-2xl text-gray-400">⌨️</span>
        </div>
        <h3 class="mb-2 text-lg font-medium text-gray-900 dark:text-gray-100">全局快捷键</h3>
        <p class="text-sm text-gray-500 dark:text-gray-400">
          此功能正在开发中，敬请期待
        </p>
      </div>
    </Tabs.Content>

    <!-- 指令别名 Tab -->
    <Tabs.Content value="aliases" class="flex-1 overflow-y-auto pt-4">
      <div class="space-y-6">
        <!-- 表头 -->
        <div class="grid grid-cols-2 gap-4 text-sm font-medium text-gray-600 dark:text-gray-400">
          <div>自定义别名</div>
          <div>目标指令</div>
        </div>

        <!-- 别名列表 -->
        <div class="space-y-3">
          {#each commandAliases as alias}
            <div class="grid grid-cols-2 gap-4 items-center">
              <div class="relative">
                <input
                  bind:value={alias.alias}
                  onblur={(e) => updateAlias(alias.id, 'alias', (e.target as HTMLInputElement)?.value || '')}
                  class="w-full rounded-lg border border-neutral-300 px-3 py-2 text-sm focus:border-blue-500 focus:outline-none dark:border-neutral-600 dark:bg-neutral-800 dark:focus:border-blue-400"
                  placeholder="输入别名"
                />
              </div>
              <div class="flex items-center gap-2">
                <input
                  bind:value={alias.command}
                  onblur={(e) => updateAlias(alias.id, 'command', (e.target as HTMLInputElement)?.value || '')}
                  class="flex-1 rounded-lg border border-neutral-300 px-3 py-2 text-sm focus:border-blue-500 focus:outline-none dark:border-neutral-600 dark:bg-neutral-800 dark:focus:border-blue-400"
                  placeholder="输入目标指令"
                />
                <Button.Root
                  onclick={() => removeAlias(alias.id)}
                  class="flex h-8 w-8 items-center justify-center rounded-lg bg-gray-100 text-gray-500 hover:bg-red-100 hover:text-red-600 dark:bg-gray-800 dark:text-gray-400 dark:hover:bg-red-900 dark:hover:text-red-400"
                >
                  ×
                </Button.Root>
              </div>
            </div>
          {/each}
        </div>

        <!-- 新增按钮 -->
        <div class="pt-4">
          <Button.Root
            onclick={addNewRow}
            class="flex items-center gap-2 rounded-lg border border-neutral-300 px-2 text-sm text-gray-600 hover:border-blue-400 hover:text-blue-600 dark:border-neutral-600 dark:text-gray-400 dark:hover:border-blue-500 dark:hover:text-blue-400"
          >
            <span class="text-lg">+</span>
            新增
          </Button.Root>
        </div>

        <!-- 空状态 -->
        {#if commandAliases.length === 0}
          <div class="flex flex-col items-center justify-center py-12 text-center">
            <div class="mb-4 flex h-16 w-16 items-center justify-center rounded-full bg-gray-100 dark:bg-gray-800">
              <span class="text-2xl text-gray-400">📝</span>
            </div>
            <h3 class="mb-2 text-lg font-medium text-gray-900 dark:text-gray-100">暂无指令别名</h3>
            <p class="mb-4 text-sm text-gray-500 dark:text-gray-400">
              创建指令别名可以让你更快速地执行常用指令
            </p>
            <Button.Root
              onclick={addNewRow}
              class="rounded-lg bg-blue-600 px-4 py-2 text-sm text-white hover:bg-blue-700"
            >
              创建第一个别名
            </Button.Root>
          </div>
        {/if}
      </div>
    </Tabs.Content>
  </Tabs.Root>
</main>