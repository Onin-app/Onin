<script lang="ts">
  import { onDestroy } from "svelte";
  import { Label, RadioGroup } from "bits-ui";

  import { theme, toggleTheme } from "$lib/utils/theme";
  import { Theme } from "$lib/type";
  import SetItem from "./SetItem.svelte";

  const themeList: { value: Theme; label: string }[] = [
    {
      value: Theme.SYSTEM,
      label: "跟随系统",
    },
    {
      value: Theme.LIGHT,
      label: "明亮",
    },
    {
      value: Theme.DARK,
      label: "暗黑",
    },
  ];
  let currentTheme = $state<Theme>(Theme.DARK);

  const getTheme = () => currentTheme;
  const setTheme = (value: Theme) => {
    currentTheme = value;
    toggleTheme(value);
  };

  const unsubscribe = theme.subscribe((value) => {
    currentTheme = value;
  });

  onDestroy(unsubscribe);
</script>

<main class="w-full h-full p-4">
  <h2 class="text-xl font-bold">主题设置</h2>
  <SetItem title="主题">
    {#snippet content()}
      <RadioGroup.Root
        class="flex gap-4 text-sm font-medium"
        bind:value={getTheme, setTheme}
      >
        {#each themeList as theme}
          <div
            class="text-foreground group flex select-none items-center transition-all"
          >
            <RadioGroup.Item
              id={theme.value}
              value={theme.value}
              class="cursor-pointer border-border-input bg-background hover:border-dark-40 data-[state=checked]:border-foreground data-[state=checked]:border-6 size-5 shrink-0 cursor-default rounded-full border transition-all duration-100 ease-in-out"
            />
            <Label.Root for={theme.value} class="pl-3 cursor-pointer">
              {theme.label}
            </Label.Root>
          </div>
        {/each}
      </RadioGroup.Root>
    {/snippet}
  </SetItem>
</main>
