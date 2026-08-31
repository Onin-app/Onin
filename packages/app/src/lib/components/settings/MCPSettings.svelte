<script lang="ts">
  import { Cpu } from "phosphor-svelte";
  import { Badge } from "$lib/components/ui/badge";
</script>

<div
  class="border-border/60 bg-card/40 flex h-72 flex-col items-center justify-center gap-3.5 rounded-2xl border border-dashed p-8 text-center shadow-2xs transition-[border-color,background-color] duration-140"
>
  <div
    class="border-border/50 bg-muted/60 text-muted-foreground flex h-14 w-14 items-center justify-center rounded-2xl border shadow-xs"
  >
    <Cpu size={28} weight="duotone" />
  </div>
  <div class="space-y-1">
    <h3 class="text-foreground text-sm font-semibold tracking-tight">
      Model Context Protocol (MCP)
    </h3>
    <p class="text-muted-foreground/75 max-w-sm text-xs leading-normal">
      支持连接本地或远程 MCP 服务器，为 AI 提供更丰富的工具与上下文能力。
    </p>
  </div>
  <Badge
    variant="secondary"
    class="border-border/40 mt-1 rounded-md border px-2 py-0.5 text-[11px] font-medium"
  >
    即将推出
  </Badge>
</div>
