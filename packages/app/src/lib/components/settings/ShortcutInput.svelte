<script lang="ts">
  import { Button } from "$lib/components/ui/button";
  import { invoke } from "@tauri-apps/api/core";
  import { platform } from "@tauri-apps/plugin-os";
  import { X, Keyboard } from "phosphor-svelte";

  let {
    value = $bindable(""),
    onSave = () => {},
    disabled = false,
    showPresets = false,
  } = $props();

  let isFocused = $state(false);
  let previousShortcut = "";
  let activeModifiers = $state<string[]>([]);
  let inputElement = $state<HTMLInputElement | null>(null);

  // 初始化平台信息
  let currentPlatform = "";
  try {
    if (typeof window !== "undefined" && (window as any).__TAURI__) {
      currentPlatform = platform();
    } else {
      const userAgent = navigator.userAgent.toLowerCase();
      if (userAgent.includes("mac")) {
        currentPlatform = "macos";
      } else if (userAgent.includes("win")) {
        currentPlatform = "windows";
      } else {
        currentPlatform = "linux";
      }
    }
  } catch (error) {
    console.error("Error detecting platform:", error);
    const navPlatform = navigator.platform?.toLowerCase() || "";
    if (navPlatform.includes("mac")) {
      currentPlatform = "macos";
    } else if (navPlatform.includes("win")) {
      currentPlatform = "windows";
    } else {
      currentPlatform = "linux";
    }
  }

  const isMac = currentPlatform === "macos";

  // 根据平台获取预设快捷键
  const getPresetShortcuts = (platformName: string) => {
    const isMacPlatform = platformName === "macos";
    return [
      {
        label: isMacPlatform ? "⌥ Space" : "Alt+Space",
        value: "Alt+Space",
      },
      {
        label: isMacPlatform ? "⌘ Space" : "Ctrl+Space",
        value: "CommandOrControl+Space",
      },
    ];
  };

  const presetShortcuts = $state<{ label: string; value: string }[]>(
    getPresetShortcuts(currentPlatform),
  );

  // 解析快捷键并转换为键帽对象列表
  interface KeyItem {
    raw: string;
    display: string;
    isModifier: boolean;
  }

  const parseShortcutToKeys = (shortcutStr: string): KeyItem[] => {
    if (!shortcutStr) return [];
    const parts = shortcutStr.split("+");

    return parts.map((part) => {
      const trimmed = part.trim();
      const lower = trimmed.toLowerCase();

      switch (lower) {
        case "commandorcontrol":
          return {
            raw: trimmed,
            display: isMac ? "⌘" : "Ctrl",
            isModifier: true,
          };
        case "ctrl":
        case "control":
          return {
            raw: trimmed,
            display: isMac ? "⌃" : "Ctrl",
            isModifier: true,
          };
        case "alt":
        case "option":
          return {
            raw: trimmed,
            display: isMac ? "⌥" : "Alt",
            isModifier: true,
          };
        case "shift":
          return {
            raw: trimmed,
            display: isMac ? "⇧" : "Shift",
            isModifier: true,
          };
        case "super":
        case "meta":
        case "win":
        case "cmd":
        case "command":
          return {
            raw: trimmed,
            display: isMac ? "⌘" : "Win",
            isModifier: true,
          };
        case "space":
          return {
            raw: trimmed,
            display: "Space",
            isModifier: false,
          };
        default:
          return {
            raw: trimmed,
            display: trimmed.length === 1 ? trimmed.toUpperCase() : trimmed,
            isModifier: false,
          };
      }
    });
  };

  let currentKeyItems = $derived(parseShortcutToKeys(value));

  function handleKeyDown(event: KeyboardEvent) {
    if (disabled || !isFocused) return;

    event.preventDefault();
    event.stopPropagation();

    // 处理取消/清除操作 (ESC / Backspace)
    if (event.key === "Escape") {
      value = previousShortcut;
      isFocused = false;
      inputElement?.blur();
      return;
    }

    if (
      (event.key === "Backspace" || event.key === "Delete") &&
      !event.ctrlKey &&
      !event.altKey &&
      !event.shiftKey &&
      !event.metaKey
    ) {
      value = "";
      isFocused = false;
      inputElement?.blur();
      onSave();
      return;
    }

    // 收集修饰键
    const modifiers: string[] = [];
    if (event.ctrlKey || event.metaKey) {
      modifiers.push("CommandOrControl");
    }
    if (event.altKey) {
      modifiers.push("Alt");
    }
    if (event.shiftKey) {
      modifiers.push("Shift");
    }

    activeModifiers = modifiers;

    // 检查是否仅按下了修饰键
    const modifierKeys = ["Control", "Alt", "Shift", "Meta", "OS", "Super"];
    if (modifierKeys.includes(event.key)) {
      return;
    }

    // 处理常规按键
    let mainKey = event.key;
    if (mainKey === " ") {
      mainKey = "Space";
    } else if (mainKey.length === 1) {
      mainKey = mainKey.toUpperCase();
    }

    // 组合快捷键字符串
    const newShortcut =
      modifiers.length > 0 ? `${modifiers.join("+")}+${mainKey}` : mainKey;

    value = newShortcut;
    isFocused = false;
    activeModifiers = [];
    inputElement?.blur();
    onSave();
  }

  function handleKeyUp(event: KeyboardEvent) {
    if (!isFocused) return;

    const modifiers: string[] = [];
    if (event.ctrlKey || event.metaKey) {
      modifiers.push("CommandOrControl");
    }
    if (event.altKey) {
      modifiers.push("Alt");
    }
    if (event.shiftKey) {
      modifiers.push("Shift");
    }

    activeModifiers = modifiers;
  }

  function handleFocus() {
    if (disabled) return;
    isFocused = true;
    previousShortcut = value;
    activeModifiers = [];
  }

  function handleBlur() {
    if (!isFocused) return;
    isFocused = false;
    activeModifiers = [];
    if (!value && previousShortcut) {
      value = previousShortcut;
    }
  }

  function handleClear(e: MouseEvent) {
    e.stopPropagation();
    value = "";
    onSave();
  }

  function handleCancel(e: MouseEvent) {
    e.stopPropagation();
    value = previousShortcut;
    isFocused = false;
    activeModifiers = [];
    inputElement?.blur();
  }

  function setPresetShortcut(presetValue: string) {
    if (disabled) return;
    value = presetValue;
    onSave();
  }
