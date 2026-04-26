<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { open } from "@tauri-apps/plugin-dialog";
  import { Switch } from "bits-ui";
  import { toast } from "svelte-sonner";
  import PhosphorIcon from "$lib/components/PhosphorIcon.svelte";
  import type { AppConfig } from "$lib/type";

  let config = $state<AppConfig | null>(null);
  let loading = $state(true);
  let saving = $state(false);
  let rebuilding = $state(false);
  let choosingDirectory = $state(false);
  let rootInput = $state("");
  let excludeInput = $state("");

  const roots = $derived(config?.file_search_roots ?? []);
  const excludedPaths = $derived(config?.file_search_excluded_paths ?? []);

  async function loadConfig() {
    loading = true;
    try {
      config = await invoke<AppConfig>("get_app_config");
    } catch (error) {
      console.error("Failed to load file search config:", error);
      toast.error("加载文件搜索设置失败");
    } finally {
      loading = false;
    }
  }

  async function saveConfig(nextConfig: AppConfig) {
    if (saving) return;

    saving = true;
    try {
      await invoke("update_app_config", { config: nextConfig });
      config = nextConfig;
      toast.success("文件搜索设置已保存");
    } catch (error) {
      console.error("Failed to save file search config:", error);
      toast.error("保存文件搜索设置失败");
    } finally {
      saving = false;
    }
  }

  function normalizedList(paths: string[]) {
    return Array.from(
      new Set(paths.map((path) => path.trim()).filter(Boolean)),
    );
  }

  async function addRoot(path: string) {
    if (!config) return;
    const nextRoots = normalizedList([...roots, path]);
    await saveConfig({ ...config, file_search_roots: nextRoots });
    rootInput = "";
  }

  async function addExcludedPath(path: string) {
    if (!config) return;
    const nextExcludedPaths = normalizedList([...excludedPaths, path]);
    await saveConfig({
      ...config,
      file_search_excluded_paths: nextExcludedPaths,
    });
    excludeInput = "";
  }

  async function removeRoot(path: string) {
    if (!config) return;
    await saveConfig({
      ...config,
      file_search_roots: roots.filter((root) => root !== path),
    });
  }

  async function removeExcludedPath(path: string) {
    if (!config) return;
    await saveConfig({
      ...config,
      file_search_excluded_paths: excludedPaths.filter(
        (excludedPath) => excludedPath !== path,
      ),
    });
  }

  async function chooseDirectory(target: "root" | "exclude") {
    if (choosingDirectory) return;

    choosingDirectory = true;
    await invoke("acquire_window_close_lock");
    try {
      const selected = await open({
        directory: true,
        multiple: false,
      });

      if (!selected || Array.isArray(selected)) {
        return;
      }

      if (target === "root") {
        await addRoot(selected);
      } else {
        await addExcludedPath(selected);
      }
    } catch (error) {
      console.error("Failed to choose file search directory:", error);
      toast.error("选择目录失败");
    } finally {
      choosingDirectory = false;
      await invoke("release_window_close_lock");
    }
  }

  async function updateIncludeHidden(includeHidden: boolean) {
    if (!config) return;
    await saveConfig({
      ...config,
      file_search_include_hidden: includeHidden,
    });
  }

  async function rebuildIndex() {
    rebuilding = true;
    try {
      await invoke("rebuild_file_search_index");
      toast.success("已开始重建文件搜索索引");
    } catch (error) {
      console.error("Failed to rebuild file search index:", error);
      toast.error(String(error));
    } finally {
      rebuilding = false;
    }
  }

  onMount(loadConfig);
</script>

