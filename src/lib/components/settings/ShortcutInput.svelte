<script lang="ts">
  let {
    value = $bindable(""),
    onSave = () => {},
    disabled = false,
  } = $props();
  let previousShortcut = "";

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
</script>

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