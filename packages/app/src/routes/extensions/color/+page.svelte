<script lang="ts">
  import { onMount } from "svelte";
  import { goto } from "$app/navigation";
  import { page } from "$app/stores";
  import { invoke } from "@tauri-apps/api/core";
  import { listen, type UnlistenFn } from "@tauri-apps/api/event";
  import { toast } from "svelte-sonner";
  import ColorPicker, { ChromeVariant } from "svelte-awesome-color-picker";
  import { Copy, ClipboardText, Eyedropper } from "phosphor-svelte";
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
  let pickerHex = $state("#ff5500");
  let headerRef: ExtensionHeader;
  let loadRequestId = 0;
  let isPicking = $state(false);

  const initialQuery = $derived($page.url.searchParams.get("q") || "#ff5500");
  const colorValue = $derived(conversion?.hex.slice(0, 7) || "#ff5500");
  const swatches = [
    "#EF4444",
    "#F97316",
    "#F59E0B",
    "#22C55E",
    "#14B8A6",
    "#0EA5E9",
    "#6366F1",
    "#A855F7",
    "#EC4899",
    "#111827",
    "#6B7280",
    "#F9FAFB",
  ];

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

  // ── Actions ────────────────────────────────────────────────────────────────
  async function loadColor(value: string, syncPicker = true) {
    const requestId = ++loadRequestId;
    query = value;
    try {
      const next = await invoke<ColorConversion | null>(
        "get_color_conversion",
        { input: value },
      );
      if (requestId !== loadRequestId) return;
      conversion = next;
      if (next && syncPicker) {
        pickerHex = next.hex;
      }
    } catch {
      if (requestId !== loadRequestId) return;
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

  function handlePickerInput(event: { hex: string | null }) {
    if (!event.hex) return;
    loadColor(event.hex, false);
  }

  async function startPicker() {
    if (isPicking) return;
    isPicking = true;

    try {
      await invoke("start_color_picker");
    } catch (error) {
      isPicking = false;
      const message = error instanceof Error ? error.message : String(error);
      toast.error(message || "取色失败");
    }
  }

  onMount(() => {
    loadColor(initialQuery);
    headerRef?.focus();

    let unlisten: UnlistenFn | undefined;

    // 监听 Rust 广播的取色结果
    listen<ColorConversion | null>("color_picker_result", (event) => {
      isPicking = false;

      if (!event.payload) {
        toast.info("已取消取色");
        return;
      }

      loadColor(event.payload.hex);
      toast.success(`${event.payload.hex} 已取色`);
    }).then((fn) => {
      unlisten = fn;
    });

    return () => {
      unlisten?.();
    };
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

        <!-- Color picker -->
        <div class="section picker-section">
          <div class="section-head">
            <span>色盘</span>
            <button
              class="btn-ghost"
              onclick={startPicker}
              disabled={isPicking}
              title="从屏幕任意位置取色"
            >
              <Eyedropper class="icon" />
              {isPicking ? "取色中…" : "取色"}
            </button>
          </div>
          <div class="picker-frame">
            <ColorPicker
              bind:hex={pickerHex}
              components={ChromeVariant}
              sliderDirection="horizontal"
              isDialog={false}
              isAlpha={true}
              isTextInput={false}
              {swatches}
              onInput={handlePickerInput}
              texts={{
                label: {
                  h: "色相",
                  s: "饱和度",
                  v: "明度",
                  r: "红",
                  g: "绿",
                  b: "蓝",
                  a: "透明度",
                  hex: "十六进制颜色",
                  withoutColor: "无颜色",
                },
                color: {
                  rgb: "RGB",
                  hsv: "HSV",
                  hex: "HEX",
                },
                changeTo: "切换到 ",
              }}
            />
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

  .picker-section {
    padding-bottom: 12px;
  }

  .picker-frame {
    min-width: 0;
    overflow: hidden;
    border-radius: 8px;
    --picker-width: 100%;
    --picker-height: 148px;
    --slider-width: 16px;
    --picker-indicator-size: 14px;
    --focus-color: rgb(23 23 23);
    --cp-bg-color: transparent;
    --cp-border-color: transparent;
    --cp-text-color: rgb(38 38 38);
    --cp-input-color: rgb(245 245 245);
    --cp-button-hover-color: rgb(238 238 238);
    --cp-swatch-grid-template-columns: repeat(12, minmax(0, 1fr));
  }

  :global(.dark) .picker-frame {
    --focus-color: rgb(245 245 245);
    --cp-text-color: rgb(220 220 220);
    --cp-input-color: rgb(24 24 24);
    --cp-button-hover-color: rgb(35 35 35);
  }

  .picker-frame :global(input[type="color"]) {
    width: 100%;
  }

  .picker-frame :global(button) {
    cursor: pointer;
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
  .picker-actions {
    display: inline-flex;
    align-items: center;
    gap: 4px;
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
  .btn-ghost:disabled {
    cursor: default;
    opacity: 0.55;
  }
  .btn-ghost:disabled:hover {
    background: transparent;
    color: rgb(140 140 140);
  }
  :global(.dark) .btn-ghost:hover {
    background: rgb(35 35 35);
    color: rgb(210 210 210);
  }
  :global(.dark) .btn-ghost:disabled:hover {
    background: transparent;
    color: rgb(140 140 140);
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
