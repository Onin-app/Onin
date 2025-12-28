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
    previousShortcut = value;
  };

  const handleBlur = () => {
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
  <input
    type="text"
    readonly
    value={formatShortcutForDisplay(value)}
    onkeydown={handleKeydown}
    onfocus={handleFocus}
    onblur={handleBlur}
    placeholder="点击设置快捷键"
    class="bg-background text-foreground w-40 rounded border p-1"
    {disabled}
  />
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
