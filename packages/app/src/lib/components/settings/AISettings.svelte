<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { Button, Combobox, ScrollArea } from "bits-ui";
  import { toast } from "svelte-sonner";
  import {
    Check,
    CaretUpDown,
    CaretDoubleUp,
    CaretDoubleDown,
    Plus,
    Trash,
    PencilSimple,
  } from "phosphor-svelte";

  // Mock data import - in production this would be a fetch call
  import remoteProviders from "$lib/mocks/ai-providers.json";

  interface AIConfig {
    active_provider_id: string | null;
    providers: ProviderConfig[];
  }

  interface ProviderConfig {
    id: string;
    provider_type: string;
    name: string;
    base_url: string;
    api_key: string | null;
    default_model: string | null;
  }

  interface RemoteProvider {
    id: string;
    name: string;
    description: string;
    baseUrl: string;
    models: { id: string; name: string }[];
    docsUrl?: string;
  }

  let config = $state<AIConfig>({ active_provider_id: null, providers: [] });
  let providers = $state<RemoteProvider[]>(remoteProviders as any);

  // Editing state
  let editingIndex = $state<number | null>(null); // null = not editing, -1 = adding new
  let editForm = $state<{
    id: string;
    provider_type: string;
    name: string;
    base_url: string;
    api_key: string | null;
    default_model: string | null;
  }>({
    id: "",
    provider_type: "",
    name: "",
    base_url: "",
    api_key: null,
    default_model: null,
  });

  // Search states for comboboxes
  let providerSearch = $state("");
  let modelSearch = $state("");

  // Computed
  let selectedRemoteProvider = $derived(
    providers.find((p) => p.id === editForm.provider_type),
  );
  let modelOptions = $derived(
    selectedRemoteProvider?.models?.map((m) => ({
      value: m.id,
      label: m.name,
    })) || [],
  );
  let filteredModelOptions = $derived(
    modelSearch === ""
      ? modelOptions
      : modelOptions.filter((m) =>
          m.label.toLowerCase().includes(modelSearch.toLowerCase()),
        ),
  );
  let providerOptions = $derived(
    providers.map((p) => ({ value: p.id, label: p.name })),
  );
  let filteredProviderOptions = $derived(
    providerSearch === ""
      ? providerOptions
      : providerOptions.filter((p) =>
          p.label.toLowerCase().includes(providerSearch.toLowerCase()),
        ),
  );

  onMount(async () => {
    try {
      config = await invoke("get_ai_config");
    } catch (e) {
      console.error("Failed to load AI config", e);
      toast.error("Failed to load AI config");
    }
  });

  function startAdd() {
    editingIndex = -1;
    editForm = {
      id: "",
      provider_type: "",
      name: "",
      base_url: "",
      api_key: null,
      default_model: null,
    };
  }

  function startEdit(index: number) {
    editingIndex = index;
    const provider = config.providers[index];
    editForm = { ...provider };
  }

  function cancelEdit() {
    editingIndex = null;
    providerSearch = "";
    modelSearch = "";
  }

  // Generate unique ID for provider instance
  function generateProviderId(templateId: string): string {
    const timestamp = Date.now();
    const random = Math.random().toString(36).substring(2, 8);
    return `${templateId}_${timestamp}_${random}`;
  }

  async function save() {
    // Validation
    if (!editForm.provider_type || !editForm.base_url) {
      toast.error("Provider and Base URL are required");
      return;
    }

    const remote = providers.find((p) => p.id === editForm.provider_type);

    // Generate unique ID for new providers, keep existing ID for edits
    const providerId =
      editingIndex === -1
        ? generateProviderId(editForm.provider_type)
        : config.providers[editingIndex!].id;

    const newProvider: ProviderConfig = {
      id: providerId,
      provider_type: editForm.provider_type,
      name: remote?.name || editForm.name,
      base_url: editForm.base_url,
      api_key: editForm.api_key || null,
      default_model: editForm.default_model || null,
    };

    if (editingIndex === -1) {
      // Adding new
      config.providers.push(newProvider);
    } else if (editingIndex !== null) {
      // Updating existing
      config.providers[editingIndex] = newProvider;
    }

    try {
      await invoke("update_ai_config", { config });
      toast.success("Provider saved");
      editingIndex = null;
      providerSearch = "";
      modelSearch = "";
    } catch (e) {
      console.error(e);
      toast.error("Error saving provider");
    }
  }

  async function deleteProvider(index: number) {
    const provider = config.providers[index];

    // Warn if deleting active provider
    if (provider.id === config.active_provider_id) {
      if (
        !confirm(
          "This is your active provider. Are you sure you want to delete it?",
        )
      ) {
        return;
      }
      config.active_provider_id = null;
    }

    config.providers.splice(index, 1);

    try {
      await invoke("update_ai_config", { config });
      toast.success("Provider deleted");
    } catch (e) {
      console.error(e);
      toast.error("Error deleting provider");
    }
  }

  async function setActive(providerId: string) {
    config.active_provider_id = providerId;
    try {
      await invoke("update_ai_config", { config });
      toast.success("Active provider updated");
    } catch (e) {
      console.error(e);
      toast.error("Error updating active provider");
    }
  }

  // Auto-fill base URL when provider is selected
  $effect(() => {
    if (editForm.provider_type && editingIndex !== null) {
      const remote = providers.find((p) => p.id === editForm.provider_type);
      if (remote && !editForm.base_url) {
        editForm.base_url = remote.baseUrl;
      }
      // Auto-select first model if none selected
      if (remote && !editForm.default_model && remote.models.length > 0) {
        editForm.default_model = remote.models[0].id;
      }
    }
  });
