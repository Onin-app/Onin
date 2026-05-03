<script lang="ts">
  import { onMount } from "svelte";
  import { goto } from "$app/navigation";
  import { page } from "$app/stores";
  import { invoke } from "@tauri-apps/api/core";
  import { toast } from "svelte-sonner";
  import { Slider } from "bits-ui";
  import { Copy, ClipboardText } from "phosphor-svelte";
  import AppScrollArea from "$lib/components/AppScrollArea.svelte";
  import ExtensionHeader from "$lib/components/ExtensionHeader.svelte";

  interface ColorConversion {
    hex: string;
    rgb: string;
    hsl: string;
    red: number;
    green: number;
    blue: number;
    alpha: number;
  }

  // ── Helpers ────────────────────────────────────────────────────────────────
  function rgbToHsv(r: number, g: number, b: number): [number, number, number] {
    const rn = r / 255,
      gn = g / 255,
      bn = b / 255;
    const max = Math.max(rn, gn, bn),
      min = Math.min(rn, gn, bn);
    const d = max - min;
    const v = max;
    const s = max === 0 ? 0 : d / max;
    let h = 0;
    if (d !== 0) {
      if (max === rn) h = ((gn - bn) / d + (gn < bn ? 6 : 0)) / 6;
      else if (max === gn) h = ((bn - rn) / d + 2) / 6;
      else h = ((rn - gn) / d + 4) / 6;
    }
    return [Math.round(h * 360), Math.round(s * 100), Math.round(v * 100)];
  }

  function rgbToHwb(r: number, g: number, b: number): [number, number, number] {
    const rn = r / 255,
      gn = g / 255,
      bn = b / 255;
    const max = Math.max(rn, gn, bn),
      min = Math.min(rn, gn, bn);
    const d = max - min;
    let h = 0;
    if (d !== 0) {
      if (max === rn) h = ((gn - bn) / d + (gn < bn ? 6 : 0)) / 6;
      else if (max === gn) h = ((bn - rn) / d + 2) / 6;
      else h = ((rn - gn) / d + 4) / 6;
    }
    return [
      Math.round(h * 360),
      Math.round(min * 100),
      Math.round((1 - max) * 100),
    ];
  }

  function linearize(c: number): number {
    const cn = c / 255;
    return cn <= 0.04045 ? cn / 12.92 : Math.pow((cn + 0.055) / 1.055, 2.4);
  }

  function rgbToOklch(
    r: number,
    g: number,
    b: number,
  ): [number, number, number] {
    const rl = linearize(r),
      gl = linearize(g),
      bl = linearize(b);
    // sRGB → linear-sRGB → OKLab
    const l = 0.4122214708 * rl + 0.5363325363 * gl + 0.0514459929 * bl;
    const m = 0.2119034982 * rl + 0.6806995451 * gl + 0.1073969566 * bl;
    const s = 0.0883024619 * rl + 0.2817188376 * gl + 0.6299787005 * bl;
    const l_ = Math.cbrt(l),
      m_ = Math.cbrt(m),
      s_ = Math.cbrt(s);
    const L = 0.2104542553 * l_ + 0.793617785 * m_ - 0.0040720468 * s_;
    const a = 1.9779984951 * l_ - 2.428592205 * m_ + 0.4505937099 * s_;
    const bv = 0.0259040371 * l_ + 0.7827717662 * m_ - 0.808675766 * s_;
    const C = Math.sqrt(a * a + bv * bv);
    let H = Math.atan2(bv, a) * (180 / Math.PI);
    if (H < 0) H += 360;
    return [
      Math.round(L * 1000) / 1000,
      Math.round(C * 1000) / 1000,
      Math.round(H * 10) / 10,
    ];
  }

  // ── State ──────────────────────────────────────────────────────────────────
  let query = $state("");
  let conversion = $state<ColorConversion | null>(null);
  let headerRef: ExtensionHeader;

  const initialQuery = $derived($page.url.searchParams.get("q") || "#ff5500");
  const colorValue = $derived(conversion?.hex.slice(0, 7) || "#ff5500");

  const previewTextColor = $derived.by(() => {
    if (!conversion) return "#ffffff";
    const lum =
      (0.2126 * conversion.red +
        0.7152 * conversion.green +
        0.0722 * conversion.blue) /
      255;
    return lum > 0.55 ? "#111827" : "#ffffff";
  });

  // All derived formats – no tab needed, all shown at once
  const formats = $derived.by(() => {
    if (!conversion) return [];
    const { red: r, green: g, blue: b, alpha: a, hex, rgb, hsl } = conversion;
    const [H, S, V] = rgbToHsv(r, g, b);
    const [Hw, W, Bl] = rgbToHwb(r, g, b);
    const [L, C, lH] = rgbToOklch(r, g, b);
    const hexFull =
      a < 0.999
        ? `#${r.toString(16).padStart(2, "0")}${g.toString(16).padStart(2, "0")}${b.toString(16).padStart(2, "0")}${Math.round(
            a * 255,
          )
            .toString(16)
            .padStart(2, "0")}`.toUpperCase()
        : hex;

    return [
      { group: "Web 标准", label: "HEX", value: hexFull },
      { group: "Web 标准", label: "RGB", value: rgb },
      { group: "Web 标准", label: "HSL", value: hsl },
      {
        group: "Web 标准",
        label: "HWB",
        value:
          a < 0.999
            ? `hwb(${Hw} ${W}% ${Bl}% / ${a.toFixed(2)})`
            : `hwb(${Hw} ${W}% ${Bl}%)`,
      },
      {
        group: "扩展格式",
        label: "HSV",
        value:
          a < 0.999
            ? `hsv(${H}, ${S}%, ${V}%, ${a.toFixed(2)})`
            : `hsv(${H}, ${S}%, ${V}%)`,
      },
      {
        group: "扩展格式",
        label: "OKLCH",
        value:
          a < 0.999
            ? `oklch(${L} ${C} ${lH} / ${a.toFixed(2)})`
            : `oklch(${L} ${C} ${lH})`,
      },
      {
        group: "扩展格式",
        label: "color()",
        value:
          a < 0.999
            ? `color(srgb ${(r / 255).toFixed(4)} ${(g / 255).toFixed(4)} ${(b / 255).toFixed(4)} / ${a.toFixed(2)})`
            : `color(srgb ${(r / 255).toFixed(4)} ${(g / 255).toFixed(4)} ${(b / 255).toFixed(4)})`,
      },
      {
        group: "扩展格式",
        label: "RGBA %",
        value: `rgba(${((r / 255) * 100).toFixed(1)}%, ${((g / 255) * 100).toFixed(1)}%, ${((b / 255) * 100).toFixed(1)}%, ${a.toFixed(2)})`,
      },
    ];
  });

  const groups = $derived.by(() => {
    const map = new Map<string, typeof formats>();
    for (const f of formats) {
      if (!map.has(f.group)) map.set(f.group, []);
      map.get(f.group)!.push(f);
    }
    return [...map.entries()];
  });

  const channelRows = $derived(
    conversion
      ? [
          { key: "red" as const, label: "R", value: conversion.red },
          { key: "green" as const, label: "G", value: conversion.green },
          { key: "blue" as const, label: "B", value: conversion.blue },
        ]
      : [],
  );

  // ── Actions ────────────────────────────────────────────────────────────────
  async function loadColor(value: string) {
    query = value;
    try {
      conversion = await invoke<ColorConversion | null>(
        "get_color_conversion",
        { input: value },
      );
    } catch {
      conversion = null;
    }
  }

  function handleBack() {
    goto("/");
  }

  function handleKeyDown(e: KeyboardEvent) {
    if (e.key === "Backspace" && query === "") {
      handleBack();
      return;
    }
    if (e.key === "Enter" && conversion) {
      e.preventDefault();
      copyValue(conversion.hex, "HEX");
    }
  }

  async function copyValue(value: string, label: string) {
    try {
      await navigator.clipboard.writeText(value);
      toast.success(`${label} 已复制`);
    } catch {
      toast.error("复制失败");
    }
  }

  function copyAllFormats() {
    if (!conversion) return;
    copyValue(
      formats.map((f) => `${f.label}: ${f.value}`).join("\n"),
      "全部格式",
    );
  }

  function updateChannel(channel: "red" | "green" | "blue", value: number) {
    if (!conversion) return;
    const n = Math.round(Math.min(255, Math.max(0, value || 0)));
    const next = {
      red: conversion.red,
      green: conversion.green,
      blue: conversion.blue,
      [channel]: n,
    };
    loadColor(`rgb(${next.red}, ${next.green}, ${next.blue})`);
  }

  onMount(() => {
    loadColor(initialQuery);
    headerRef?.focus();
  });
