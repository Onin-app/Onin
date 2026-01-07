<script lang="ts">
  /**
   * EmptyPluginState Component
   *
   * 插件列表为空时的状态显示
   */
  import { Package } from "phosphor-svelte";
</script>

<div class="flex h-full flex-col items-center justify-center text-neutral-500">
  <Package class="mb-4 h-12 w-12 opacity-50" />
  <p class="text-lg">没有找到插件</p>
  <p class="mt-2 text-sm">尝试导入插件或调整搜索条件</p>
</div>
