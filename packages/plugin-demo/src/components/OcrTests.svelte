<script lang="ts">
  import { getContext } from 'svelte';
  import { ocr, clipboard, toast } from 'onin-sdk';
  import type { LoggerState } from '../state/logger.svelte';

  const logger = getContext<LoggerState>('logger');

  // OCR 测试相关的状态
  let ocrImageSrc = $state<string>(''); // 用于预览的图片源 (base64 或 相对/绝对路径)
  let ocrResult = $state<any>(null); // 保存 OCR 识别的结构化结果
  let ocrLang = $state<string>(''); // OCR 识别的语言参数，比如 'zh-CN' 或 'en-US'
  let ocrLoading = $state<boolean>(false); // 识别加载状态

  // 用于计算缩放后 OCR 区域的框
  let imgWidth = $state<number>(0);
  let imgHeight = $state<number>(0);
  let naturalWidth = $state<number>(1);
  let naturalHeight = $state<number>(1);

  // 缩放比例
  let scaleX = $derived(imgWidth / naturalWidth);
  let scaleY = $derived(imgHeight / naturalHeight);

  // 当图片加载完成时获取真实尺寸与当前尺寸
  function handleImageLoad(e: Event) {
    const img = e.currentTarget as HTMLImageElement;
    naturalWidth = img.naturalWidth || 1;
    naturalHeight = img.naturalHeight || 1;
    imgWidth = img.clientWidth;
    imgHeight = img.clientHeight;
  }

  // 执行 OCR 识别
  async function runOcr(imageSource: string) {
    if (!imageSource) {
      logger.log('✗ 识别失败: 没有可用的图片源', 'error');
      return;
    }
    ocrLoading = true;
    ocrResult = null;
    logger.log('⏳ 正在执行 OCR 识别...');
    try {
      const options = ocrLang ? { language: ocrLang } : undefined;
      const startTime = Date.now();
      const result = await ocr.recognize(imageSource, options);
      const duration = Date.now() - startTime;
      ocrResult = result;
      logger.log(`✓ OCR 识别成功 (耗时 ${duration}ms):`, 'success');
      logger.log(`识别文本:\n${result.text}`, 'success');
      logger.log(`检测到 ${result.lines?.length || 0} 行文本`);
    } catch (err: any) {
      logger.log(`✗ OCR 识别失败: ${err?.message || err}`, 'error');
    } finally {
      ocrLoading = false;
    }
  }

  // 从剪贴板读取图片并识别
  async function runOcrFromClipboard() {
    logger.log('⏳ 正在从剪贴板读取图片...');
    try {
      const hasImg = await clipboard.hasImage();
      if (!hasImg) {
        logger.log('✗ 剪贴板中没有图片', 'error');
        toast.error('剪贴板中没有图片');
        return;
      }
      const base64Data = await clipboard.readImage();
      if (!base64Data) {
        logger.log('✗ 未能成功读取剪贴板图片', 'error');
        return;
      }
      ocrImageSrc = `data:image/png;base64,${base64Data}`;
      await runOcr(ocrImageSrc);
    } catch (err: any) {
      logger.log(`✗ 读取剪贴板失败: ${err?.message || err}`, 'error');
    }
  }

  // 通过选择本地文件上传识别
  function handleFileChange(event: Event) {
    const target = event.target as HTMLInputElement;
    const file = target.files?.[0];
    if (!file) return;

    logger.log(`⏳ 正在读取本地图片 ${file.name}...`);
    const reader = new FileReader();
    reader.onload = async (e) => {
      const base64Data = e.target?.result as string;
      ocrImageSrc = base64Data;
      await runOcr(ocrImageSrc);
    };
    reader.onerror = (err) => {
      logger.log(`✗ 读取本地图片失败: ${err}`, 'error');
    };
    reader.readAsDataURL(file);
  }

  // 手动输入路径识别
  let manualPath = $state<string>('');
  async function runOcrFromManualPath() {
    if (!manualPath.trim()) {
      toast.warning('请输入图片绝对路径');
      return;
    }
    ocrImageSrc = ''; // 清除预览
    logger.log(`⏳ 正在识别绝对路径图片: ${manualPath}`);
    await runOcr(manualPath.trim());
  }
</script>

