<script lang="ts">
  import { Button } from "bits-ui";

  let {
    value = $bindable(""),
    onSave = () => {},
    disabled = false,
    showPresets = false,
  } = $props();
  let previousShortcut = "";

  // 预设的快捷键选项（用户无法通过键盘录入的）
  const presetShortcuts = [
    { label: "Alt+Space", value: "Alt+Space" },
    { label: "Ctrl+Space", value: "Ctrl+Space" },
  ];

  const handleKeydown = (e: KeyboardEvent) => {
    e.preventDefault();
    e.stopPropagation();

    const parts: string[] = [];
    if (e.ctrlKey) parts.push("Ctrl");
    if (e.altKey) parts.push("Alt");
    if (e.shiftKey) parts.push("Shift");
    if (e.metaKey) parts.push("Super");

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
    const modifiers = ["Ctrl", "Alt", "Shift", "Super"];
    const parts = value.split("+");
    const lastPart = parts[parts.length - 1];

    if (!value || (parts.length > 0 && modifiers.includes(lastPart))) {
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
    bind:value
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
