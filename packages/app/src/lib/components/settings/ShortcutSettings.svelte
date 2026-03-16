<script lang="ts">
  import { Button, Combobox } from "bits-ui";
  import AppScrollArea from "$lib/components/AppScrollArea.svelte";
  import { onMount } from "svelte";
  import ShortcutInput from "./ShortcutInput.svelte";
  import { invoke } from "@tauri-apps/api/core";
  import {
    CaretUpDown,
    Check,
    CaretDoubleUp,
    CaretDoubleDown,
    X,
    Plus,
  } from "phosphor-svelte";
  import type { Command, Shortcut } from "$lib/type";

  let shortcuts = $state<Shortcut[]>([]);
  let commands = $state<Command[]>([]);
  let searchValue = $state("");

  onMount(async () => {
    try {
      shortcuts = await invoke<Shortcut[]>("get_shortcuts");
      shortcuts.forEach((s) => {
        if (s.command_name === "toggle_window") {
          s.readonly = true;
          s.command_title = "显示/隐藏窗口";
        } else if (s.command_name === "detach_window") {
          s.readonly = true;
          s.command_title = "分离窗口";
        }
      });
      shortcuts.sort((a, b) => {
        if (a.readonly && !b.readonly) return -1;
        if (!a.readonly && b.readonly) return 1;
        return 0;
      });
    } catch (error) {
      console.error("Failed to fetch shortcuts:", error);
    }
    try {
      commands = await invoke<Command[]>("get_commands");
    } catch (error) {
      console.error("Failed to fetch commands:", error);
    }
  });

  const filteredCommands = $derived(
    searchValue === ""
      ? commands
      : commands.filter((command) =>
          command.title.toLowerCase().includes(searchValue.toLowerCase()),
        ),
  );

  async function addShortcut() {
    const newShortcut = { shortcut: "", command_name: "" };
    shortcuts.push(newShortcut);
    shortcuts = [...shortcuts];
  }

  async function removeShortcut(index: number) {
    const shortcutToRemove = shortcuts[index];
    if (shortcutToRemove && shortcutToRemove.shortcut) {
      try {
        await invoke("remove_shortcut", {
          shortcutStr: shortcutToRemove.shortcut,
        });
        shortcuts.splice(index, 1);
        shortcuts = [...shortcuts];
      } catch (error) {
        console.error("Failed to remove shortcut:", error);
      }
    } else {
      shortcuts.splice(index, 1);
      shortcuts = [...shortcuts];
    }
  }

  async function saveShortcut(shortcut: Shortcut) {
    if (!shortcut.shortcut || !shortcut.command_name) {
      return;
    }
    try {
      await invoke("add_shortcut", { shortcut });
    } catch (error) {
      console.error("Failed to add shortcut:", error);
    }
  }
</script>

