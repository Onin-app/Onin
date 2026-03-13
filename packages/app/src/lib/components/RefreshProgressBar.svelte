<script lang="ts">
  /**
   * RefreshProgressBar Component
   *
   * 刷新进度条组件
   * 显示应用列表刷新时的进度指示器
   */
  interface Props {
    isRefreshing: boolean;
  }

  let { isRefreshing }: Props = $props();
</script>

{#if isRefreshing}
  <div
    class="absolute top-0 left-0 z-10 h-0.5 w-full overflow-hidden bg-neutral-200 dark:bg-neutral-700"
  >
    <div
      class="refresh-progress-bar h-full w-1/3 bg-gradient-to-r from-blue-400 via-blue-500 to-blue-400"
    ></div>
  </div>
{/if}

<style>
  .refresh-progress-bar {
    animation: progress-slide 1.5s ease-in-out infinite;
  }

  @keyframes progress-slide {
    0% {
      transform: translateX(-100%);
    }
    100% {
      transform: translateX(400%);
    }
  }
</style>
