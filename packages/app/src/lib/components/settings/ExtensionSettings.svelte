<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { Switch } from "bits-ui";
  import { toast } from "svelte-sonner";
  import AppScrollArea from "$lib/components/AppScrollArea.svelte";
  import PhosphorIcon from "$lib/components/PhosphorIcon.svelte";

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
      toast.success(`${extension.name} ${enabled ? "已启用" : "已禁用"}`);
      extensions = await invoke<ExtensionInfo[]>("get_extensions");
    } catch (error) {
      console.error("Failed to toggle extension:", error);
      extension.enabled = previous;
      extensions = [...extensions];
      toast.error("更新扩展状态失败");
    } finally {
      savingId = null;
    }
  }

  onMount(loadExtensions);
</script>

<AppScrollArea class="h-full w-full" viewportClass="h-full w-full">
  <main class="h-full w-full pr-2 pb-8">
    <header class="mb-6 flex items-start justify-between gap-4">
      <div>
        <h2 class="text-xl font-bold text-neutral-950 dark:text-neutral-50">
          扩展
        </h2>
        <p class="mt-1 text-sm text-neutral-500 dark:text-neutral-400">
          管理 Onin 内置功能模块，禁用后将从搜索和匹配指令中移除。
        </p>
      </div>
      <div
        class="rounded-lg border border-neutral-200 px-3 py-2 text-right dark:border-neutral-800"
      >
        <div class="text-xs text-neutral-500 dark:text-neutral-400">已启用</div>
        <div
          class="text-lg font-semibold text-neutral-950 dark:text-neutral-50"
        >
          {enabledCount}/{extensions.length}
        </div>
      </div>
    </header>

    {#if loading}
      <div
        class="rounded-xl border border-neutral-200 bg-white p-6 text-sm text-neutral-500 dark:border-neutral-800 dark:bg-neutral-900 dark:text-neutral-400"
      >
        正在加载扩展...
      </div>
    {:else if extensions.length === 0}
      <div
        class="rounded-xl border border-neutral-200 bg-white p-6 text-sm text-neutral-500 dark:border-neutral-800 dark:bg-neutral-900 dark:text-neutral-400"
      >
        暂无可管理的扩展
      </div>
    {:else}
      <section class="flex flex-col gap-3">
        {#each extensions as extension (extension.id)}
          <article
            class="rounded-xl border border-neutral-200 bg-white p-4 transition-colors dark:border-neutral-800 dark:bg-neutral-900"
          >
            <div class="flex items-start gap-4">
              <div
                class="flex h-10 w-10 shrink-0 items-center justify-center rounded-lg bg-neutral-100 text-neutral-700 dark:bg-neutral-800 dark:text-neutral-200"
              >
                <PhosphorIcon icon={extension.icon} class="h-6 w-6" />
              </div>

              <div class="min-w-0 flex-1">
                <div class="flex items-center gap-2">
                  <h3
                    class="truncate text-sm font-semibold text-neutral-950 dark:text-neutral-50"
                  >
                    {extension.name}
                  </h3>
                  <span
                    class="rounded-md px-1.5 py-0.5 text-xs {extension.enabled
                      ? 'bg-emerald-50 text-emerald-700 dark:bg-emerald-950/40 dark:text-emerald-300'
                      : 'bg-neutral-100 text-neutral-500 dark:bg-neutral-800 dark:text-neutral-400'}"
                  >
                    {extension.enabled ? "启用中" : "已禁用"}
                  </span>
                </div>
                <p class="mt-1 text-sm text-neutral-500 dark:text-neutral-400">
                  {extension.description}
                </p>
              </div>

              <Switch.Root
                checked={extension.enabled}
                disabled={savingId === extension.id}
                onCheckedChange={(enabled) =>
                  toggleExtension(extension, enabled)}
                class="peer inline-flex h-6 w-11 shrink-0 cursor-pointer items-center rounded-full border-2 border-transparent transition-colors focus-visible:ring-2 focus-visible:ring-neutral-950 focus-visible:ring-offset-2 focus-visible:outline-hidden disabled:cursor-not-allowed disabled:opacity-50 data-[state=checked]:bg-neutral-900 data-[state=unchecked]:bg-neutral-200 dark:focus-visible:ring-neutral-300 dark:data-[state=checked]:bg-neutral-50 dark:data-[state=unchecked]:bg-neutral-700"
              >
                <Switch.Thumb
                  class="pointer-events-none block h-5 w-5 rounded-full bg-white shadow-lg ring-0 transition-transform data-[state=checked]:translate-x-5 data-[state=unchecked]:translate-x-0 dark:bg-neutral-950"
                />
              </Switch.Root>
            </div>

            <div
              class="mt-4 border-t border-neutral-100 pt-3 dark:border-neutral-800"
            >
              <div class="mb-2 text-xs font-medium text-neutral-400">指令</div>
              <div class="flex flex-wrap gap-2">
                {#each extension.commands as command (command.code)}
                  <span
                    class="inline-flex max-w-full items-center gap-1.5 rounded-md bg-neutral-100 px-2 py-1 text-xs text-neutral-700 dark:bg-neutral-800 dark:text-neutral-300"
                  >
                    <PhosphorIcon icon={command.icon} class="h-3.5 w-3.5" />
                    <span class="truncate">{command.name}</span>
                    {#if command.has_matches}
                      <span class="text-neutral-400">匹配</span>
                    {/if}
                  </span>
                {/each}
              </div>
            </div>
          </article>
        {/each}
      </section>
    {/if}
  </main>
</AppScrollArea>