{#if loading}
  <div class="py-3 text-sm text-neutral-500 dark:text-neutral-400">
    正在加载文件搜索设置...
  </div>
{:else if config}
  <div class="flex flex-col gap-5">
    <section class="border-t border-neutral-100 pt-4 dark:border-neutral-800">
      <div class="mb-3 flex items-center justify-between gap-4">
        <div>
          <h4
            class="text-sm font-semibold text-neutral-950 dark:text-neutral-50"
          >
            索引目录
          </h4>
          <p class="mt-1 text-xs text-neutral-500 dark:text-neutral-400">
            文件搜索只会索引这些目录及其子目录。
          </p>
        </div>
        <button
          class="inline-flex items-center gap-1.5 rounded-md border border-neutral-200 px-2.5 py-1.5 text-xs font-medium text-neutral-700 transition-colors hover:bg-neutral-100 dark:border-neutral-700 dark:text-neutral-200 dark:hover:bg-neutral-800"
          disabled={choosingDirectory}
          onclick={() => chooseDirectory("root")}
        >
          <PhosphorIcon icon="folderPlus" class="h-4 w-4" />
          选择目录
        </button>
      </div>

      <div class="mb-3 flex gap-2">
        <input
          class="min-w-0 flex-1 rounded-md border border-neutral-200 bg-white px-3 py-2 text-sm outline-none focus:border-neutral-400 dark:border-neutral-700 dark:bg-neutral-950 dark:focus:border-neutral-500"
          placeholder="/Users/name/Documents"
          bind:value={rootInput}
          onkeydown={(event) => {
            if (event.key === "Enter") addRoot(rootInput);
          }}
        />
        <button
          class="rounded-md bg-neutral-900 px-3 py-2 text-sm font-medium text-white disabled:cursor-not-allowed disabled:opacity-50 dark:bg-neutral-100 dark:text-neutral-900"
          disabled={!rootInput.trim()}
          onclick={() => addRoot(rootInput)}
        >
          添加
        </button>
      </div>

      <div class="flex flex-col gap-2">
        {#each roots as root (root)}
          <div
            class="flex items-center gap-2 rounded-md bg-neutral-100 px-3 py-2 text-sm dark:bg-neutral-800"
          >
            <PhosphorIcon icon="folder" class="h-4 w-4 shrink-0" />
            <span class="min-w-0 flex-1 truncate font-mono text-xs">
              {root}
            </span>
            <button
              class="rounded p-1 text-neutral-500 hover:bg-neutral-200 hover:text-neutral-900 dark:hover:bg-neutral-700 dark:hover:text-neutral-100"
              title="移除"
              onclick={() => removeRoot(root)}
            >
              <PhosphorIcon icon="trash" class="h-4 w-4" />
            </button>
          </div>
        {:else}
          <div class="text-sm text-neutral-500 dark:text-neutral-400">
            暂无索引目录
          </div>
        {/each}
      </div>
    </section>

    <section class="border-t border-neutral-100 pt-4 dark:border-neutral-800">
      <div class="mb-3 flex items-center justify-between gap-4">
        <div>
          <h4
            class="text-sm font-semibold text-neutral-950 dark:text-neutral-50"
          >
            排除路径
          </h4>
          <p class="mt-1 text-xs text-neutral-500 dark:text-neutral-400">
            匹配这些路径的文件和目录不会进入索引。
          </p>
        </div>
        <button
          class="inline-flex items-center gap-1.5 rounded-md border border-neutral-200 px-2.5 py-1.5 text-xs font-medium text-neutral-700 transition-colors hover:bg-neutral-100 dark:border-neutral-700 dark:text-neutral-200 dark:hover:bg-neutral-800"
          disabled={choosingDirectory}
          onclick={() => chooseDirectory("exclude")}
        >
          <PhosphorIcon icon="folderPlus" class="h-4 w-4" />
          选择目录
        </button>
      </div>

      <div class="mb-3 flex gap-2">
        <input
          class="min-w-0 flex-1 rounded-md border border-neutral-200 bg-white px-3 py-2 text-sm outline-none focus:border-neutral-400 dark:border-neutral-700 dark:bg-neutral-950 dark:focus:border-neutral-500"
          placeholder="/Users/name/Downloads/tmp"
          bind:value={excludeInput}
          onkeydown={(event) => {
            if (event.key === "Enter") addExcludedPath(excludeInput);
          }}
        />
        <button
          class="rounded-md bg-neutral-900 px-3 py-2 text-sm font-medium text-white disabled:cursor-not-allowed disabled:opacity-50 dark:bg-neutral-100 dark:text-neutral-900"
          disabled={!excludeInput.trim()}
          onclick={() => addExcludedPath(excludeInput)}
        >
          添加
        </button>
      </div>

      <div class="flex flex-col gap-2">
        {#each excludedPaths as path (path)}
          <div
            class="flex items-center gap-2 rounded-md bg-neutral-100 px-3 py-2 text-sm dark:bg-neutral-800"
          >
            <PhosphorIcon icon="prohibit" class="h-4 w-4 shrink-0" />
            <span class="min-w-0 flex-1 truncate font-mono text-xs">
              {path}
            </span>
            <button
              class="rounded p-1 text-neutral-500 hover:bg-neutral-200 hover:text-neutral-900 dark:hover:bg-neutral-700 dark:hover:text-neutral-100"
              title="移除"
              onclick={() => removeExcludedPath(path)}
            >
              <PhosphorIcon icon="trash" class="h-4 w-4" />
            </button>
          </div>
        {:else}
          <div class="text-sm text-neutral-500 dark:text-neutral-400">
            暂无额外排除路径
          </div>
        {/each}
      </div>
    </section>

    <section
      class="flex items-center justify-between gap-4 border-t border-neutral-100 pt-4 dark:border-neutral-800"
    >
      <div>
        <h4 class="text-sm font-semibold text-neutral-950 dark:text-neutral-50">
          隐藏文件
        </h4>
        <p class="mt-1 text-xs text-neutral-500 dark:text-neutral-400">
          开启后会索引以点号开头的文件和目录。
        </p>
      </div>
      <Switch.Root
        checked={config.file_search_include_hidden ?? false}
        onCheckedChange={updateIncludeHidden}
        class="peer inline-flex h-6 w-11 shrink-0 cursor-pointer items-center rounded-full border-2 border-transparent transition-colors focus-visible:ring-2 focus-visible:ring-neutral-950 focus-visible:ring-offset-2 focus-visible:outline-hidden data-[state=checked]:bg-neutral-900 data-[state=unchecked]:bg-neutral-200 dark:focus-visible:ring-neutral-300 dark:data-[state=checked]:bg-neutral-50 dark:data-[state=unchecked]:bg-neutral-700"
      >
        <Switch.Thumb
          class="pointer-events-none block h-5 w-5 rounded-full bg-white shadow-lg ring-0 transition-transform data-[state=checked]:translate-x-5 data-[state=unchecked]:translate-x-0 dark:bg-neutral-950"
        />
      </Switch.Root>
    </section>

    <section
      class="flex items-center justify-between gap-4 border-t border-neutral-100 pt-4 dark:border-neutral-800"
    >
      <div>
        <h4 class="text-sm font-semibold text-neutral-950 dark:text-neutral-50">
          索引维护
        </h4>
        <p class="mt-1 text-xs text-neutral-500 dark:text-neutral-400">
          调整目录或排除项后，可手动重建索引。
        </p>
      </div>
      <button
        class="inline-flex items-center gap-2 rounded-md bg-neutral-900 px-3 py-2 text-sm font-medium text-white transition-colors hover:bg-neutral-700 disabled:cursor-not-allowed disabled:opacity-60 dark:bg-neutral-100 dark:text-neutral-900 dark:hover:bg-neutral-300"
        disabled={rebuilding}
        onclick={rebuildIndex}
      >
        <PhosphorIcon icon="arrowsClockwise" class="h-4 w-4" />
        {rebuilding ? "重建中" : "重建索引"}
      </button>
    </section>
  </div>
{/if}
