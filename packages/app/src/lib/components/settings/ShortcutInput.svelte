<script lang="ts">
  import { Button } from "bits-ui";
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
        case "cmd":
        case "command":
          return {
            raw: trimmed,
            display: isMac ? "⌘" : "Win",
            isModifier: true,
          };
        case "win":
          return {
            raw: trimmed,
            display: "Win",
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
            display:
              trimmed.length === 1
                ? trimmed.toUpperCase()
                : trimmed.charAt(0).toUpperCase() + trimmed.slice(1),
            isModifier: false,
          };
      }
    });
  };

  // 解析当前已按下的修饰键
  const getActiveModifierKeys = (mods: string[]): KeyItem[] => {
    return mods.map((mod) => {
      switch (mod) {
        case "CommandOrControl":
          return {
            raw: mod,
            display: isMac ? "⌘" : "Ctrl",
            isModifier: true,
          };
        case "Control":
          return {
            raw: mod,
            display: isMac ? "⌃" : "Ctrl",
            isModifier: true,
          };
        case "Super":
          return {
            raw: mod,
            display: isMac ? "⌘" : "Win",
            isModifier: true,
          };
        case "Alt":
          return {
            raw: mod,
            display: isMac ? "⌥" : "Alt",
            isModifier: true,
          };
        case "Shift":
          return {
            raw: mod,
            display: isMac ? "⇧" : "Shift",
            isModifier: true,
          };
        default:
          return {
            raw: mod,
            display: mod,
            isModifier: true,
          };
      }
    });
  };

  const handleKeydown = (e: KeyboardEvent) => {
    if (disabled) return;
    e.preventDefault();
    e.stopPropagation();

    // 按 Backspace 或 Delete 且无修饰键时清空
    if (
      (e.key === "Backspace" || e.key === "Delete") &&
      !e.ctrlKey &&
      !e.altKey &&
      !e.shiftKey &&
      !e.metaKey
    ) {
      value = "";
      previousShortcut = "";
      activeModifiers = [];
      onSave();
      inputElement?.blur();
      return;
    }

    const parts: string[] = [];

    // 主修饰键
    if ((e.metaKey && isMac) || (e.ctrlKey && !isMac)) {
      parts.push("CommandOrControl");
    }

    // 额外的修饰键
    if (e.ctrlKey && isMac) {
      parts.push("Control");
    }
    if (e.metaKey && !isMac) {
      parts.push("Super");
    }
    if (e.altKey) parts.push("Alt");
    if (e.shiftKey) parts.push("Shift");

    const key = e.key;

    // 如果当前仅按下了修饰键，更新实时修饰键列表
    if (["Control", "Alt", "Shift", "Meta"].includes(key)) {
      activeModifiers = parts;
      return;
    }

    // 格式化主按键
    let finalKey = key;
    if (key === " ") {
      finalKey = "Space";
    } else if (key.length === 1 && /[a-zA-Z]/.test(key)) {
      finalKey = key.toUpperCase();
    } else if (key.startsWith("Arrow")) {
      finalKey = key.replace("Arrow", "");
    }

    parts.push(finalKey);
    value = parts.join("+");
    activeModifiers = [];
    inputElement?.blur();
  };

  const handleKeyup = (e: KeyboardEvent) => {
    if (!isFocused || disabled) return;
    const parts: string[] = [];
    if ((e.metaKey && isMac) || (e.ctrlKey && !isMac)) {
      parts.push("CommandOrControl");
    }
    if (e.ctrlKey && isMac) {
      parts.push("Control");
    }
    if (e.metaKey && !isMac) {
      parts.push("Super");
    }
    if (e.altKey) parts.push("Alt");
    if (e.shiftKey) parts.push("Shift");
    activeModifiers = parts;
  };

  const handleFocus = () => {
    if (disabled) return;
    isFocused = true;
    previousShortcut = value;
    activeModifiers = [];
    // 通知后端：正在录制快捷键，Windows 的 Alt+Space 等钩子放行
    invoke("set_shortcut_recording", { active: true }).catch(() => {});
  };

  const handleBlur = () => {
    isFocused = false;
    activeModifiers = [];
    invoke("set_shortcut_recording", { active: false }).catch(() => {});

    const modifiers = [
      "commandorcontrol",
      "control",
      "alt",
      "shift",
      "super",
      "command",
      "cmd",
    ];
    const parts = value ? value.split("+") : [];
    const lastPart = parts[parts.length - 1];

    // 如果快捷键不完整（只有修饰键），恢复原值
    if (parts.length > 0 && modifiers.includes(lastPart.toLowerCase())) {
      value = previousShortcut;
    }

    if (value !== previousShortcut) {
      onSave();
    }
  };

  const handleCancel = (e: MouseEvent) => {
    e.stopPropagation();
    value = previousShortcut;
    activeModifiers = [];
    inputElement?.blur();
  };

  const handleClear = (e: MouseEvent) => {
    e.stopPropagation();
    if (disabled) return;
    value = "";
    previousShortcut = "";
    activeModifiers = [];
    onSave();
  };

  const setPresetShortcut = (presetValue: string) => {
    if (disabled) return;
    value = presetValue;
    onSave();
  };

  const currentKeyItems = $derived(parseShortcutToKeys(value));
  const activeModifierItems = $derived(getActiveModifierKeys(activeModifiers));
