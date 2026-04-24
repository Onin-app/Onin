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
  } from "phosphor-svelte";

  interface PhosphorIconProps {
    icon: string;
    class?: string;
  }

  const { icon, class: cls }: PhosphorIconProps = $props();

  // 图标映射表
  const iconMap: Record<string, any> = {
    shutdown: Power,
    restart: ArrowsClockwise,
    arrowsClockwise: ArrowsClockwise,
    sleep: Moon,
    lock: Lock,
    logout: SignOut,
    folder: Folder,
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
  };

  const IconComponent = $derived(iconMap[icon] || Power);
</script>

{@render IconComponent({ class: cls })}
