<script lang="ts">
  import { onMount } from "svelte";
  import "../../index.css";

  onMount(() => {
    // 强制清除 html/body 背景色污染，确保指示框内部完全穿透透明，显示桌面原本的画面
    document.documentElement.style.backgroundColor = "transparent";
    document.body.style.backgroundColor = "transparent";
  });
</script>

<!-- 该页面作为录制区域标识边框。
     四边绘制红色精致细边，整体鼠标穿透不遮挡用户日常工作流 -->
<div
  class="pointer-events-none box-border h-screen w-screen border-2 border-red-500 bg-transparent"
></div>

<style>
  :global(html),
  :global(body) {
    margin: 0;
    padding: 0;
    overflow: hidden;
    background: transparent !important;
    pointer-events: none !important;
  }
</style>
