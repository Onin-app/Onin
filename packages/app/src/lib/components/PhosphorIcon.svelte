<script lang="ts">
  import {
    Power,
    ArrowsClockwise,
    Moon,
    Lock,
    SignOut,
    Folder,
    Cube,
    Calculator,
    Ruler,
    CurrencyDollar,
    CalendarBlank,
    Smiley,
    Translate,
    GlobeSimple,
    Globe,
    LinkSimple,
    Link,
    Clipboard,
    File,
    FolderPlus,
    Trash,
    Prohibit,
    CaretDown,
  } from "phosphor-svelte";

  interface PhosphorIconProps {
    icon: string;
    size?: number | string;
    class?: string;
  }

  const { icon, size, class: cls }: PhosphorIconProps = $props();

  // 图标映射表
  const iconMap: Record<string, any> = {
    shutdown: Power,
    restart: ArrowsClockwise,
    arrowsClockwise: ArrowsClockwise,
    sleep: Moon,
    lock: Lock,
    logout: SignOut,
    folder: Folder,
    folderPlus: FolderPlus,
    "folder-plus": FolderPlus,
    cube: Cube,
    package: Cube,
    calculator: Calculator,
    ruler: Ruler,
    currencyDollar: CurrencyDollar,
    calendar: CalendarBlank,
    smiley: Smiley,
    translate: Translate,
    globe: Globe,
    globeSimple: GlobeSimple,
    link: Link,
    linkSimple: LinkSimple,
    clipboard: Clipboard,
    file: File,
    trash: Trash,
    prohibit: Prohibit,
    caretDown: CaretDown,
    "caret-down": CaretDown,
  };

  const IconComponent = $derived(iconMap[icon] || Power);
</script>

<IconComponent {size} class={cls} />
