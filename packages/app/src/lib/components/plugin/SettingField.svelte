<script lang="ts">
  import type { SettingField } from "$lib/types/plugin-settings";
  import { Switch, Select, Button, Label, RadioGroup, Slider } from "bits-ui";
  import {
    Check,
    CaretUpDown,
    CaretDoubleUp,
    CaretDoubleDown,
    Eye,
    EyeSlash,
  } from "phosphor-svelte";

  interface Props {
    field: SettingField;
    value: any;
    onChange: (value: any) => void;
  }

  let { field, value = $bindable(), onChange }: Props = $props();

  // 判断是否为多选 select
  const isMultipleSelect = $derived(
    field.type === "select" && "multiple" in field && field.multiple === true,
  );

  // 生成唯一的 ID（使用 $derived 使其响应式）
  const fieldId = $derived(
    `setting-${field.key}-${Math.random().toString(36).substring(2, 9)}`,
  );

  // 密码显示/隐藏状态
  let showPassword = $state(false);

  // 初始化默认值
  $effect(() => {
    if (
      value === undefined &&
      "defaultValue" in field &&
      field.defaultValue !== undefined
    ) {
      value = field.defaultValue;
      onChange(value);
    }
  });
</script>

<div
  class="setting-field border-b border-neutral-200 py-3 dark:border-neutral-700"
