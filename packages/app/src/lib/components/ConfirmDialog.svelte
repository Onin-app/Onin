<script lang="ts">
  import { AlertDialog } from "bits-ui";
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
      // 对话框关闭时,清理待处理的操作
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

<AlertDialog.Root {open} onOpenChange={handleOpenChange}>
  <AlertDialog.Portal>
    <AlertDialog.Overlay
      class="data-[state=open]:animate-in data-[state=closed]:animate-out data-[state=closed]:fade-out-0 data-[state=open]:fade-in-0 fixed inset-0 z-50 bg-black/50"
    />
    <AlertDialog.Content
      class="data-[state=open]:animate-in data-[state=closed]:animate-out data-[state=closed]:fade-out-0 data-[state=open]:fade-in-0 data-[state=closed]:zoom-out-95 data-[state=open]:zoom-in-95 data-[state=closed]:slide-out-to-left-1/2 data-[state=closed]:slide-out-to-top-[48%] data-[state=open]:slide-in-from-left-1/2 data-[state=open]:slide-in-from-top-[48%] fixed top-[50%] left-[50%] z-50 w-full max-w-md translate-x-[-50%] translate-y-[-50%] rounded-lg bg-white p-6 shadow-xl dark:bg-neutral-900"
    >
      <!-- 警告图标 -->
      <div class="mb-4 flex items-center gap-3">
        <div
          class="flex h-12 w-12 shrink-0 items-center justify-center rounded-full {variant ===
          'danger'
            ? 'bg-red-100 dark:bg-red-900/30'
            : 'bg-neutral-100 dark:bg-neutral-800'}"
        >
          <WarningCircle
            class="h-6 w-6 {variant === 'danger'
              ? 'text-red-600 dark:text-red-400'
              : 'text-neutral-700 dark:text-neutral-200'}"
            weight="fill"
          />
        </div>
        <div class="flex-1">
          <AlertDialog.Title
            class="text-lg font-semibold text-neutral-900 dark:text-neutral-100"
          >
            {title}
          </AlertDialog.Title>
        </div>
      </div>

      <!-- 描述 -->
      <AlertDialog.Description
        class="mb-6 text-sm text-neutral-600 dark:text-neutral-400"
      >
        {description}
      </AlertDialog.Description>

      <!-- 操作按钮 -->
      <div class="flex justify-end gap-3">
        <AlertDialog.Cancel
          class="inline-flex h-9 items-center justify-center rounded-md border border-neutral-300 bg-white px-4 text-sm font-medium text-neutral-700 hover:bg-neutral-50 focus:ring-2 focus:ring-neutral-400 focus:ring-offset-2 focus:outline-none dark:border-neutral-600 dark:bg-neutral-800 dark:text-neutral-300 dark:hover:bg-neutral-700"
          disabled={loading}
          onclick={handleCancel}
        >
          {cancelLabel}
        </AlertDialog.Cancel>
        <AlertDialog.Action
          class="inline-flex h-9 items-center justify-center rounded-md px-4 text-sm font-medium text-white focus:ring-2 focus:ring-offset-2 focus:outline-none disabled:cursor-not-allowed disabled:opacity-70 {variant ===
          'danger'
            ? 'bg-red-600 hover:bg-red-700 focus:ring-red-500 dark:bg-red-700 dark:hover:bg-red-800'
            : 'bg-neutral-900 hover:bg-neutral-700 focus:ring-neutral-500 dark:bg-neutral-100 dark:text-neutral-900 dark:hover:bg-neutral-300'}"
          disabled={loading}
          onclick={handleConfirm}
        >
          {loading ? "处理中..." : confirmLabel}
        </AlertDialog.Action>
      </div>
    </AlertDialog.Content>
  </AlertDialog.Portal>
</AlertDialog.Root>
