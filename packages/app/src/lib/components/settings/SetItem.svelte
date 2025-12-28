<script lang="ts">
  import type { Snippet } from "svelte";
  import { Tooltip } from "bits-ui";
  import { Question } from "phosphor-svelte";

  interface SetItemProps {
    title: string;
    description?: string;
    content: Snippet;
  }

  const { title, description, content }: SetItemProps = $props();
</script>

<div class="mt-2 flex items-center justify-between">
  <div class="label flex items-center">
    <p class="title mr-2">{title}</p>
    {#if description}
      <Tooltip.Provider>
        <Tooltip.Root delayDuration={200}>
          <Tooltip.Trigger class="">
            <Question size={20} />
          </Tooltip.Trigger>
          <Tooltip.Content sideOffset={8}>
            <div class="">{description}</div>
          </Tooltip.Content>
        </Tooltip.Root>
      </Tooltip.Provider>
    {/if}
  </div>
  <div class="content">
    {@render content()}
  </div>
</div>
