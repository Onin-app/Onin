<script lang="ts">
  import { Button } from "bits-ui";
  import { platform } from "@tauri-apps/plugin-os";

  let {
    value = $bindable(""),
    onSave = () => {},
    disabled = false,
    showPresets = false,
  } = $props();
  let previousShortcut = "";
  let isFocused = $state(false);

  // 立即初始化平台信息，不要等到 onMount
  let currentPlatform = "";
  try {
    if (typeof window !== "undefined" && (window as any).__TAURI__) {
      currentPlatform = platform();
    } else {
      // 在浏览器环境中，使用 navigator 来检测平台
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
    // 使用 navigator.platform 作为后备方案
    const navPlatform = navigator.platform.toLowerCase();
    if (navPlatform.includes("mac")) {
      currentPlatform = "macos";
    } else if (navPlatform.includes("win")) {
      currentPlatform = "windows";
    } else {
      // Linux 或其他类 Unix 系统
      currentPlatform = "linux";
    }
  }

  // 根据平台获取预设快捷键选项
  const getPresetShortcuts = (platformName: string) => {
    const isMac = platformName === "macos";
    return [
      {
        label: isMac ? "⌥+Space" : "Alt+Space",
        value: "Alt+Space",
      },
      {
        label: isMac ? "⌘+Space" : "Ctrl+Space",
        value: "CommandOrControl+Space",
      },
    ];
  };

  // 将快捷键转换为用户友好的显示格式（输入框用文字）
  const formatShortcutForDisplay = (shortcut: string) => {
    if (!shortcut) return "";

    const isMac = currentPlatform === "macos";
    const parts = shortcut.split("+");

    const displayParts = parts.map((part) => {
      const trimmedPart = part.trim();
      const lowerPart = trimmedPart.toLowerCase();
      switch (lowerPart) {
        case "commandorcontrol":
          return isMac ? "Command" : "Ctrl";
        case "ctrl":
        case "control":
          return "Control";
        case "alt":
        case "option":
          return isMac ? "Option" : "Alt";
        case "shift":
          return "Shift";
        case "super":
        case "cmd":
        case "command":
          return "Command";
        case "win":
          return "Win";
        case "space":
          return "Space";
        default:
          // 保持首字母大写
          return (
            trimmedPart.charAt(0).toUpperCase() +
            trimmedPart.slice(1).toLowerCase()
          );
      }
    });

    return displayParts.join("+");
  };

  let presetShortcuts = $state<{ label: string; value: string }[]>(
    getPresetShortcuts(currentPlatform),
  );

  const handleKeydown = (e: KeyboardEvent) => {
    e.preventDefault();
    e.stopPropagation();

    const parts: string[] = [];
    const isMac = currentPlatform === "macos";

    // 处理修饰键的逻辑：
    // 1. 在 macOS 上，Cmd 键映射为 CommandOrControl
    // 2. 在其他平台上，Ctrl 键映射为 CommandOrControl
    // 3. 如果同时按下了其他修饰键（如 macOS 上的 Ctrl），也要记录

    // 主修饰键（跨平台）
    if ((e.metaKey && isMac) || (e.ctrlKey && !isMac)) {
      parts.push("CommandOrControl");
    }

    // 额外的修饰键
    // 在 macOS 上，如果按下了 Ctrl，单独记录（即使同时按下了 Cmd）
    if (e.ctrlKey && isMac) {
      parts.push("Control");
    }
    // 在非 macOS 上，如果按下了 Meta/Super 键，单独记录（即使同时按下了 Ctrl）
    if (e.metaKey && !isMac) {
      parts.push("Super");
    }

    if (e.altKey) parts.push("Alt");
    if (e.shiftKey) parts.push("Shift");

    const key = e.key;

    if (["Control", "Alt", "Shift", "Meta"].includes(key)) {
      value = parts.join("+");
      return;
    }

    let finalKey = key;
    if (key === " ") {
      finalKey = "Space";
    } else if (key.length === 1 && /[a-zA-Z]/.test(key)) {
      finalKey = key.toUpperCase();
    }

    parts.push(finalKey);
    value = parts.join("+");
  };

  const handleFocus = () => {
    isFocused = true;
    previousShortcut = value;
  };

  const handleBlur = () => {
    isFocused = false;
    const modifiers = [
      "commandorcontrol",
      "control",
      "alt",
      "shift",
      "super",
      "command",
      "cmd",
    ];
    const parts = value.split("+");
    const lastPart = parts[parts.length - 1];

    if (
      !value ||
      (parts.length > 0 && modifiers.includes(lastPart.toLowerCase()))
    ) {
      value = previousShortcut;
    }
    onSave();
  };

  const setPresetShortcut = (shortcut: string) => {
    value = shortcut;
    onSave();
  };
</script>

<div class="flex flex-col gap-2">
  <div class="relative w-full">
    <input
      type="text"
      readonly
      value={formatShortcutForDisplay(value)}
      onkeydown={handleKeydown}
      onfocus={handleFocus}
      onblur={handleBlur}
      placeholder={isFocused ? "请按下组合键..." : "点击设置"}
      class="h-8 w-full rounded-md border bg-transparent px-2 text-sm font-medium transition-all placeholder:text-xs placeholder:text-neutral-400 focus:outline-none {isFocused
        ? 'border-neutral-800 ring-4 ring-neutral-900/5 placeholder:text-neutral-500 dark:border-neutral-200 dark:ring-neutral-100/10'
        : 'border-neutral-200 hover:border-neutral-300 dark:border-neutral-800 dark:hover:border-neutral-700'} text-neutral-900 dark:text-neutral-100 dark:placeholder:text-neutral-600"
      {disabled}
    />
    {#if isFocused}
      <div
        class="pointer-events-none absolute top-1/2 right-2 -translate-y-1/2"
      >
        <span class="flex h-1.5 w-1.5">
          <span
            class="absolute inline-flex h-full w-full animate-ping rounded-full bg-neutral-400 opacity-75 dark:bg-neutral-500"
          ></span>
          <span
            class="relative inline-flex h-1.5 w-1.5 rounded-full bg-neutral-900 dark:bg-neutral-100"
          ></span>
        </span>
      </div>
    {/if}
  </div>
  {#if showPresets}
    <div class="flex gap-2">
      {#each presetShortcuts as preset}
        <Button.Root
          class="rounded-input bg-dark text-background shadow-mini hover:bg-dark/95 inline-flex
	                items-center justify-center px-2
	                text-[12px]  active:scale-[0.98] active:transition-all"
          onclick={() => setPresetShortcut(preset.value)}
          {disabled}
        >
          {preset.label}
        </Button.Root>
      {/each}
    </div>
  {/if}
</div>
