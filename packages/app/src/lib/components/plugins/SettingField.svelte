<script lang="ts">
  import type { SettingField } from "$lib/types/plugin-settings";
  import { Switch } from "$lib/components/ui/switch";
  import { Slider } from "$lib/components/ui/slider";
  import { Button } from "$lib/components/ui/button";
  import { Input } from "$lib/components/ui/input";
  import {
    Select,
    SelectTrigger,
    SelectContent,
    SelectItem,
  } from "$lib/components/ui/select";
  import { RadioGroup, Label } from "bits-ui";
  import { Eye, EyeSlash } from "phosphor-svelte";

  interface Props {
    field: SettingField;
    value: any;
    onChange: (value: any) => void;
  }

  let { field, value = $bindable(), onChange }: Props = $props();

  const isMultipleSelect = $derived(
    field.type === "select" && "multiple" in field && field.multiple === true,
  );

  const fieldId = $derived(
    `setting-${field.key}-${Math.random().toString(36).substring(2, 9)}`,
  );

  let showPassword = $state(false);

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

<div class="setting-field border-border/40 border-b py-3 last:border-0">
  <div class="flex items-start gap-6">
    <div class="min-w-0 flex-1">
      <label for={fieldId} class="text-foreground block text-sm font-medium">
        {field.label}
        {#if field.required}
          <span class="text-destructive">*</span>
        {/if}
      </label>

      {#if field.description}
        <p class="text-muted-foreground mt-1 text-xs">
          {field.description}
        </p>
      {/if}
    </div>

    <div class="flex min-w-0 flex-1 justify-end">
      {#if field.type === "password"}
        <div class="relative w-full">
          <Input
            id={fieldId}
            type={showPassword ? "text" : "password"}
            {value}
            placeholder={field.placeholder}
            required={field.required}
            maxlength={field.maxLength}
            minlength={field.minLength}
            onchange={(e) => onChange((e.target as HTMLInputElement).value)}
            class="h-8 w-full pr-10 text-xs"
          />
          <button
            type="button"
            onclick={() => (showPassword = !showPassword)}
            class="text-muted-foreground hover:text-foreground absolute top-1/2 right-2 -translate-y-1/2 cursor-pointer rounded p-1"
            aria-label={showPassword ? "隐藏密码" : "显示密码"}
          >
            {#if showPassword}
              <EyeSlash class="size-3.5" />
            {:else}
              <Eye class="size-3.5" />
            {/if}
          </button>
        </div>
      {:else if field.type === "text" || field.type === "color" || field.type === "date" || field.type === "time" || field.type === "datetime"}
        <Input
          id={fieldId}
          type={field.type === "datetime" ? "datetime-local" : field.type}
          {value}
          placeholder={"placeholder" in field ? field.placeholder : undefined}
          required={field.required}
          maxlength={"maxLength" in field ? field.maxLength : undefined}
          minlength={"minLength" in field ? field.minLength : undefined}
          onchange={(e) => onChange((e.target as HTMLInputElement).value)}
          class="h-8 w-full text-xs"
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
          class="border-input placeholder:text-muted-foreground focus-visible:ring-ring w-full resize-y rounded-md border bg-transparent px-3 py-2 text-xs focus-visible:ring-1 focus-visible:outline-none"
        ></textarea>
      {:else if field.type === "number"}
        <Input
          id={fieldId}
          type="number"
          {value}
          placeholder={field.placeholder}
          required={field.required}
          min={field.min}
          max={field.max}
          step={field.step}
          onchange={(e) => onChange((e.target as HTMLInputElement).value)}
          class="h-8 w-full text-xs"
        />
      {:else if field.type === "slider"}
        <div class="w-full">
          <Slider type="single" {value} onValueChange={onChange} />
        </div>
      {:else if field.type === "switch"}
        <Switch checked={value} onCheckedChange={onChange} />
      {:else if field.type === "radio"}
        <RadioGroup.Root
          {value}
          onValueChange={onChange}
          class="flex flex-col gap-2 text-xs font-medium"
        >
          {#each field.options as option}
            <div
              class="text-foreground group flex items-center transition-all select-none"
            >
              <RadioGroup.Item
                id={option.value}
                value={option.value}
                class="border-input bg-background hover:border-foreground data-[state=checked]:border-primary size-4 shrink-0 cursor-pointer rounded-full border transition-all data-[state=checked]:border-[5px]"
              />
              <Label.Root
                for={option.value}
                class="cursor-pointer pl-2 text-xs"
              >
                {option.label}
              </Label.Root>
            </div>
          {/each}
        </RadioGroup.Root>
      {:else if field.type === "select"}
        <Select
          type={isMultipleSelect ? "multiple" : "single"}
          {value}
          onValueChange={onChange}
        >
          <SelectTrigger class="h-8 max-w-xs text-xs" aria-label={field.label}>
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
          </SelectTrigger>
          <SelectContent class="w-[var(--bits-select-anchor-width)]">
            {#each "options" in field ? field.options : [] as option, i (i + option.value)}
              <SelectItem value={option.value} label={option.label}>
                <span class="truncate">{option.label}</span>
              </SelectItem>
            {/each}
          </SelectContent>
        </Select>
      {:else if field.type === "button"}
        <Button size="sm" class="h-8 text-xs" onclick={() => field.onClick?.()}>
          {field.buttonText || field.label}
        </Button>
      {/if}
    </div>
  </div>
</div>
