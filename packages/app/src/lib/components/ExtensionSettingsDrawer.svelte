<script lang="ts">
  import {
    Sheet,
    SheetContent,
    SheetHeader,
    SheetTitle,
  } from "$lib/components/ui/sheet";
  import { ScrollArea } from "$lib/components/ui/scroll-area";
  import FileSearchSettings from "$lib/components/settings/FileSearchSettings.svelte";
  import OcrSettings from "$lib/components/settings/OcrSettings.svelte";

  interface Props {
    open: boolean;
    extensionId: string | null;
    extensionName: string;
  }

  let { open = $bindable(false), extensionId, extensionName }: Props = $props();
</script>

<Sheet bind:open>
  <SheetContent side="right" class="flex flex-col p-0 sm:max-w-[480px]">
    <SheetHeader class="border-b px-6 py-4">
      <SheetTitle>
        {extensionName || "扩展"}设置
      </SheetTitle>
    </SheetHeader>

    <div class="flex-1 overflow-hidden">
      <ScrollArea class="h-full w-full" viewportClass="h-full w-full">
        <div class="p-6">
          {#if extensionId === "file_search"}
            <FileSearchSettings />
          {:else if extensionId === "ocr"}
            <OcrSettings />
          {/if}
        </div>
      </ScrollArea>
    </div>
  </SheetContent>
</Sheet>