</script>

<div class="flex h-full min-h-0 w-full flex-col overflow-hidden">
  <ExtensionHeader
    bind:this={headerRef}
    placeholder="Hex, RGB, HSL…"
    bind:value={query}
    onInput={loadColor}
    onBack={handleBack}
    onKeyDown={handleKeyDown}
  />

  <AppScrollArea
    class="h-0 min-h-0 flex-1 overflow-hidden"
    viewportClass="h-full max-h-full w-full overflow-y-auto overflow-x-hidden"
  >
    <main class="color-tool">
      {#if conversion}
        <!-- Swatch -->
        <div
          class="swatch"
          style="--c:{colorValue}; --ct:{previewTextColor};"
          aria-label="颜色预览"
        >
          <div class="swatch-meta">
            <strong>{conversion.hex}</strong>
            <span>{conversion.rgb}</span>
          </div>
        </div>

        <!-- Format rows -->
        <div class="section">
          <div class="section-head">
            <span>格式</span>
            <button
              class="btn-ghost"
              onclick={copyAllFormats}
              title="复制全部格式"
            >
              <ClipboardText class="icon" /> 全部
            </button>
          </div>

          {#each groups as [group, items]}
            <div class="group-label">{group}</div>
            <div class="format-grid">
              {#each items as fmt}
                <button
                  class="fmt-cell"
                  onclick={() => copyValue(fmt.value, fmt.label)}
                >
                  <span class="fmt-label">{fmt.label}</span>
                  <code class="fmt-value">{fmt.value}</code>
                  <Copy class="icon copy-icon" />
                </button>
              {/each}
            </div>
          {/each}
        </div>

        <!-- RGB Channels -->
        <div class="section">
          <div class="section-head"><span>RGB 微调</span></div>
          <div class="channels">
            {#each channelRows as ch}
              <div class="channel-row">
                <span class="ch-label">{ch.label}</span>
                <Slider.Root
                  type="single"
                  value={ch.value}
                  min={0}
                  max={255}
                  step={1}
                  disabled={!conversion}
                  onValueChange={(v) => updateChannel(ch.key, v)}
                  class="slider-root"
                >
                  <span class="slider-track"
                    ><Slider.Range class="slider-fill" /></span
                  >
                  <Slider.Thumb index={0} class="slider-thumb" />
                </Slider.Root>
                <input
                  type="number"
                  min="0"
                  max="255"
                  value={ch.value}
                  disabled={!conversion}
                  class="ch-input"
                  oninput={(e) =>
                    updateChannel(
                      ch.key,
                      Number((e.target as HTMLInputElement).value),
                    )}
                />
              </div>
            {/each}
          </div>
        </div>
      {:else}
        <div class="empty">无法识别颜色</div>
      {/if}
    </main>
  </AppScrollArea>
</div>

<style>
  .color-tool {
    display: flex;
    flex-direction: column;
    gap: 6px;
    padding: 4px 10px 16px 6px;
    min-height: 100%;
  }

  /* ── Swatch ── */
  .swatch {
    position: relative;
    height: 108px;
    border-radius: 12px;
    background: var(--c);
    color: var(--ct);
    overflow: hidden;
    flex-shrink: 0;
  }
  .swatch-meta {
    position: absolute;
    inset: auto 12px 10px;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }
  .swatch-meta strong {
    font-family: ui-monospace, Menlo, Monaco, Consolas, monospace;
    font-size: 20px;
    line-height: 1.1;
    letter-spacing: -0.02em;
  }
  .swatch-meta span {
    font-family: ui-monospace, Menlo, Monaco, Consolas, monospace;
    font-size: 11px;
    opacity: 0.75;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  /* ── Section ── */
  .section {
    border-radius: 10px;
    border: 1px solid rgb(229 229 229);
    background: rgba(255, 255, 255, 0.9);
    padding: 10px;
    display: flex;
    flex-direction: column;
    gap: 6px;
  }
  :global(.dark) .section {
    border-color: rgb(38 38 38);
    background: rgba(13, 13, 13, 0.92);
  }

  .section-head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 2px;
  }
  .section-head > span {
    font-size: 10.5px;
    font-weight: 700;
    color: rgb(160 160 160);
    text-transform: uppercase;
    letter-spacing: 0.06em;
  }

  /* ── Group label ── */
  .group-label {
    font-size: 10px;
    font-weight: 700;
    color: rgb(190 190 190);
    text-transform: uppercase;
    letter-spacing: 0.05em;
    margin-top: 2px;
  }
  :global(.dark) .group-label {
    color: rgb(80 80 80);
  }

  /* ── Format grid ── */
  .format-grid {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 3px;
    margin-bottom: 2px;
  }

  .fmt-cell {
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    gap: 2px;
    padding: 6px 8px;
    border-radius: 7px;
    border: 1px solid transparent;
    text-align: left;
    min-width: 0;
    transition:
      background 110ms ease,
      border-color 110ms ease;
    position: relative;
  }
  .fmt-cell:hover {
    background: rgb(246 246 246);
    border-color: rgb(228 228 228);
  }
  .fmt-cell:hover :global(.copy-icon) {
    opacity: 1;
  }
  :global(.dark) .fmt-cell:hover {
    background: rgb(20 20 20);
    border-color: rgb(40 40 40);
  }

  .fmt-label {
    font-size: 9.5px;
    font-weight: 700;
    color: rgb(170 170 170);
    letter-spacing: 0.04em;
    text-transform: uppercase;
    line-height: 1;
  }
  :global(.dark) .fmt-label {
    color: rgb(90 90 90);
  }

  .fmt-value {
    width: 100%;
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    font-family: ui-monospace, Menlo, Monaco, Consolas, monospace;
    font-size: 11.5px;
    color: rgb(25 25 25);
    line-height: 1.3;
  }
  :global(.dark) .fmt-value {
    color: rgb(205 205 205);
  }

  :global(.icon) {
    width: 12px;
    height: 12px;
    flex-shrink: 0;
  }
  :global(.copy-icon) {
    position: absolute;
    top: 6px;
    right: 7px;
    color: rgb(190 190 190);
    opacity: 0;
    transition: opacity 100ms ease;
  }

  /* ── Ghost Button ── */
  .btn-ghost {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    font-size: 11px;
    font-weight: 600;
    color: rgb(140 140 140);
    border-radius: 6px;
    padding: 3px 7px;
    transition:
      background 110ms ease,
      color 110ms ease;
  }
  .btn-ghost:hover {
    background: rgb(240 240 240);
    color: rgb(30 30 30);
  }
  :global(.dark) .btn-ghost:hover {
    background: rgb(35 35 35);
    color: rgb(210 210 210);
  }

  /* ── Channels ── */
  .channels {
    display: flex;
    flex-direction: column;
    gap: 5px;
  }

  .channel-row {
    display: flex;
    align-items: center;
    gap: 8px;
    height: 26px;
  }
  .ch-label {
    width: 14px;
    flex-shrink: 0;
    font-size: 10.5px;
    font-weight: 800;
    color: rgb(160 160 160);
  }
  :global(.dark) .ch-label {
    color: rgb(90 90 90);
  }

  :global(.slider-root) {
    position: relative;
    display: flex;
    flex: 1;
    min-width: 0;
    align-items: center;
    touch-action: none;
    user-select: none;
  }
  .slider-track {
    position: relative;
    height: 5px;
    width: 100%;
    flex-grow: 1;
    cursor: pointer;
    overflow: hidden;
    border-radius: 999px;
    background: rgb(229 229 229);
  }
  :global(.dark) .slider-track {
    background: rgb(45 45 45);
  }
  :global(.slider-fill) {
    position: absolute;
    height: 100%;
    background: rgb(30 30 30);
  }
  :global(.dark) :global(.slider-fill) {
    background: rgb(180 180 180);
  }
  :global(.slider-thumb) {
    display: block;
    width: 13px;
    height: 13px;
    cursor: pointer;
    border-radius: 999px;
    border: 1px solid rgb(210 210 210);
    background: white;
    box-shadow: 0 1px 3px rgba(0, 0, 0, 0.14);
  }
  :global(.dark) :global(.slider-thumb) {
    border-color: rgb(70 70 70);
    background: rgb(28 28 28);
  }
  :global(.slider-thumb:focus-visible) {
    outline: 2px solid rgb(23 23 23);
    outline-offset: 2px;
  }

  .ch-input {
    width: 50px;
    flex-shrink: 0;
    border: 1px solid rgb(229 229 229);
    background: rgb(250 250 250);
    border-radius: 6px;
    color: rgb(38 38 38);
    padding: 2px 5px;
    text-align: right;
    font-family: ui-monospace, Menlo, Monaco, Consolas, monospace;
    font-size: 11.5px;
    transition: border-color 100ms ease;
  }
  .ch-input:focus {
    border-color: rgb(155 155 155);
    outline: none;
  }
  :global(.dark) .ch-input {
    border-color: rgb(40 40 40);
    background: rgb(18 18 18);
    color: rgb(220 220 220);
  }
  :global(.dark) .ch-input:focus {
    border-color: rgb(75 75 75);
  }

  /* ── Empty ── */
  .empty {
    display: flex;
    align-items: center;
    justify-content: center;
    min-height: 100px;
    border-radius: 10px;
    border: 1px dashed rgb(215 215 215);
    font-size: 12.5px;
    color: rgb(170 170 170);
  }
  :global(.dark) .empty {
    border-color: rgb(45 45 45);
    color: rgb(90 90 90);
  }
</style>
