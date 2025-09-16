<script lang="ts">
  import { Button, Combobox } from "bits-ui";
  import { onMount } from "svelte";
  import ShortcutInput from "./ShortcutInput.svelte";
  import { invoke } from "@tauri-apps/api/core";
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
          s.command_name = "显示/隐藏窗口";
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

<main class="p-4">
  <div class="mb-4 grid grid-cols-[1fr_1fr_auto] items-center gap-4">
    <div class="font-semibold">快捷键</div>
    <div class="font-semibold">目标指令</div>
    <div></div>
  </div>

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
      <Combobox.Root
        type="single"
        name="command"
        disabled={shortcutInfo.readonly}
        inputValue={shortcutInfo.readonly
          ? "显示/隐藏窗口"
          : shortcutInfo.command_title}
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
        <div class="relative">
          <span
            class="text-muted-foreground absolute start-3 top-1/2 size-6 -translate-y-1/2"
          >
            😍
          </span>
          <Combobox.Input
            oninput={(e) => (searchValue = e.currentTarget.value)}
            class="h-input rounded-9px border-border-input bg-background placeholder:text-foreground-alt/50 focus:ring-foreground focus:ring-offset-background inline-flex w-[296px] touch-none truncate border px-11 text-base transition-colors focus:ring-2 focus:ring-offset-2 focus:outline-hidden sm:text-sm"
            placeholder="搜索一个指令"
            aria-label="搜索一个指令"
          />
          <Combobox.Trigger
            class="absolute end-3 top-1/2 size-6 -translate-y-1/2 touch-none"
          >
            <span class="text-muted-foreground size-6">😇</span>
          </Combobox.Trigger>
        </div>
        <Combobox.Portal>
          <Combobox.Content
            class="focus-override border-muted bg-background shadow-popover data-[state=open]:animate-in data-[state=closed]:animate-out data-[state=closed]:fade-out-0 data-[state=open]:fade-in-0 data-[state=closed]:zoom-out-95 data-[state=open]:zoom-in-95 data-[side=bottom]:slide-in-from-top-2 data-[side=left]:slide-in-from-right-2 data-[side=right]:slide-in-from-left-2 data-[side=top]:slide-in-from-bottom-2 z-50 h-96 max-h-[var(--bits-combobox-content-available-height)] w-[var(--bits-combobox-anchor-width)] min-w-[var(--bits-combobox-anchor-width)] rounded-xl border px-1 py-3 outline-hidden select-none data-[side=bottom]:translate-y-1 data-[side=left]:-translate-x-1 data-[side=right]:translate-x-1 data-[side=top]:-translate-y-1"
            sideOffset={10}
          >
            <Combobox.ScrollUpButton
              class="flex w-full items-center justify-center py-1"
            >
              🔼
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
                        <span>✓</span>
                      </div>
                    {/if}
                  {/snippet}
                </Combobox.Item>
              {:else}
                <span class="block px-5 py-2 text-sm text-muted-foreground">
                  No results found, try again.
                </span>
              {/each}
            </Combobox.Viewport>
            <Combobox.ScrollDownButton
              class="flex w-full items-center justify-center py-1"
            >
              🔽
            </Combobox.ScrollDownButton>
          </Combobox.Content>
        </Combobox.Portal>
      </Combobox.Root>
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
</main>
