<script lang="ts">
  import { Button } from "$lib/components/ui/button";
  import { Badge } from "$lib/components/ui/badge";
  import { Card } from "$lib/components/ui/card";
  import { ScrollArea } from "$lib/components/ui/scroll-area";
  import { Combobox } from "bits-ui";
  import { onMount } from "svelte";
  import ShortcutInput from "./ShortcutInput.svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { toast } from "svelte-sonner";
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
        } else {
          s.originalShortcut = s.shortcut;
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
    const newShortcut: Shortcut = { shortcut: "", command_name: "" };
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
        toast.success("快捷键已删除");
      } catch (error) {
        console.error("Failed to remove shortcut:", error);
        toast.error("删除快捷键失败");
      }
    } else {
      shortcuts.splice(index, 1);
      shortcuts = [...shortcuts];
    }
  }

  async function saveShortcut(
    shortcut: Shortcut,
    successMessage = "快捷键已保存",
  ) {
    if (!shortcut.shortcut || !shortcut.command_name) {
      return;
    }
    try {
      const hasShortcutChanged =
        shortcut.originalShortcut &&
        shortcut.originalShortcut !== shortcut.shortcut;
      await invoke("add_shortcut", {
        shortcut: {
          shortcut: shortcut.shortcut,
          command_name: shortcut.command_name,
          command_title: shortcut.command_title,
        },
        oldShortcutStr: hasShortcutChanged ? shortcut.originalShortcut : null,
      });
      shortcut.originalShortcut = shortcut.shortcut;
      toast.success(successMessage);
    } catch (error) {
      console.error("Failed to add shortcut:", error);
      toast.error("保存快捷键失败");
    }
  }
</script>

<main class="flex h-full flex-col gap-4">
  <!-- 简单说明 -->
  <div class="flex items-center justify-between px-1">
    <div class="space-y-1">
      <h2 class="text-foreground text-sm font-semibold">快捷键配置</h2>
      <p class="text-muted-foreground text-xs">
        自定义全局热键来快速触发常用指令
      </p>
    </div>
  </div>

  <!-- 列表容器 -->
  <ScrollArea
    class="-mr-2 flex-1 overflow-hidden"
    viewportClass="h-full w-full pr-2"
  >
    <Card class="overflow-hidden p-0">
      {#if shortcuts.length === 0}
        <div
          class="flex h-32 flex-col items-center justify-center p-4 text-center"
        >
          <p class="text-muted-foreground text-sm">暂无任何快捷键</p>
          <Button
            variant="link"
            size="sm"
            class="mt-2 text-xs"
            onclick={addShortcut}
          >
            立即添加
          </Button>
        </div>
      {:else}
        <div class="divide-border/50 divide-y">
          {#each shortcuts as shortcutInfo, index}
            <div
              class="group bg-card hover:bg-accent/40 flex items-center gap-4 px-4 py-3 transition-colors"
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
                      const command = commands.find(
                        (item) => item.name === value,
                      );
                      shortcutInfo.command_name = value;
                      shortcutInfo.command_title = command?.title;
                      saveShortcut(shortcutInfo, "快捷键指令已保存");
                      searchValue = "";
                    }}
                  >
                    <div class="relative w-full">
                      <Combobox.Input
                        oninput={(e) => (searchValue = e.currentTarget.value)}
                        class="text-foreground placeholder:text-muted-foreground hover:bg-accent/50 focus:border-input focus:bg-background focus:ring-ring h-9 w-full rounded-md border border-transparent bg-transparent px-3 text-sm font-medium transition-all focus:ring-1 focus:outline-none"
                        placeholder="选择触发指令..."
                        aria-label="选择触发指令"
                      />
                      {#if !shortcutInfo.readonly}
                        <Combobox.Trigger
                          class="text-muted-foreground absolute top-1/2 right-2 -translate-y-1/2 cursor-pointer opacity-0 transition-opacity group-hover:opacity-100"
                        >
                          <CaretUpDown class="h-4 w-4" />
                        </Combobox.Trigger>
                      {/if}
                    </div>

                    <Combobox.Portal>
                      <Combobox.Content
                        class="bg-popover text-popover-foreground z-50 max-h-64 w-[var(--bits-combobox-anchor-width)] overflow-hidden rounded-lg border shadow-md"
                        sideOffset={4}
                      >
                        <Combobox.ScrollUpButton
                          class="text-muted-foreground flex w-full items-center justify-center py-1"
                        >
                          <CaretDoubleUp class="h-3 w-3" />
                        </Combobox.ScrollUpButton>
                        <Combobox.Viewport class="p-1">
                          {#each filteredCommands as command, i (i + command.title)}
                            <Combobox.Item
                              class="text-popover-foreground data-[highlighted]:bg-accent data-[highlighted]:text-accent-foreground flex cursor-pointer items-center rounded-md px-2 py-1.5 text-sm outline-none select-none"
                              value={command.name}
                              label={command.title}
                            >
                              {#snippet children({ selected })}
                                <span class="flex-1">{command.title}</span>
                                {#if selected}
                                  <Check class="text-primary h-4 w-4" />
                                {/if}
                              {/snippet}
                            </Combobox.Item>
                          {:else}
                            <div
                              class="px-2 py-3 text-center text-sm text-muted-foreground"
                            >
                              无匹配结果
                            </div>
                          {/each}
                        </Combobox.Viewport>
                        <Combobox.ScrollDownButton
                          class="text-muted-foreground flex w-full items-center justify-center py-1"
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
                  <Badge variant="secondary" class="px-1.5 py-0 text-[10px]">
                    系统
                  </Badge>
                {:else}
                  <Button
                    variant="ghost"
                    size="icon"
                    class="text-muted-foreground hover:text-destructive h-7 w-7 opacity-0 transition-opacity group-hover:opacity-100"
                    onclick={() => removeShortcut(index)}
                  >
                    <X class="h-4 w-4" />
                  </Button>
                {/if}
              </div>
            </div>
          {/each}
        </div>

        <!-- 底部添加栏 -->
        <div class="border-border/50 bg-muted/30 border-t p-2">
          <Button
            variant="outline"
            class="w-full border-dashed"
            onclick={addShortcut}
          >
            <Plus class="h-4 w-4" />
            添加新快捷键
          </Button>
        </div>
      {/if}
    </Card>
  </ScrollArea>
</main>