</script>

<div class="flex flex-col gap-2">
  <!-- 快捷键输入容器 -->
  <div
    role="button"
    tabindex={disabled ? -1 : 0}
    onclick={() => !disabled && inputElement?.focus()}
    onkeydown={(e) => {
      if (e.key === "Enter" || e.key === " ") {
        !disabled && inputElement?.focus();
      }
    }}
    class="group relative flex min-h-8 w-full items-center justify-between gap-2 rounded-lg border px-2.5 py-1 transition-all select-none
      {disabled
      ? 'cursor-not-allowed border-neutral-200/80 bg-neutral-50/50 opacity-60 dark:border-neutral-800/80 dark:bg-neutral-900/50'
      : isFocused
        ? 'cursor-text border-blue-500 bg-blue-50/40 shadow-sm ring-2 ring-blue-500/20 dark:border-blue-400 dark:bg-blue-950/20 dark:ring-blue-400/20'
        : 'cursor-pointer border-neutral-200 bg-white hover:border-neutral-300 hover:bg-neutral-50/80 dark:border-neutral-800 dark:bg-neutral-900 dark:hover:border-neutral-700 dark:hover:bg-neutral-800/60'}"
  >
    <!-- 隐藏但保留焦点的真实输入框以捕获按键 -->
    <input
      bind:this={inputElement}
      type="text"
      readonly
      data-shortcut-recorder
      onkeydown={handleKeydown}
      onkeyup={handleKeyup}
      onfocus={handleFocus}
      onblur={handleBlur}
      {disabled}
      class="sr-only"
      aria-label="快捷键输入"
    />

    <!-- 左侧快捷键/状态内容展示 -->
    <div class="flex flex-1 items-center gap-1 overflow-x-auto py-0.5">
      {#if isFocused}
        <!-- 录入中状态 -->
        {#if activeModifierItems.length > 0}
          <!-- 实时按下的修饰键预览 -->
          {#each activeModifierItems as item, i}
            {#if i > 0}
              <span
                class="text-[10px] font-bold text-neutral-400 dark:text-neutral-500"
                >+</span
              >
            {/if}
            <kbd
              class="animate-in fade-in zoom-in-95 inline-flex h-5.5 min-w-[22px] items-center justify-center rounded border border-blue-300 bg-blue-100/80 px-1.5 font-mono text-[11px] font-semibold text-blue-600 shadow-[0_1px_0_0_rgba(59,130,246,0.2)] dark:border-blue-500/40 dark:bg-blue-500/20 dark:text-blue-300 dark:shadow-[0_1px_0_0_rgba(0,0,0,0.3)]"
            >
              {item.display}
            </kbd>
          {/each}
          <span
            class="text-[10px] font-bold text-neutral-400 dark:text-neutral-500"
            >+</span
          >
          <kbd
            class="inline-flex h-5.5 min-w-[24px] animate-pulse items-center justify-center rounded border border-dashed border-blue-400/80 bg-blue-500/10 px-1.5 font-mono text-[10px] font-medium text-blue-500 select-none dark:border-blue-400/60 dark:text-blue-300"
          >
            主键
          </kbd>
        {:else}
          <!-- 等待录入提示 -->
          <div
            class="flex items-center gap-1.5 text-xs font-medium text-blue-600 dark:text-blue-400"
          >
            <span class="relative flex h-2 w-2">
              <span
                class="absolute inline-flex h-full w-full animate-ping rounded-full bg-blue-400 opacity-75"
              ></span>
              <span
                class="relative inline-flex h-2 w-2 rounded-full bg-blue-500 dark:bg-blue-400"
              ></span>
            </span>
            <span>按下快捷键组合...</span>
          </div>
        {/if}
      {:else if currentKeyItems.length > 0}
        <!-- 常态展示：已设置快捷键键帽 -->
        {#each currentKeyItems as item, i}
          {#if i > 0}
            <span
              class="text-[10px] font-bold text-neutral-300 dark:text-neutral-600"
              >+</span
            >
          {/if}
          <kbd
            class="inline-flex h-5.5 min-w-[22px] items-center justify-center rounded border border-neutral-200/90 bg-neutral-100 px-1.5 font-mono text-[11px] font-semibold text-neutral-800 shadow-[0_1.5px_0_0_rgba(0,0,0,0.06)] dark:border-neutral-700/80 dark:bg-neutral-800 dark:text-neutral-200 dark:shadow-[0_1.5px_0_0_rgba(255,255,255,0.05)]"
          >
            {item.display}
          </kbd>
        {/each}
      {:else}
        <!-- 未设置快捷键的空状态 -->
        <div
          class="flex items-center gap-1.5 text-xs text-neutral-400 dark:text-neutral-500"
        >
          <Keyboard class="h-3.5 w-3.5" />
          <span>点击录入快捷键</span>
        </div>
      {/if}
    </div>

    <!-- 右侧状态徽章 / 清除操作 -->
    <div class="flex shrink-0 items-center gap-1">
      {#if isFocused}
        <!-- 录入态取消按钮 -->
        <Button.Root
          class="rounded px-1.5 py-0.5 text-[11px] font-medium text-neutral-500 transition-colors hover:bg-neutral-200/80 hover:text-neutral-800 dark:text-neutral-400 dark:hover:bg-neutral-800 dark:hover:text-neutral-200"
          onclick={handleCancel}
        >
          取消
        </Button.Root>
      {:else if value && !disabled}
        <!-- 常态下 Hover 出现的清除按钮 -->
        <Button.Root
          class="flex h-5 w-5 items-center justify-center rounded text-neutral-400 opacity-0 transition-all group-hover:opacity-100 hover:bg-neutral-200 hover:text-neutral-700 dark:hover:bg-neutral-800 dark:hover:text-neutral-200"
          onclick={handleClear}
          title="清除快捷键"
        >
          <X class="h-3 w-3" />
        </Button.Root>
      {/if}
    </div>
  </div>

  <!-- 预设快捷键按钮组 -->
  {#if showPresets}
    <div class="flex items-center gap-1.5 pt-0.5">
      {#each presetShortcuts as preset}
        <Button.Root
          class="inline-flex items-center justify-center rounded-md border border-neutral-200 bg-white px-2 py-0.5 text-[11px] font-medium text-neutral-700 shadow-xs transition-colors hover:bg-neutral-100 hover:text-neutral-900 active:scale-[0.98] disabled:opacity-50 dark:border-neutral-800 dark:bg-neutral-900 dark:text-neutral-300 dark:hover:bg-neutral-800 dark:hover:text-neutral-100"
          onclick={() => setPresetShortcut(preset.value)}
          {disabled}
        >
          {preset.label}
        </Button.Root>
      {/each}
    </div>
  {/if}
</div>