<div class="api-group">
  <h3>OCR 参数配置</h3>
  <div class="ocr-config">
    <div class="input-group">
      <label for="ocr-lang"
        >识别语言 (可选 BCP-47，例如 zh-CN, en-US, ja-JP):</label
      >
      <input
        id="ocr-lang"
        type="text"
        bind:value={ocrLang}
        placeholder="留空则使用系统默认语言"
        class="ocr-input"
      />
    </div>
  </div>
</div>

<div class="api-group">
  <h3>识别操作</h3>
  <div class="test-grid">
    <button
      class="test-btn"
      onclick={runOcrFromClipboard}
      disabled={ocrLoading}
    >
      <span class="api-name">Clipboard Image</span>
      <span class="api-desc">从剪贴板读取图片并识别</span>
    </button>

    <label class="test-btn upload-btn" class:disabled={ocrLoading}>
      <span class="api-name">Upload Image</span>
      <span class="api-desc">上传本地图片文件并识别</span>
      <input
        type="file"
        accept="image/*"
        onchange={handleFileChange}
        disabled={ocrLoading}
        style="display: none;"
      />
    </label>
  </div>

  <div class="manual-path-box">
    <input
      type="text"
      bind:value={manualPath}
      placeholder="输入本地图片绝对路径，例如 C:\path\to\image.png"
      class="ocr-input"
      disabled={ocrLoading}
    />
    <button
      class="btn btn-primary"
      onclick={runOcrFromManualPath}
      disabled={ocrLoading || !manualPath.trim()}
    >
      路径识别
    </button>
  </div>
</div>