<main class="flex h-full flex-col gap-4">
  <!-- 简单的说明 -->
  <div class="flex items-center justify-between px-1">
    <div class="space-y-1">
      <h2 class="text-sm font-semibold text-neutral-900 dark:text-neutral-100">
        快捷键配置
      </h2>
      <p class="text-xs text-neutral-500 dark:text-neutral-400">
        自定义全局热键来快速触发常用指令
      </p>
    </div>
  </div>

  <!-- 列表容器 -->
  <AppScrollArea class="-mr-2 flex-1 overflow-hidden" viewportClass="h-full w-full pr-2">
      <div
        class="overflow-hidden rounded-xl border border-neutral-200 bg-white shadow-sm dark:border-neutral-800 dark:bg-neutral-900"
      >
        {#if shortcuts.length === 0}
          <div
            class="flex h-32 flex-col items-center justify-center p-4 text-center"
          >
            <p class="text-sm text-neutral-500">暂无任何快捷键</p>
            <Button.Root
              class="mt-2 text-xs font-medium text-blue-600 hover:underline dark:text-blue-400"
              onclick={addShortcut}
            >
              立即添加
            </Button.Root>
          </div>
        {:else}
          <div class="divide-y divide-neutral-100 dark:divide-neutral-800">
            {#each shortcuts as shortcutInfo, index}
              <div
                class="group flex items-center gap-4 bg-white px-4 py-3 transition-colors hover:bg-neutral-50 dark:bg-neutral-900 dark:hover:bg-neutral-800/50"
                class:opacity-75={shortcutInfo.readonly}
              >
                <!-- 快捷键输入 -->
                <div class="w-1/3 min-w-[140px]">
                  <ShortcutInput
                    bind:value={shortcutInfo.shortcut}
                    onSave={() => saveShortcut(shortcutInfo)}
                    disabled={shortcutInfo.readonly}
                  />
                </div>

                <!-- 目标指令 -->
                <div class="flex-1">
                  <div class="relative">
                    <Combobox.Root
                      type="single"
                      name="command"
                      disabled={shortcutInfo.readonly}
                      inputValue={shortcutInfo.command_title || ""}
                      onOpenChange={(o) => {
                        if (!o) searchValue = "";
                      }}
                      onValueChange={(value) => {
                        saveShortcut({
                          shortcut: shortcutInfo.shortcut,
                          command_name: value,
                        });
                        searchValue = "";
                      }}
                    >
                      <div class="relative w-full">
                        <Combobox.Input
                          oninput={(e) => (searchValue = e.currentTarget.value)}
                          class="h-8 w-full rounded-md border border-transparent bg-transparent px-2 text-sm font-medium text-neutral-900 transition-all placeholder:text-neutral-400 hover:bg-neutral-100 focus:border-neutral-200 focus:bg-white focus:ring-2 focus:ring-neutral-100 focus:outline-none dark:text-neutral-100 dark:placeholder:text-neutral-600 dark:hover:bg-neutral-800 dark:focus:border-neutral-700 dark:focus:bg-neutral-900 dark:focus:ring-neutral-800"
                          placeholder="选择触发指令..."
                          aria-label="选择触发指令"
                        />
                        {#if !shortcutInfo.readonly}
                          <Combobox.Trigger
                            class="absolute top-1/2 right-2 -translate-y-1/2 text-neutral-400 opacity-0 transition-opacity group-hover:opacity-100"
                          >
                            <CaretUpDown class="h-4 w-4" />
                          </Combobox.Trigger>
                        {/if}
                      </div>

                      <Combobox.Portal>
                        <Combobox.Content
                          class="z-50 max-h-64 w-[var(--bits-combobox-anchor-width)] overflow-hidden rounded-lg border border-neutral-200 bg-white shadow-lg dark:border-neutral-700 dark:bg-neutral-800"
                          sideOffset={4}
                        >
                          <Combobox.ScrollUpButton
                            class="flex w-full items-center justify-center py-1 text-neutral-400"
                          >
                            <CaretDoubleUp class="h-3 w-3" />
                          </Combobox.ScrollUpButton>
                          <Combobox.Viewport class="p-1">
                            {#each filteredCommands as command, i (i + command.title)}
                              <Combobox.Item
                                class="flex cursor-pointer items-center rounded-md px-2 py-1.5 text-sm text-neutral-700 outline-none select-none data-[highlighted]:bg-neutral-100 dark:text-neutral-200 dark:data-[highlighted]:bg-neutral-700"
                                value={command.name}
                                label={command.title}
                              >
                                {#snippet children({ selected })}
                                  <span class="flex-1">{command.title}</span>
                                  {#if selected}
                                    <Check
                                      class="h-4 w-4 text-neutral-600 dark:text-neutral-300"
                                    />
                                  {/if}
                                {/snippet}
                              </Combobox.Item>
                            {:else}
                              <div
                                class="px-2 py-3 text-center text-sm text-neutral-400"
                              >
                                无匹配结果
                              </div>
                            {/each}
                          </Combobox.Viewport>
                          <Combobox.ScrollDownButton
                            class="flex w-full items-center justify-center py-1 text-neutral-400"
                          >
                            <CaretDoubleDown class="h-3 w-3" />
                          </Combobox.ScrollDownButton>
                        </Combobox.Content>
                      </Combobox.Portal>
                    </Combobox.Root>
                  </div>
                </div>

                <!-- 操作区 -->
                <div class="flex w-8 items-center justify-end">
                  {#if shortcutInfo.readonly}
                    <span
                      class="rounded bg-neutral-100 px-1.5 py-0.5 text-[10px] text-neutral-500 dark:bg-neutral-800"
                      >系统</span
                    >
                  {:else}
                    <Button.Root
                      class="flex h-7 w-7 items-center justify-center rounded-md text-neutral-400 opacity-0 transition-all group-hover:opacity-100 hover:bg-neutral-100 hover:text-red-500 dark:hover:bg-neutral-800"
                      onclick={() => removeShortcut(index)}
                    >
                      <X class="h-4 w-4" />
                    </Button.Root>
                  {/if}
                </div>
              </div>
            {/each}
          </div>

          <!-- 底部添加栏 -->
          <div
            class="border-t border-neutral-100 bg-neutral-50 px-3 py-2 dark:border-neutral-800 dark:bg-neutral-900/50"
          >
            <Button.Root
              class="flex w-full items-center justify-center gap-2 rounded-lg border border-dashed border-neutral-300 bg-transparent py-2 text-sm font-medium text-neutral-500 transition-colors hover:border-neutral-400 hover:bg-white hover:text-neutral-900 active:scale-[0.99] dark:border-neutral-700 dark:text-neutral-400 dark:hover:bg-neutral-800 dark:hover:text-neutral-200"
              onclick={addShortcut}
            >
              <Plus class="h-4 w-4" />
              添加新快捷键
            </Button.Root>
          </div>
        {/if}
      </div>
  </AppScrollArea>
</main>
