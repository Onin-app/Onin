<script lang="ts">
  import { Button, Combobox, ScrollArea } from "bits-ui";
  import { onMount } from "svelte";
  import ShortcutInput from "./ShortcutInput.svelte";
  import { invoke } from "@tauri-apps/api/core";
  import {
    CaretUpDown,
    Check,
    Command as CommandIcon,
    CaretDoubleUp,
    CaretDoubleDown,
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
      // Maybe show a notification to the user
      return;
    }
    try {
      await invoke("add_shortcut", { shortcut });
    } catch (error) {
      console.error("Failed to add shortcut:", error);
      // Revert UI or notify user
    }
  }
</script>

<main class="flex h-full flex-col">
  <div class="mb-4 grid grid-cols-[1fr_1fr_auto] items-center gap-4">
    <div class="font-semibold">快捷键</div>
    <div class="font-semibold">目标指令</div>
    <div></div>
  </div>

  <ScrollArea.Root class="flex-1 overflow-hidden" type="hover">
    <ScrollArea.Viewport class="h-full w-full">
      {#each shortcuts as shortcutInfo, index}
        <div
          class="mb-2 grid grid-cols-[1fr_1fr_auto] items-center gap-4 rounded"
          class:opacity-50={shortcutInfo.readonly}
        >
          <ShortcutInput
            bind:value={shortcutInfo.shortcut}
            onSave={() => saveShortcut(shortcutInfo)}
            disabled={shortcutInfo.readonly}
          />
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
              <CommandIcon
                class="text-muted-foreground absolute start-3 top-1/2 size-6 -translate-y-1/2"
              />
              <Combobox.Input
                oninput={(e) => (searchValue = e.currentTarget.value)}
                class="h-input rounded-9px border-border-input bg-background placeholder:text-foreground-alt/50 focus:ring-foreground focus:ring-offset-background inline-flex w-full touch-none truncate border px-11 text-base transition-colors focus:ring-2 focus:ring-offset-2 focus:outline-hidden sm:text-sm"
                placeholder="搜索一个指令"
                aria-label="搜索一个指令"
              />
              <Combobox.Trigger
                class="absolute end-3 top-1/2 size-6 -translate-y-1/2 touch-none"
              >
                <CaretUpDown class="text-muted-foreground size-6" />
              </Combobox.Trigger>
              <Combobox.Portal>
                <Combobox.Content
                  class="focus-override border-muted bg-background shadow-popover data-[state=open]:animate-in data-[state=closed]:animate-out data-[state=closed]:fade-out-0 data-[state=open]:fade-in-0 data-[state=closed]:zoom-out-95 data-[state=open]:zoom-in-95 data-[side=bottom]:slide-in-from-top-2 data-[side=left]:slide-in-from-right-2 data-[side=right]:slide-in-from-left-2 data-[side=top]:slide-in-from-bottom-2 z-50 h-96 max-h-[var(--bits-combobox-content-available-height)] w-[var(--bits-combobox-anchor-width)] min-w-[var(--bits-combobox-anchor-width)] rounded-xl border px-1 py-3 outline-hidden select-none data-[side=bottom]:translate-y-1 data-[side=left]:-translate-x-1 data-[side=right]:translate-x-1 data-[side=top]:-translate-y-1"
                  sideOffset={10}
                >
                  <Combobox.ScrollUpButton
                    class="flex w-full items-center justify-center py-1"
                  >
                    <CaretDoubleUp class="size-3" />
                  </Combobox.ScrollUpButton>
                  <Combobox.Viewport class="p-1">
                    {#each filteredCommands as command, i (i + command.title)}
                      <Combobox.Item
                        class="rounded-button data-highlighted:bg-muted flex h-10 w-full items-center py-3 pr-1.5 pl-5 text-sm capitalize outline-hidden select-none"
                        value={command.name}
                        label={command.title}
                      >
                        {#snippet children({ selected })}
                          {command.title}
                          {#if selected}
                            <div class="ml-auto">
                              <Check />
                            </div>
                          {/if}
                        {/snippet}
                      </Combobox.Item>
                    {:else}
                      <span
                        class="block px-5 py-2 text-sm text-muted-foreground"
                      >
                        No results found, try again.
                      </span>
                    {/each}
                  </Combobox.Viewport>
                  <Combobox.ScrollDownButton
                    class="flex w-full items-center justify-center py-1"
                  >
                    <CaretDoubleDown class="size-3" />
                  </Combobox.ScrollDownButton>
                </Combobox.Content>
              </Combobox.Portal>
            </Combobox.Root>
          </div>
          <Button.Root
            class="rounded-input shadow-mini inline-flex h-8 w-8 items-center justify-center bg-red-500 p-0 text-lg font-semibold text-white hover:bg-red-600 active:scale-[0.98] active:transition-all"
            onclick={() => removeShortcut(index)}
            disabled={shortcutInfo.readonly}
          >
            &times;
          </Button.Root>
        </div>
      {/each}

      <Button.Root
        class="rounded-input bg-dark text-background shadow-mini hover:bg-dark/95 mt-4 inline-flex h-8 items-center justify-center px-[14px] text-[12px] font-semibold active:scale-[0.98] active:transition-all"
        onclick={addShortcut}
      >
        + 新增
      </Button.Root>
    </ScrollArea.Viewport>
    <ScrollArea.Scrollbar
      orientation="vertical"
      class="bg-muted hover:bg-dark-10 data-[state=visible]:animate-in data-[state=hidden]:animate-out data-[state=hidden]:fade-out-0 data-[state=visible]:fade-in-0 flex w-1.5 touch-none rounded-full border-l border-l-transparent p-px transition-all duration-200 select-none hover:w-3"
    >
      <ScrollArea.Thumb class="bg-muted-foreground flex-1 rounded-full" />
    </ScrollArea.Scrollbar>
    <ScrollArea.Corner />
  </ScrollArea.Root>
</main>
