<script lang="ts">
  /**
   * PluginMenu Component
   *
   * 插件操作下拉菜单组件
   * 包含分离窗口、关闭插件、自动分离开关
   */
  import { DropdownMenu } from "bits-ui";
  import {
    AppWindow,
    DotsThreeVertical,
    X,
    PuzzlePiece,
    Check,
  } from "phosphor-svelte";

  interface Props {
    autoDetach: boolean;
    detachShortcut: string | null;
    onDetach: () => void;
    onClose: () => void;
    onToggleAutoDetach: (checked: boolean) => void;
  }

  let {
    autoDetach = $bindable(),
    detachShortcut,
    onDetach,
    onClose,
    onToggleAutoDetach,
  }: Props = $props();
</script>

<DropdownMenu.Root>
  <DropdownMenu.Trigger class="ml-2 cursor-pointer">
    <DotsThreeVertical class="size-8" />
  </DropdownMenu.Trigger>
  <DropdownMenu.Portal>
    <DropdownMenu.Content
      class="border-muted bg-background shadow-popover outline-hidden focus-visible:outline-hidden rounded-xl border px-1 py-1.5"
      sideOffset={8}
    >
      <!-- 分离窗口 -->
      <DropdownMenu.Item
        class="rounded-button data-highlighted:bg-muted ring-0! ring-transparent! flex h-10 select-none items-center py-3 pl-3 pr-1.5 text-sm font-medium focus-visible:outline-none"
      >
        <button
          class="flex w-full cursor-pointer items-center"
          onclick={onDetach}
        >
          <AppWindow class="text-foreground-alt mr-2 size-5" />
          <span class="mr-2">分离窗口</span>
          {#if detachShortcut}
            <kbd
              class="rounded-button border-dark-10 bg-background-alt text-muted-foreground shadow-kbd ml-auto inline-flex items-center justify-center border px-1 text-xs uppercase"
            >
              {detachShortcut}
            </kbd>
          {/if}
        </button>
      </DropdownMenu.Item>

      <!-- 关闭插件 -->
      <DropdownMenu.Item
        class="rounded-button data-highlighted:bg-muted ring-0! ring-transparent! flex h-10 select-none items-center py-3 pl-3 pr-1.5 text-sm font-medium focus-visible:outline-none"
      >
        <button
          class="flex w-full cursor-pointer items-center"
          onclick={onClose}
        >
          <X class="text-foreground-alt mr-2 size-5" />
          <span class="mr-2">关闭插件</span>
          <kbd
            class="rounded-button border-dark-10 bg-background-alt text-muted-foreground shadow-kbd ml-auto inline-flex items-center justify-center border px-1 text-xs"
          >
            ESC
          </kbd>
        </button>
      </DropdownMenu.Item>

      <!-- 自动分离开关 -->
      <DropdownMenu.CheckboxItem
        bind:checked={autoDetach}
        class="rounded-button data-highlighted:bg-muted ring-0! ring-transparent! flex h-10 cursor-pointer select-none items-center py-3 pl-3 pr-1.5 text-sm font-medium focus-visible:outline-none"
        onCheckedChange={(checked) => {
          if (typeof checked === "boolean") {
            onToggleAutoDetach(checked);
          }
        }}
      >
        {#snippet children({ checked })}
          <div class="flex items-center pr-4">
            <PuzzlePiece class="text-foreground-alt mr-2 size-5" />
            自动分离为独立窗口
          </div>
          <div class="ml-auto flex items-center gap-px">
            {#if checked}
              <Check class="size-4" />
            {/if}
          </div>
        {/snippet}
      </DropdownMenu.CheckboxItem>
    </DropdownMenu.Content>
  </DropdownMenu.Portal>
</DropdownMenu.Root>