</script>

<div class="flex flex-col gap-1.5">
  <!-- 输入触发容器 -->
  <!-- svelte-ignore a11y_click_events_have_key_events -->
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div
    class="group bg-background relative flex h-9 min-w-[170px] cursor-pointer items-center justify-between gap-2 rounded-xl border px-3 transition-[border-color,box-shadow,background-color] duration-140 ease-[cubic-bezier(0.23,1,0.32,1)]
      {isFocused
      ? 'border-primary ring-primary/20 shadow-xs ring-2'
      : 'border-input hover:border-border hover:bg-accent/30'}
      {disabled ? 'cursor-not-allowed opacity-50' : ''}"
    onclick={() => {
      if (!disabled) {
        inputElement?.focus();
      }
    }}
  >
    <!-- 隐藏但捕获焦点的真实 input -->
    <input
      bind:this={inputElement}
      type="text"
      class="sr-only"
      onfocus={handleFocus}
      onblur={handleBlur}
      onkeydown={handleKeyDown}
      onkeyup={handleKeyUp}
      {disabled}
      tabindex={disabled ? -1 : 0}
      aria-label="快捷键输入"
    />

    <!-- 键帽/状态显示区域 -->
    <div class="flex flex-1 flex-wrap items-center gap-1 overflow-hidden">
      {#if isFocused}
        <!-- 录入状态 -->
        {#if activeModifiers.length > 0}
          <!-- 正在按下修饰键 -->
          {#each activeModifiers as mod}
            <kbd
              class="border-primary/40 bg-primary/10 text-primary shadow-kbd inline-flex h-5.5 min-w-[24px] items-center justify-center rounded-md border px-1.5 font-mono text-[11px] font-semibold select-none"
            >
              {parseShortcutToKeys(mod)[0]?.display || mod}
            </kbd>
          {/each}
          <span class="text-muted-foreground/60 text-[10px] font-bold">+</span>
          <kbd
            class="border-primary/60 bg-primary/10 text-primary inline-flex h-5.5 min-w-[28px] animate-pulse items-center justify-center rounded-md border border-dashed px-1.5 font-mono text-[10px] font-medium select-none"
          >
            主键
          </kbd>
        {:else}
          <!-- 等待录入提示 -->
          <div
            class="text-primary flex items-center gap-1.5 text-xs font-medium"
          >
            <span class="relative flex h-2 w-2">
              <span
                class="bg-primary absolute inline-flex h-full w-full animate-ping rounded-full opacity-75"
              ></span>
              <span class="bg-primary relative inline-flex h-2 w-2 rounded-full"
              ></span>
            </span>
            <span>按下快捷键组合...</span>
          </div>
        {/if}
      {:else if currentKeyItems.length > 0}
        <!-- 常态展示：已设置快捷键键帽 -->
        {#each currentKeyItems as item, i}
          {#if i > 0}
            <span class="text-muted-foreground/50 text-[10px] font-bold">+</span
            >
          {/if}
          <kbd
            class="border-border/80 bg-muted/80 text-foreground shadow-kbd inline-flex h-5.5 min-w-[24px] items-center justify-center rounded-md border px-1.5 font-mono text-[11px] font-medium select-none"
          >
            {item.display}
          </kbd>
        {/each}
      {:else}
        <!-- 未设置快捷键的空状态 -->
        <div class="text-muted-foreground/70 flex items-center gap-1.5 text-xs">
          <Keyboard class="h-3.5 w-3.5" />
          <span>点击录入快捷键</span>
        </div>
      {/if}
    </div>

    <!-- 右侧操作 -->
    <div class="flex shrink-0 items-center gap-1">
      {#if isFocused}
        <Button
          variant="ghost"
          size="sm"
          class="h-6 px-1.5 text-[11px] transition-transform duration-120 active:scale-95"
          onclick={handleCancel}
        >
          取消
        </Button>
      {:else if value && !disabled}
        <Button
          variant="ghost"
          size="icon"
          class="text-muted-foreground/60 hover:text-foreground hover:bg-muted/80 h-5 w-5 opacity-0 transition-all duration-120 group-hover:opacity-100 active:scale-90"
          onclick={handleClear}
          title="清除快捷键"
        >
          <X class="h-3 w-3" />
        </Button>
      {/if}
    </div>
  </div>

  <!-- 预设快捷键按钮组 -->
  {#if showPresets}
    <div class="flex items-center gap-1.5 pt-0.5">
      {#each presetShortcuts as preset}
        <Button
          variant="outline"
          size="sm"
          class="h-6 rounded-lg px-2 text-[11px] shadow-2xs transition-[transform,background-color] duration-120 active:scale-95"
          onclick={() => setPresetShortcut(preset.value)}
          {disabled}
        >
          {preset.label}
        </Button>
      {/each}
    </div>
  {/if}
</div>
