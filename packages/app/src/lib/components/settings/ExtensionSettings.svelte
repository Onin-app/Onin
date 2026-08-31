<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { Switch } from "$lib/components/ui/switch";
  import { Badge } from "$lib/components/ui/badge";
  import { Button } from "$lib/components/ui/button";
  import { Card } from "$lib/components/ui/card";
  import { ScrollArea } from "$lib/components/ui/scroll-area";
  import { toast } from "svelte-sonner";
  import PhosphorIcon from "$lib/components/PhosphorIcon.svelte";
  import ExtensionSettingsDrawer from "$lib/components/ExtensionSettingsDrawer.svelte";

  interface ExtensionCommandInfo {
    code: string;
    name: string;
    description?: string;
    icon: string;
    keywords: string[];
    has_matches: boolean;
  }

  interface ExtensionInfo {
    id: string;
    name: string;
    description: string;
    icon: string;
    enabled: boolean;
    commands: ExtensionCommandInfo[];
  }

  let extensions = $state<ExtensionInfo[]>([]);
  let loading = $state(true);
  let savingId = $state<string | null>(null);

  const enabledCount = $derived(
    extensions.filter((extension) => extension.enabled).length,
  );

  async function loadExtensions() {
    loading = true;
    try {
      extensions = await invoke<ExtensionInfo[]>("get_extensions");
    } catch (error) {
      console.error("Failed to load extensions:", error);
      toast.error("加载扩展失败");
    } finally {
      loading = false;
    }
  }

  async function toggleExtension(extension: ExtensionInfo, enabled: boolean) {
    const previous = extension.enabled;
    extension.enabled = enabled;
    extensions = [...extensions];
    savingId = extension.id;

    try {
      await invoke("toggle_extension", {
        extensionId: extension.id,
        enabled,
      });
      toast.success(`${extension.name}已${enabled ? "启用" : "禁用"}`);
    } catch (error) {
      console.error("Failed to toggle extension:", error);
      extension.enabled = previous;
      extensions = [...extensions];
      toast.error(`更新扩展状态失败`);
    } finally {
      savingId = null;
    }
  }

  let settingsDrawerOpen = $state(false);
  let activeExtensionId = $state<string | null>(null);
  let activeExtensionName = $state("");

  function openExtensionSettings(id: string, name: string) {
    activeExtensionId = id;
    activeExtensionName = name;
    settingsDrawerOpen = true;
  }

  onMount(loadExtensions);
</script>

<ScrollArea class="h-full w-full" viewportClass="h-full w-full">
  <main class="h-full w-full pr-2 pb-8">
    <header class="mb-6 flex items-center justify-between px-1">
      <div>
        <h2 class="text-foreground text-sm font-semibold">内置扩展</h2>
        <p class="text-muted-foreground text-xs">
          管理随应用内置的基础能力与专属设置
        </p>
      </div>
      <div class="text-right">
        <div class="text-muted-foreground text-xs">已启用</div>
        <div class="text-foreground text-lg font-semibold">
          {enabledCount}/{extensions.length}
        </div>
      </div>
    </header>

    {#if loading}
      <Card
        class="border-border/60 text-muted-foreground rounded-2xl p-6 text-sm"
        >正在加载扩展...</Card
      >
    {:else if extensions.length === 0}
      <Card
        class="border-border/60 text-muted-foreground rounded-2xl p-6 text-sm"
        >暂无可管理的扩展</Card
      >
    {:else}
      <section class="flex flex-col gap-3">
        {#each extensions as extension (extension.id)}
          <Card
            class="border-border/60 bg-card hover:border-border rounded-2xl p-4 shadow-2xs transition-[border-color,box-shadow] duration-140 ease-[cubic-bezier(0.23,1,0.32,1)]"
          >
            <div class="flex items-start gap-4">
              <div
                class="bg-muted/60 text-foreground border-border/50 flex h-10 w-10 shrink-0 items-center justify-center rounded-xl border shadow-xs"
              >
                <PhosphorIcon icon={extension.icon} class="h-5.5 w-5.5" />
              </div>

              <div class="min-w-0 flex-1">
                <div class="flex items-center gap-2">
                  <h3
                    class="text-foreground truncate text-sm font-semibold tracking-tight"
                  >
                    {extension.name}
                  </h3>
                  <Badge
                    variant={extension.enabled ? "secondary" : "outline"}
                    class="rounded-md px-1.5 py-0 text-[10px] font-normal"
                  >
                    {extension.enabled ? "启用中" : "已禁用"}
                  </Badge>
                </div>
                <p class="text-muted-foreground/75 mt-1 text-xs leading-normal">
                  {extension.description}
                </p>
              </div>

              <div class="flex items-center gap-2.5">
                {#if extension.id === "file_search" || extension.id === "ocr"}
                  <Button
                    variant="ghost"
                    size="icon"
                    class="text-muted-foreground hover:text-foreground h-8 w-8 cursor-pointer rounded-lg transition-transform duration-120 active:scale-90"
                    onclick={() =>
                      openExtensionSettings(extension.id, extension.name)}
                    title="设置"
                    aria-label="设置"
                  >
                    <PhosphorIcon icon="gear" class="h-4.5 w-4.5" />
                  </Button>
                {/if}

                <Switch
                  checked={extension.enabled}
                  disabled={savingId === extension.id}
                  onCheckedChange={(enabled) =>
                    toggleExtension(extension, enabled)}
                />
              </div>
            </div>

            <div class="border-border/40 mt-4 border-t pt-3">
              <div
                class="text-muted-foreground/70 mb-2 text-[11px] font-medium tracking-wider uppercase"
              >
                指令
              </div>
              <div class="flex flex-wrap gap-1.5">
                {#each extension.commands as command (command.code)}
                  <span
                    class="bg-muted/60 text-foreground border-border/40 inline-flex max-w-full items-center gap-1.5 rounded-lg border px-2.5 py-1 text-xs shadow-2xs"
                  >
                    <PhosphorIcon icon={command.icon} class="h-3.5 w-3.5" />
                    <span class="truncate">{command.name}</span>
                    {#if command.has_matches}
                      <span class="text-muted-foreground/60 text-[10px]"
                        >匹配</span
                      >
                    {/if}
                  </span>
                {/each}
              </div>
            </div>
          </Card>
        {/each}
      </section>
    {/if}
  </main>
</ScrollArea>

<ExtensionSettingsDrawer
  bind:open={settingsDrawerOpen}
  extensionId={activeExtensionId}
  extensionName={activeExtensionName}
/>