</script>

<ScrollArea.Root class="h-full w-full" type="hover">
  <ScrollArea.Viewport class="h-full w-full">
    <main class="h-full w-full pr-2 pb-8">
      <!-- Header -->
      <div class="mb-6 px-1">
        <h2
          class="mb-1 text-sm font-semibold text-neutral-900 dark:text-neutral-100"
        >
          AI Providers
        </h2>
        <p class="text-xs text-neutral-500 dark:text-neutral-400">
          管理你的 AI 服务提供商
        </p>
      </div>

      <!-- Provider List or Edit Form -->
      <div class="space-y-3">
        {#if editingIndex === null}
          <!-- List View -->
          {#if config.providers.length === 0}
            <div
              class="rounded-xl border border-dashed border-neutral-300 bg-neutral-50 px-6 py-12 text-center dark:border-neutral-700 dark:bg-neutral-900/50"
            >
              <p class="mb-4 text-sm text-neutral-500 dark:text-neutral-400">
                No providers configured yet
              </p>
              <Button.Root
                class="inline-flex h-9 items-center justify-center gap-2 rounded-lg bg-neutral-900 px-4 text-sm font-medium text-neutral-50 shadow-sm transition-colors hover:bg-neutral-900/90 focus-visible:ring-2 focus-visible:ring-neutral-950 focus-visible:outline-hidden dark:bg-neutral-50 dark:text-neutral-900 dark:hover:bg-neutral-50/90"
                onclick={startAdd}
              >
                <Plus class="h-4 w-4" />
                Add Your First Provider
              </Button.Root>
            </div>
          {:else}
            {#each config.providers as provider, index (provider.id)}
              <div
                class="group relative overflow-hidden rounded-xl border transition-all {config.active_provider_id ===
                provider.id
                  ? 'border-green-500 bg-green-50/50 dark:border-green-600 dark:bg-green-950/20'
                  : 'border-neutral-200 bg-white hover:border-neutral-300 dark:border-neutral-800 dark:bg-neutral-900 dark:hover:border-neutral-700'}"
              >
                <div class="flex items-start gap-4 p-4">
                  <!-- Active Indicator -->
                  <button
                    class="mt-0.5 flex h-5 w-5 shrink-0 items-center justify-center rounded-full border-2 transition-colors {config.active_provider_id ===
                    provider.id
                      ? 'border-green-500 bg-green-500'
                      : 'border-neutral-300 hover:border-neutral-400 dark:border-neutral-600 dark:hover:border-neutral-500'}"
                    onclick={() => setActive(provider.id)}
                    aria-label="Set as active provider"
                  >
                    {#if config.active_provider_id === provider.id}
                      <div class="h-2 w-2 rounded-full bg-white"></div>
                    {/if}
                  </button>

                  <!-- Provider Info -->
                  <div class="min-w-0 flex-1">
                    <div class="flex items-center gap-2">
                      <h3
                        class="font-semibold text-neutral-900 dark:text-neutral-100"
                      >
                        {provider.name}
                      </h3>
                      {#if config.active_provider_id === provider.id}
                        <span
                          class="rounded-full bg-green-100 px-2 py-0.5 text-xs font-medium text-green-700 dark:bg-green-900/30 dark:text-green-400"
                        >
                          Active
                        </span>
                      {/if}
                    </div>
                    <div
                      class="mt-1 flex items-center gap-2 text-xs text-neutral-500 dark:text-neutral-400"
                    >
                      {#if provider.default_model}
                        <span>{provider.default_model}</span>
                        <span>•</span>
                      {/if}
                      <span class="truncate">{provider.base_url}</span>
                    </div>
                  </div>

                  <!-- Actions -->
                  <div class="flex shrink-0 gap-2">
                    <Button.Root
                      class="inline-flex h-8 items-center justify-center gap-1.5 rounded-md border border-neutral-200 bg-white px-3 text-xs font-medium text-neutral-700 shadow-sm transition-colors hover:bg-neutral-50 focus-visible:ring-2 focus-visible:ring-neutral-950 focus-visible:outline-hidden dark:border-neutral-700 dark:bg-neutral-800 dark:text-neutral-300 dark:hover:bg-neutral-700"
                      onclick={() => startEdit(index)}
                    >
                      <PencilSimple class="h-3.5 w-3.5" />
                      Edit
                    </Button.Root>
                    <Button.Root
                      class="inline-flex h-8 items-center justify-center gap-1.5 rounded-md border border-red-200 bg-white px-3 text-xs font-medium text-red-600 shadow-sm transition-colors hover:bg-red-50 focus-visible:ring-2 focus-visible:ring-red-500 focus-visible:outline-hidden dark:border-red-900 dark:bg-neutral-800 dark:text-red-400 dark:hover:bg-red-950/30"
                      onclick={() => deleteProvider(index)}
                    >
                      <Trash class="h-3.5 w-3.5" />
                      Delete
                    </Button.Root>
                  </div>
                </div>
              </div>
            {/each}

            <!-- Add Button -->
            <Button.Root
              class="flex h-12 w-full items-center justify-center gap-2 rounded-xl border-2 border-dashed border-neutral-300 bg-transparent text-sm font-medium text-neutral-600 transition-colors hover:border-neutral-400 hover:bg-neutral-50 dark:border-neutral-700 dark:text-neutral-400 dark:hover:border-neutral-600 dark:hover:bg-neutral-800/50"
              onclick={startAdd}
            >
              <Plus class="h-4 w-4" />
              Add New Provider
            </Button.Root>
          {/if}
        {:else}
          <!-- Edit Form -->
          <div
            class="overflow-hidden rounded-xl border border-neutral-200 bg-white dark:border-neutral-800 dark:bg-neutral-900"
          >
            <div
              class="border-b border-neutral-200 bg-neutral-50 px-4 py-3 dark:border-neutral-800 dark:bg-neutral-800/50"
            >
              <h3 class="font-semibold text-neutral-900 dark:text-neutral-100">
                {editingIndex === -1 ? "Add New Provider" : "Edit Provider"}
              </h3>
            </div>

            <div class="space-y-4 p-4">
              <!-- Provider Selector -->
              <div>
                <label
                  class="mb-1.5 block text-sm font-medium text-neutral-700 dark:text-neutral-300"
                >
                  Provider
                </label>
                <Combobox.Root
                  type="single"
                  name="provider"
                  inputValue={providerOptions.find(
                    (o) => o.value === editForm.provider_type,
                  )?.label || ""}
                  onOpenChange={(o) => {
                    if (!o) providerSearch = "";
                  }}
                  onValueChange={(v) => {
                    if (v) editForm.provider_type = v;
                    providerSearch = "";
                  }}
                >
                  <div class="relative w-full">
                    <Combobox.Input
                      oninput={(e) => (providerSearch = e.currentTarget.value)}
                      class="h-10 w-full rounded-lg border border-neutral-200 bg-white px-3 text-sm font-medium text-neutral-900 placeholder:text-neutral-500 focus:ring-2 focus:ring-neutral-950 focus:ring-offset-2 focus:outline-hidden disabled:cursor-not-allowed disabled:opacity-50 dark:border-neutral-700 dark:bg-neutral-800 dark:text-neutral-100 dark:ring-offset-neutral-950 dark:placeholder:text-neutral-400 dark:focus:ring-neutral-300"
                      placeholder="Select a provider"
                    />
                    <Combobox.Trigger
                      class="absolute top-1/2 right-3 -translate-y-1/2 text-neutral-400"
                    >
                      <CaretUpDown class="h-4 w-4" />
                    </Combobox.Trigger>
                  </div>

                  <Combobox.Portal>
                    <Combobox.Content
                      class="data-[state=open]:animate-in data-[state=closed]:animate-out data-[state=closed]:fade-out-0 data-[state=open]:fade-in-0 data-[state=closed]:zoom-out-95 data-[state=open]:zoom-in-95 data-[side=bottom]:slide-in-from-top-2 data-[side=left]:slide-in-from-right-2 data-[side=right]:slide-in-from-left-2 data-[side=top]:slide-in-from-bottom-2 z-50 max-h-64 w-[var(--bits-combobox-anchor-width)] overflow-hidden rounded-md border border-neutral-200 bg-white shadow-md dark:border-neutral-800 dark:bg-neutral-950 dark:text-neutral-50"
                    >
                      <Combobox.ScrollUpButton
                        class="flex w-full items-center justify-center py-1 text-neutral-400"
                      >
                        <CaretDoubleUp class="h-3 w-3" />
                      </Combobox.ScrollUpButton>
                      <Combobox.Viewport class="p-1">
                        {#each filteredProviderOptions as option (option.value)}
                          <Combobox.Item
                            class="flex cursor-pointer items-center rounded-sm px-2 py-1.5 text-sm outline-hidden select-none data-[highlighted]:bg-neutral-100 dark:data-[highlighted]:bg-neutral-800"
                            value={option.value}
                            label={option.label}
                          >
                            {#snippet children({ selected })}
                              <span class="flex-1">{option.label}</span>
                              {#if selected}
                                <Check class="h-4 w-4" />
                              {/if}
                            {/snippet}
                          </Combobox.Item>
                        {:else}
                          <div
                            class="px-2 py-3 text-center text-sm text-neutral-400"
                          >
                            No results found
                          </div>
                        {/each}
                      </Combobox.Viewport>
                      <Combobox.ScrollDownButton
                        class="flex w-full items-center justify-center py-1 text-neutral-400"
                      >
                        <CaretDoubleDown class="h-3 w-3" />
                      </Combobox.ScrollDownButton>
                    </Combobox.Content>
                  </Combobox.Portal>
                </Combobox.Root>
              </div>

              <!-- Base URL -->
              <div>
                <label
                  class="mb-1.5 block text-sm font-medium text-neutral-700 dark:text-neutral-300"
                >
                  Base URL
                </label>
                <input
                  type="text"
                  bind:value={editForm.base_url}
                  placeholder="https://..."
                  class="h-10 w-full rounded-lg border border-neutral-200 bg-white px-3 text-sm placeholder:text-neutral-400 focus:border-neutral-900 focus:outline-hidden dark:border-neutral-700 dark:bg-neutral-800 dark:text-neutral-100 dark:focus:border-neutral-100"
                />
              </div>

              <!-- API Key -->
              <div>
                <label
                  class="mb-1.5 block text-sm font-medium text-neutral-700 dark:text-neutral-300"
                >
                  API Key
                </label>
                <input
                  type="password"
                  bind:value={editForm.api_key}
                  placeholder="sk-..."
                  class="h-10 w-full rounded-lg border border-neutral-200 bg-white px-3 text-sm placeholder:text-neutral-400 focus:border-neutral-900 focus:outline-hidden dark:border-neutral-700 dark:bg-neutral-800 dark:text-neutral-100 dark:focus:border-neutral-100"
                />
              </div>

              <!-- Model Selector -->
              <div>
                <label
                  class="mb-1.5 block text-sm font-medium text-neutral-700 dark:text-neutral-300"
                >
                  Default Model
                </label>
                {#if modelOptions.length > 0}
                  <Combobox.Root
                    type="single"
                    name="model"
                    inputValue={modelOptions.find(
                      (o) => o.value === editForm.default_model,
                    )?.label ||
                      editForm.default_model ||
                      ""}
                    onOpenChange={(o) => {
                      if (!o) modelSearch = "";
                    }}
                    onValueChange={(v) => {
                      if (v) editForm.default_model = v;
                      modelSearch = "";
                    }}
                  >
                    <div class="relative w-full">
                      <Combobox.Input
                        oninput={(e) => (modelSearch = e.currentTarget.value)}
                        class="h-10 w-full rounded-lg border border-neutral-200 bg-white px-3 text-sm font-medium text-neutral-900 placeholder:text-neutral-500 focus:ring-2 focus:ring-neutral-950 focus:ring-offset-2 focus:outline-hidden disabled:cursor-not-allowed disabled:opacity-50 dark:border-neutral-700 dark:bg-neutral-800 dark:text-neutral-100 dark:ring-offset-neutral-950 dark:placeholder:text-neutral-400 dark:focus:ring-neutral-300"
                        placeholder="Select a model"
                      />
                      <Combobox.Trigger
                        class="absolute top-1/2 right-3 -translate-y-1/2 text-neutral-400"
                      >
                        <CaretUpDown class="h-4 w-4" />
                      </Combobox.Trigger>
                    </div>

                    <Combobox.Portal>
                      <Combobox.Content
                        class="data-[state=open]:animate-in data-[state=closed]:animate-out data-[state=closed]:fade-out-0 data-[state=open]:fade-in-0 data-[state=closed]:zoom-out-95 data-[state=open]:zoom-in-95 data-[side=bottom]:slide-in-from-top-2 data-[side=left]:slide-in-from-right-2 data-[side=right]:slide-in-from-left-2 data-[side=top]:slide-in-from-bottom-2 z-50 max-h-64 w-[var(--bits-combobox-anchor-width)] overflow-hidden rounded-md border border-neutral-200 bg-white shadow-md dark:border-neutral-800 dark:bg-neutral-950 dark:text-neutral-50"
                      >
                        <Combobox.ScrollUpButton
                          class="flex w-full items-center justify-center py-1 text-neutral-400"
                        >
                          <CaretDoubleUp class="h-3 w-3" />
                        </Combobox.ScrollUpButton>
                        <Combobox.Viewport class="p-1">
                          {#each filteredModelOptions as option (option.value)}
                            <Combobox.Item
                              class="flex cursor-pointer items-center rounded-sm px-2 py-1.5 text-sm outline-hidden select-none data-[highlighted]:bg-neutral-100 dark:data-[highlighted]:bg-neutral-800"
                              value={option.value}
                              label={option.label}
                            >
                              {#snippet children({ selected })}
                                <span class="flex-1">{option.label}</span>
                                {#if selected}
                                  <Check class="h-4 w-4" />
                                {/if}
                              {/snippet}
                            </Combobox.Item>
                          {:else}
                            <div
                              class="px-2 py-3 text-center text-sm text-neutral-400"
                            >
                              No results found
                            </div>
                          {/each}
                        </Combobox.Viewport>
                        <Combobox.ScrollDownButton
                          class="flex w-full items-center justify-center py-1 text-neutral-400"
                        >
                          <CaretDoubleDown class="h-3 w-3" />
                        </Combobox.ScrollDownButton>
                      </Combobox.Content>
                    </Combobox.Portal>
                  </Combobox.Root>
                {:else}
                  <input
                    class="h-10 w-full rounded-lg border border-neutral-200 bg-white px-3 text-sm placeholder:text-neutral-400 focus:border-neutral-900 focus:outline-hidden dark:border-neutral-700 dark:bg-neutral-800 dark:text-neutral-100 dark:focus:border-neutral-100"
                    bind:value={editForm.default_model}
                    placeholder="e.g. gpt-4o"
                  />
                {/if}
              </div>

              <!-- Actions -->
              <div class="flex justify-end gap-2 pt-2">
                <Button.Root
                  class="inline-flex h-9 items-center justify-center rounded-lg border border-neutral-200 bg-white px-4 text-sm font-medium text-neutral-700 shadow-sm transition-colors hover:bg-neutral-50 focus-visible:ring-2 focus-visible:ring-neutral-950 focus-visible:outline-hidden dark:border-neutral-700 dark:bg-neutral-800 dark:text-neutral-300 dark:hover:bg-neutral-700"
                  onclick={cancelEdit}
                >
                  Cancel
                </Button.Root>
                <Button.Root
                  class="inline-flex h-9 items-center justify-center rounded-lg bg-neutral-900 px-4 text-sm font-semibold text-neutral-50 shadow-sm transition-colors hover:bg-neutral-900/90 focus-visible:ring-2 focus-visible:ring-neutral-950 focus-visible:outline-hidden dark:bg-neutral-50 dark:text-neutral-900 dark:hover:bg-neutral-50/90"
                  onclick={save}
                >
                  Save
                </Button.Root>
              </div>
            </div>
          </div>
        {/if}
      </div>
    </main>
  </ScrollArea.Viewport>
  <ScrollArea.Scrollbar
    orientation="vertical"
    class="bg-muted hover:bg-dark-10 data-[state=visible]:animate-in data-[state=hidden]:animate-out data-[state=hidden]:fade-out-0 data-[state=visible]:fade-in-0 flex w-1.5 touch-none rounded-full border-l border-l-transparent p-px transition-all duration-200 select-none hover:w-3"
  >
    <ScrollArea.Thumb class="bg-muted-foreground flex-1 rounded-full" />
  </ScrollArea.Scrollbar>
  <ScrollArea.Corner />
</ScrollArea.Root>
