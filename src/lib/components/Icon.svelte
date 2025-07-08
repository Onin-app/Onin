<script lang="ts">
  interface IconProps {
    icon: string;
    size?: number | string;
    class?: string;
    style?: string;
  }

  const { icon, size, class: cls, style }: IconProps = $props();

  const currentSize = typeof size === "number" ? `${size}px` : size;
</script>

<svg
  class="icon {cls}"
  style="width: {currentSize}; height: {currentSize}; {style}"
  aria-hidden="true"
>
  <use xlink:href="#icon-{icon}"></use>
</svg>

<style type="text/css">
  .icon {
    width: 1em;
    height: 1em;
    vertical-align: -0.15em;
    fill: currentColor;
    overflow: hidden;
  }
</style>
