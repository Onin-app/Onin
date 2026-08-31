<script lang="ts">
  import {
    AlertDialog,
    AlertDialogContent,
    AlertDialogHeader,
    AlertDialogFooter,
    AlertDialogTitle,
    AlertDialogDescription,
    AlertDialogAction,
    AlertDialogCancel,
  } from "$lib/components/ui/alert-dialog";
  import { buttonVariants } from "$lib/components/ui/button";
  import { WarningCircle } from "phosphor-svelte";

  interface Props {
    open: boolean;
    title: string;
    description: string;
    onConfirm: () => void | Promise<void>;
    onCancel: () => void;
    confirmLabel?: string;
    cancelLabel?: string;
    loading?: boolean;
    closeOnConfirm?: boolean;
    variant?: "danger" | "default";
  }

  let {
    open = $bindable(false),
    title,
    description,
    onConfirm,
    onCancel,
    confirmLabel = "确认",
    cancelLabel = "取消",
    loading = false,
    closeOnConfirm = true,
    variant = "danger",
  }: Props = $props();

  function handleOpenChange(newOpen: boolean) {
    open = newOpen;
    if (!newOpen) {
      onCancel();
    }
  }

  async function handleConfirm(event: MouseEvent) {
    if (loading) return;
    if (!closeOnConfirm) {
      event.preventDefault();
    }
    await onConfirm();
    if (closeOnConfirm) {
      open = false;
    }
  }

  function handleCancel() {
    onCancel();
    open = false;
  }
</script>

<AlertDialog {open} onOpenChange={handleOpenChange}>
  <AlertDialogContent class="max-w-md">
    <AlertDialogHeader>
      <div class="flex items-center gap-3">
        <div
          class="flex h-10 w-10 shrink-0 items-center justify-center rounded-full {variant ===
          'danger'
            ? 'bg-destructive/10 text-destructive'
            : 'bg-muted text-foreground'}"
        >
          <WarningCircle class="h-5 w-5" weight="fill" />
        </div>
        <AlertDialogTitle class="text-base font-semibold">
          {title}
        </AlertDialogTitle>
      </div>
      <AlertDialogDescription class="text-muted-foreground pt-2 text-sm">
        {description}
      </AlertDialogDescription>
    </AlertDialogHeader>

    <AlertDialogFooter class="gap-2 sm:gap-2">
      <AlertDialogCancel disabled={loading} onclick={handleCancel}>
        {cancelLabel}
      </AlertDialogCancel>
      <AlertDialogAction
        class={variant === "danger"
          ? buttonVariants({ variant: "destructive" })
          : buttonVariants({ variant: "default" })}
        disabled={loading}
        onclick={handleConfirm}
      >
        {loading ? "处理中..." : confirmLabel}
      </AlertDialogAction>
    </AlertDialogFooter>
  </AlertDialogContent>
</AlertDialog>