>
  <div class="flex items-start gap-6">
    <div class="min-w-0 flex-1">
      <label for={fieldId} class="block text-sm font-medium">
        {field.label}
        {#if field.required}
          <span class="text-red-500">*</span>
        {/if}
      </label>

      {#if field.description}
        <p class="mt-1 text-xs text-neutral-500 dark:text-neutral-400">
          {field.description}
        </p>
      {/if}
    </div>

    <div class="flex min-w-0 flex-1 justify-end">
      {#if field.type === "password"}
        <div class="relative w-full">
          <input
            id={fieldId}
            type={showPassword ? "text" : "password"}
            {value}
            placeholder={field.placeholder}
            required={field.required}
            maxlength={field.maxLength}
            minlength={field.minLength}
            onchange={(e) => onChange((e.target as HTMLInputElement).value)}
            class="h-8 w-full rounded border border-neutral-300 bg-white px-3 py-2 pr-10 text-sm focus:border-neutral-500 focus:outline-none dark:border-neutral-600 dark:bg-neutral-800"
          />
          <button
            type="button"
            onclick={() => (showPassword = !showPassword)}
            class="absolute top-1/2 right-2 -translate-y-1/2 rounded p-1 text-neutral-500 hover:bg-neutral-100 hover:text-neutral-700 dark:text-neutral-400 dark:hover:bg-neutral-700 dark:hover:text-neutral-200"
            aria-label={showPassword ? "隐藏密码" : "显示密码"}
          >
            {#if showPassword}
              <EyeSlash class="size-4" />
            {:else}
              <Eye class="size-4" />
            {/if}
          </button>
        </div>
      {:else if field.type === "text" || field.type === "color" || field.type === "date" || field.type === "time" || field.type === "datetime"}
        <input
          id={fieldId}
          type={field.type === "datetime" ? "datetime-local" : field.type}
          {value}
          placeholder={"placeholder" in field ? field.placeholder : undefined}
          required={field.required}
          maxlength={"maxLength" in field ? field.maxLength : undefined}
          minlength={"minLength" in field ? field.minLength : undefined}
          onchange={(e) => onChange((e.target as HTMLInputElement).value)}
          class="h-8 w-full rounded border border-neutral-300 bg-white px-3 py-2 text-sm focus:border-neutral-500 focus:outline-none dark:border-neutral-600 dark:bg-neutral-800"
        />
      {:else if field.type === "textarea"}
        <textarea
          id={fieldId}
          {value}
          placeholder={field.placeholder}
          required={field.required}
          maxlength={field.maxLength}
          minlength={field.minLength}
          onchange={(e) => onChange((e.target as HTMLInputElement).value)}
          rows="3"
          class="w-full resize-y rounded border border-neutral-300 bg-white px-3 py-2 text-sm focus:border-neutral-500 focus:outline-none dark:border-neutral-600 dark:bg-neutral-800"
        ></textarea>
      {:else if field.type === "number"}
        <input
          id={fieldId}
          type="number"
          {value}
          placeholder={field.placeholder}
          required={field.required}
          min={field.min}
          max={field.max}
          step={field.step}
          onchange={(e) => onChange((e.target as HTMLInputElement).value)}
          class="h-8 w-full rounded border border-neutral-300 bg-white px-3 py-2 text-sm focus:border-neutral-500 focus:outline-none dark:border-neutral-600 dark:bg-neutral-800"
        />
      {:else if field.type === "slider"}
        <div class="w-full">
          <Slider.Root
            type="single"
            {value}
            onValueChange={onChange}
            class="relative flex w-full touch-none items-center select-none"
          >
            <span
              class="bg-dark-10 relative h-2 w-full grow cursor-pointer overflow-hidden rounded-full"
            >
              <Slider.Range class="bg-foreground absolute h-full" />
            </span>
            <Slider.Thumb
              index={0}
              class="border-border-input bg-background hover:border-dark-40 focus-visible:ring-foreground dark:bg-foreground dark:shadow-card data-active:border-dark-40 block size-[25px] cursor-pointer rounded-full border shadow-sm transition-colors focus-visible:ring-2 focus-visible:ring-offset-2 focus-visible:outline-hidden disabled:pointer-events-none disabled:opacity-50 data-active:scale-[0.98]"
            />
            <Slider.Tick index={value} />
            <Slider.TickLabel index={value} position="right">
              {value}
            </Slider.TickLabel>
          </Slider.Root>
        </div>
      {:else if field.type === "switch"}
        <Switch.Root
          checked={value}
          onCheckedChange={onChange}
          class="focus-visible:ring-foreground focus-visible:ring-offset-background data-[state=checked]:bg-foreground data-[state=unchecked]:bg-dark-10 data-[state=unchecked]:shadow-mini-inset dark:data-[state=checked]:bg-foreground peer inline-flex h-5 min-h-5 w-9 shrink-0 cursor-pointer items-center rounded-full px-[2px] transition-colors focus-visible:ring-1 focus-visible:ring-offset-1 focus-visible:outline-hidden disabled:cursor-not-allowed disabled:opacity-50"
        >
          <Switch.Thumb
            class="bg-background data-[state=unchecked]:shadow-mini dark:border-background/30 dark:bg-foreground dark:shadow-popover pointer-events-none block size-4 shrink-0 rounded-full transition-transform data-[state=checked]:translate-x-[14px] data-[state=unchecked]:translate-x-0 dark:border dark:data-[state=unchecked]:border"
          />
        </Switch.Root>
      {:else if field.type === "radio"}
        <RadioGroup.Root
          {value}
          onValueChange={onChange}
          class="flex flex-col gap-2 text-sm font-medium"
        >
          {#each field.options as option}
            <div
              class="text-foreground group flex items-center transition-all select-none"
            >
              <RadioGroup.Item
                id={option.value}
                value={option.value}
                class="border-border-input bg-background hover:border-dark-40 data-[state=checked]:border-foreground size-4 shrink-0 cursor-default rounded-full border transition-all duration-100 ease-in-out data-[state=checked]:border-[5px]"
              />
              <Label.Root
                for={option.value}
                class="cursor-pointer pl-2 text-sm"
              >
                {option.label}
              </Label.Root>
            </div>
          {/each}
        </RadioGroup.Root>
      {:else if field.type === "select"}
        <Select.Root
          type={isMultipleSelect ? "multiple" : "single"}
          {value}
          onValueChange={onChange}
          items={"options" in field ? field.options : []}
          allowDeselect
        >
          <Select.Trigger
            class="border-border-input bg-background data-placeholder:text-foreground-alt/50 inline-flex h-9 w-full max-w-xs touch-none items-center rounded-lg border px-3 text-sm transition-colors select-none"
            aria-label={field.label}
          >
            {value
              ? isMultipleSelect
                ? ("options" in field ? field.options : [])
                    .filter((o) => value.includes(o.value))
                    .map((o) => o.label)
                    .join("、")
                : ("options" in field ? field.options : []).find(
                    (o) => o.value === value,
                  )?.label || field.label
              : ("placeholder" in field ? field.placeholder : undefined) ||
                field.label}
            <CaretUpDown class="text-muted-foreground ml-auto size-4" />
          </Select.Trigger>
          <Select.Portal>
            <Select.Content
              class="focus-override border-muted bg-background shadow-popover data-[state=open]:animate-in data-[state=closed]:animate-out data-[state=closed]:fade-out-0 data-[state=open]:fade-in-0 data-[state=closed]:zoom-out-95 data-[state=open]:zoom-in-95 data-[side=bottom]:slide-in-from-top-2 data-[side=left]:slide-in-from-right-2 data-[side=right]:slide-in-from-left-2 data-[side=top]:slide-in-from-bottom-2 z-50 max-h-60 w-[var(--bits-select-anchor-width)] min-w-[var(--bits-select-anchor-width)] rounded-lg border px-1 py-2 outline-hidden select-none data-[side=bottom]:translate-y-1 data-[side=left]:-translate-x-1 data-[side=right]:translate-x-1 data-[side=top]:-translate-y-1"
              sideOffset={8}
            >
              <Select.ScrollUpButton
                class="flex w-full items-center justify-center py-1"
              >
                <CaretDoubleUp class="size-3" />
              </Select.ScrollUpButton>
              <Select.Viewport class="p-1">
                {#each "options" in field ? field.options : [] as option, i (i + option.value)}
                  <Select.Item
                    class="data-highlighted:bg-muted flex h-8 w-full cursor-pointer items-center rounded py-2 pr-2 pl-3 text-sm outline-hidden select-none data-disabled:opacity-50"
                    value={option.value}
                    label={option.label}
                  >
                    {#snippet children({ selected })}
                      {option.label}
                      {#if selected}
                        <div class="ml-auto">
                          <Check aria-label="check" class="size-4" />
                        </div>
                      {/if}
                    {/snippet}
                  </Select.Item>
                {/each}
              </Select.Viewport>
              <Select.ScrollDownButton
                class="flex w-full items-center justify-center py-1"
              >
                <CaretDoubleDown class="size-3" />
              </Select.ScrollDownButton>
            </Select.Content>
          </Select.Portal>
        </Select.Root>
      {:else if field.type === "button"}
        <Button.Root
          onclick={() => field.onClick?.()}
          class="inline-flex h-8 items-center justify-center rounded-md bg-neutral-900 px-3 text-sm font-medium text-white hover:bg-neutral-800 active:scale-[0.98] dark:bg-neutral-100 dark:text-neutral-900 dark:hover:bg-neutral-200"
        >
          {field.buttonText || field.label}
        </Button.Root>
      {/if}
    </div>
  </div>
</div>