{#if ocrImageSrc || ocrResult || ocrLoading}
  <div class="api-group">
    <h3>可视化结果</h3>
    <div class="ocr-visualizer">
      <!-- 左侧：图片预览与 Overlay -->
      <div class="ocr-preview-container">
        {#if ocrLoading}
          <div class="ocr-loading-overlay">
            <span class="spinner"></span>
            <span>识别中...</span>
          </div>
        {/if}

        {#if ocrImageSrc}
          <div class="image-wrapper">
            <img
              src={ocrImageSrc}
              alt="OCR 预览"
              onload={handleImageLoad}
              bind:clientWidth={imgWidth}
              bind:clientHeight={imgHeight}
              class="ocr-img"
            />

            <!-- 标注 Overlay 层 -->
            {#if ocrResult && ocrResult.lines}
              <div class="ocr-overlay">
                {#each ocrResult.lines as line}
                  <div
                    class="ocr-line-box"
                    role="button"
                    tabindex="0"
                    style="
                      left: {line.x * scaleX}px;
                      top: {line.y * scaleY}px;
                      width: {line.width * scaleX}px;
                      height: {line.height * scaleY}px;
                    "
                    title={line.text}
                    onclick={() => {
                      clipboard.writeText(line.text);
                      toast.success('已复制该行文本');
                    }}
                    onkeydown={(e) => {
                      if (e.key === 'Enter' || e.key === ' ') {
                        clipboard.writeText(line.text);
                        toast.success('已复制该行文本');
                      }
                    }}
                  >
                    <span class="tooltip-text">{line.text}</span>
                  </div>
                {/each}
              </div>
            {/if}
          </div>
        {:else if ocrLoading}
          <div class="ocr-placeholder">正在加载并识别图像...</div>
        {:else}
          <div class="ocr-placeholder">
            本地路径识别暂不支持在界面中直接预览，请在右侧或日志面板查看结果。
          </div>
        {/if}
      </div>

      <!-- 右侧：纯文本输出 -->
      <div class="ocr-text-result">
        <div class="result-header">
          <h4>文本结果</h4>
          {#if ocrResult}
            <button
              class="btn btn-secondary btn-xs"
              onclick={() => {
                clipboard.writeText(ocrResult.text);
                toast.success('完整文本已复制');
              }}
            >
              📋 复制全部
            </button>
          {/if}
        </div>
        <div class="result-content">
          {#if ocrResult}
            <pre>{ocrResult.text}</pre>
          {:else}
            <span class="muted-text">暂无识别数据</span>
          {/if}
        </div>
      </div>
    </div>
  </div>
{/if}

<style>
  /* OCR 测试相关样式 */
  .ocr-config {
    background: var(--bg-secondary);
    padding: 16px;
    border-radius: var(--radius);
    border: 1px solid var(--border);
    margin-bottom: 12px;
  }

  .input-group {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .input-group label {
    font-size: 12px;
    color: var(--text-secondary);
  }

  .ocr-input {
    background: var(--bg-tertiary);
    border: 1px solid var(--border);
    color: var(--text-primary);
    padding: 8px 12px;
    border-radius: var(--radius);
    outline: none;
    font-size: 13px;
    transition: border-color 0.15s;
  }

  .ocr-input:focus {
    border-color: var(--accent);
  }

  .upload-btn {
    cursor: pointer;
    position: relative;
  }

  .upload-btn.disabled {
    opacity: 0.6;
    pointer-events: none;
  }

  .manual-path-box {
    display: flex;
    gap: 8px;
    margin-top: 12px;
  }

  .manual-path-box .ocr-input {
    flex: 1;
  }

  .ocr-visualizer {
    display: flex;
    gap: 16px;
    background: var(--bg-secondary);
    border: 1px solid var(--border);
    border-radius: var(--radius);
    padding: 16px;
    min-height: 300px;
  }

  .ocr-preview-container {
    flex: 1;
    position: relative;
    background: var(--bg-primary);
    border: 1px solid var(--border);
    border-radius: var(--radius);
    display: flex;
    align-items: center;
    justify-content: center;
    min-height: 250px;
    overflow: hidden;
  }

  .ocr-loading-overlay {
    position: absolute;
    inset: 0;
    background: rgba(0, 0, 0, 0.7);
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 12px;
    z-index: 10;
  }

  .spinner {
    width: 24px;
    height: 24px;
    border: 3px solid rgba(255, 255, 255, 0.1);
    border-top: 3px solid var(--accent);
    border-radius: 50%;
    animation: ocr-spin 1s linear infinite;
  }

  @keyframes ocr-spin {
    0% {
      transform: rotate(0deg);
    }
    100% {
      transform: rotate(360deg);
    }
  }

  .image-wrapper {
    position: relative;
    max-width: 100%;
    max-height: 450px;
    display: inline-block;
  }

  .ocr-img {
    max-width: 100%;
    max-height: 450px;
    display: block;
    object-fit: contain;
  }

  .ocr-overlay {
    position: absolute;
    top: 0;
    left: 0;
    width: 100%;
    height: 100%;
    pointer-events: auto; /* 必须允许 hover 和 click 事件 */
  }

  .ocr-line-box {
    position: absolute;
    border: 1px dashed rgba(99, 102, 241, 0.5);
    background: rgba(99, 102, 241, 0.1);
    cursor: pointer;
    transition: all 0.15s ease-in-out;
  }

  .ocr-line-box:hover {
    border-color: var(--accent);
    background: rgba(99, 102, 241, 0.35);
    box-shadow: 0 0 8px rgba(99, 102, 241, 0.5);
    z-index: 5;
  }

  /* Tooltip 气泡提示 */
  .ocr-line-box .tooltip-text {
    visibility: hidden;
    background-color: #1e1b4b;
    color: #fff;
    text-align: center;
    border: 1px solid var(--accent);
    border-radius: 4px;
    padding: 4px 8px;
    position: absolute;
    z-index: 20;
    bottom: 125%; /* 定位在框框的上方 */
    left: 50%;
    transform: translateX(-50%);
    opacity: 0;
    transition: opacity 0.15s;
    font-size: 11px;
    white-space: nowrap;
    pointer-events: none;
    box-shadow: 0 4px 6px -1px rgba(0, 0, 0, 0.5);
  }

  .ocr-line-box:hover .tooltip-text {
    visibility: visible;
    opacity: 1;
  }

  .ocr-placeholder {
    color: var(--text-muted);
    font-size: 13px;
    padding: 24px;
    text-align: center;
  }

  .ocr-text-result {
    width: 250px;
    border-left: 1px solid var(--border);
    padding-left: 16px;
    display: flex;
    flex-direction: column;
  }

  .ocr-text-result h4 {
    font-size: 14px;
    margin: 0 0 12px 0;
    font-weight: 500;
  }

  .result-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 8px;
  }

  .result-header h4 {
    margin: 0;
  }

  .result-content {
    flex: 1;
    background: var(--bg-primary);
    border: 1px solid var(--border);
    border-radius: var(--radius);
    padding: 8px;
    overflow-y: auto;
    font-family: 'JetBrains Mono', monospace;
    font-size: 12px;
    max-height: 400px;
  }

  .result-content pre {
    margin: 0;
    white-space: pre-wrap;
    word-break: break-all;
  }

  .muted-text {
    color: var(--text-muted);
  }
</style>
