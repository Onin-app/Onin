<script lang="ts">
  interface Props {
    fileType: string;
    fileName: string;
    class?: string;
  }

  let { fileType, fileName, class: className = "size-6" }: Props = $props();

  // 获取文件扩展名
  const getExtension = (name: string) => {
    const parts = name.split(".");
    return parts.length > 1 ? parts[parts.length - 1].toLowerCase() : "";
  };

  // 使用 $derived 使变量响应式，正确追踪 props 变化
  const ext = $derived(getExtension(fileName));
  const isFolder = $derived(!ext || fileName.endsWith("/")); // 文件夹通常没有扩展名或以/结尾
  const isImage = $derived(fileType.startsWith("image/"));
  const isVideo = $derived(fileType.startsWith("video/"));
  const isAudio = $derived(fileType.startsWith("audio/"));
  const isPdf = $derived(fileType === "application/pdf" || ext === "pdf");
  const isWord = $derived(
    fileType.includes("word") || ["doc", "docx"].includes(ext),
  );
  const isExcel = $derived(
    fileType.includes("excel") || ["xls", "xlsx"].includes(ext),
  );
  const isPowerPoint = $derived(
    fileType.includes("presentation") || ["ppt", "pptx"].includes(ext),
  );
  const isZip = $derived(
    fileType.includes("zip") ||
      fileType.includes("rar") ||
      ["zip", "rar", "7z", "tar", "gz", "bz2", "xz"].includes(ext),
  );
  const isCode = $derived(
    [
      "js",
      "ts",
      "jsx",
      "tsx",
      "py",
      "java",
      "cpp",
      "c",
      "h",
      "css",
      "html",
      "json",
      "xml",
      "rs",
      "go",
      "php",
      "rb",
      "swift",
      "kt",
      "dart",
      "vue",
      "svelte",
    ].includes(ext),
  );
  const isExecutable = $derived(
    ["exe", "app", "dmg", "msi", "apk", "deb", "rpm"].includes(ext),
  );
  const isFont = $derived(["ttf", "otf", "woff", "woff2", "eot"].includes(ext));
  const isMarkdown = $derived(["md", "markdown"].includes(ext));
  const isSvg = $derived(ext === "svg");
  const isText = $derived(fileType.startsWith("text/") || ext === "txt");
</script>

{#if isFolder}
  <!-- 文件夹图标 -->
  <svg
    class={className}
    viewBox="0 0 24 24"
    fill="none"
    xmlns="http://www.w3.org/2000/svg"
  >
    <rect x="3" y="3" width="18" height="18" rx="2" fill="#FCD34D" />
    <path
      d="M6 8C6 7.44772 6.44772 7 7 7H10L11 9H17C17.5523 9 18 9.44772 18 10V16C18 16.5523 17.5523 17 17 17H7C6.44772 17 6 16.5523 6 16V8Z"
      fill="white"
    />
  </svg>
{:else if isExecutable}
  <!-- 可执行文件图标 -->
  <svg
    class={className}
    viewBox="0 0 24 24"
    fill="none"
    xmlns="http://www.w3.org/2000/svg"
  >
    <rect x="3" y="3" width="18" height="18" rx="2" fill="#7C3AED" />
    <path d="M8 8H16V10H8V8Z" fill="white" />
    <path d="M8 11H16V13H8V11Z" fill="white" opacity="0.7" />
    <path d="M8 14H13V16H8V14Z" fill="white" opacity="0.5" />
    <circle cx="16" cy="15" r="2" fill="#22C55E" />
  </svg>
{:else if isFont}
  <!-- 字体文件图标 -->
  <svg
    class={className}
    viewBox="0 0 24 24"
    fill="none"
    xmlns="http://www.w3.org/2000/svg"
  >
    <rect x="3" y="3" width="18" height="18" rx="2" fill="#64748B" />
    <text
      x="12"
      y="17"
      font-family="serif"
      font-size="12"
      font-weight="bold"
      fill="white"
      text-anchor="middle"
    >
      Aa
    </text>
  </svg>
{:else if isMarkdown}
  <!-- Markdown 图标 -->
  <svg
    class={className}
    viewBox="0 0 24 24"
    fill="none"
    xmlns="http://www.w3.org/2000/svg"
  >
    <rect x="3" y="3" width="18" height="18" rx="2" fill="#374151" />
    <path
      d="M7 14V10L9 12L11 10V14"
      stroke="white"
      stroke-width="1.5"
      stroke-linecap="round"
      stroke-linejoin="round"
    />
    <path
      d="M14 14L16 12L14 10"
      stroke="white"
      stroke-width="1.5"
      stroke-linecap="round"
      stroke-linejoin="round"
    />
  </svg>
{:else if isSvg}
  <!-- SVG 图标 -->
  <svg
    class={className}
    viewBox="0 0 24 24"
    fill="none"
    xmlns="http://www.w3.org/2000/svg"
  >
    <rect x="3" y="3" width="18" height="18" rx="2" fill="#FB923C" />
    <text
      x="12"
      y="15"
      font-family="Arial, sans-serif"
      font-size="7"
      font-weight="bold"
      fill="white"
      text-anchor="middle"
    >
      SVG
    </text>
  </svg>
{:else if isImage}
  <!-- 图片图标 -->
  <svg
    class={className}
    viewBox="0 0 24 24"
    fill="none"
    xmlns="http://www.w3.org/2000/svg"
  >
    <rect x="3" y="3" width="18" height="18" rx="2" fill="#10B981" />
    <path
      d="M8 10C8.55228 10 9 9.55228 9 9C9 8.44772 8.55228 8 8 8C7.44772 8 7 8.44772 7 9C7 9.55228 7.44772 10 8 10Z"
      fill="white"
    />
    <path
      d="M7 16L10 13L13 16L17 12L19 14V18C19 18.5523 18.5523 19 18 19H6C5.44772 19 5 18.5523 5 18V16H7Z"
      fill="white"
      opacity="0.8"
    />
  </svg>
{:else if isVideo}
  <!-- 视频图标 -->
  <svg
    class={className}
    viewBox="0 0 24 24"
    fill="none"
    xmlns="http://www.w3.org/2000/svg"
  >
    <rect x="3" y="3" width="18" height="18" rx="2" fill="#8B5CF6" />
    <path
      d="M10 8.5V15.5L16 12L10 8.5Z"
      fill="white"
      stroke="white"
      stroke-width="1.5"
      stroke-linejoin="round"
    />
  </svg>
{:else if isAudio}
  <!-- 音频图标 -->
  <svg
    class={className}
    viewBox="0 0 24 24"
    fill="none"
    xmlns="http://www.w3.org/2000/svg"
  >
    <rect x="3" y="3" width="18" height="18" rx="2" fill="#EC4899" />
    <path
      d="M15 7V14C15 15.1046 14.1046 16 13 16C11.8954 16 11 15.1046 11 14C11 12.8954 11.8954 12 13 12C13.3506 12 13.6872 12.0602 14 12.1707V9L15 7Z"
      fill="white"
    />
    <path d="M15 7L17 8V11L15 9V7Z" fill="white" opacity="0.7" />
  </svg>
{:else if isPdf}
  <!-- PDF 图标 -->
  <svg
    class={className}
    viewBox="0 0 24 24"
    fill="none"
    xmlns="http://www.w3.org/2000/svg"
  >
    <rect x="3" y="3" width="18" height="18" rx="2" fill="#EF4444" />
    <text
      x="12"
      y="15"
      font-family="Arial, sans-serif"
      font-size="8"
      font-weight="bold"
      fill="white"
      text-anchor="middle"
    >
      PDF
    </text>
  </svg>
{:else if isWord}
  <!-- Word 图标 -->
  <svg
    class={className}
    viewBox="0 0 24 24"
    fill="none"
    xmlns="http://www.w3.org/2000/svg"
  >
    <rect x="3" y="3" width="18" height="18" rx="2" fill="#2563EB" />
    <text
      x="12"
      y="15"
      font-family="Arial, sans-serif"
      font-size="8"
      font-weight="bold"
      fill="white"
      text-anchor="middle"
    >
      DOC
    </text>
  </svg>
{:else if isExcel}
  <!-- Excel 图标 -->
  <svg
    class={className}
    viewBox="0 0 24 24"
    fill="none"
    xmlns="http://www.w3.org/2000/svg"
  >
    <rect x="3" y="3" width="18" height="18" rx="2" fill="#10B981" />
    <text
      x="12"
      y="15"
      font-family="Arial, sans-serif"
      font-size="8"
      font-weight="bold"
      fill="white"
      text-anchor="middle"
    >
      XLS
    </text>
  </svg>
{:else if isPowerPoint}
  <!-- PowerPoint 图标 -->
  <svg
    class={className}
    viewBox="0 0 24 24"
    fill="none"
    xmlns="http://www.w3.org/2000/svg"
  >
    <rect x="3" y="3" width="18" height="18" rx="2" fill="#F97316" />
    <text
      x="12"
      y="15"
      font-family="Arial, sans-serif"
      font-size="8"
      font-weight="bold"
      fill="white"
      text-anchor="middle"
    >
      PPT
    </text>
  </svg>
{:else if isZip}
  <!-- 压缩包图标 -->
  <svg
    class={className}
    viewBox="0 0 24 24"
    fill="none"
    xmlns="http://www.w3.org/2000/svg"
  >
    <rect x="3" y="3" width="18" height="18" rx="2" fill="#F59E0B" />
    <rect x="11" y="6" width="2" height="2" fill="white" />
    <rect x="11" y="9" width="2" height="2" fill="white" opacity="0.7" />
    <rect x="11" y="12" width="2" height="2" fill="white" />
    <path
      d="M10 15H14V17C14 17.5523 13.5523 18 13 18H11C10.4477 18 10 17.5523 10 17V15Z"
      fill="white"
      opacity="0.8"
    />
  </svg>
{:else if isCode}
  <!-- 代码文件图标 -->
  <svg
    class={className}
    viewBox="0 0 24 24"
    fill="none"
    xmlns="http://www.w3.org/2000/svg"
  >
    <rect x="3" y="3" width="18" height="18" rx="2" fill="#06B6D4" />
    <path
      d="M9 10L7 12L9 14"
      stroke="white"
      stroke-width="1.5"
      stroke-linecap="round"
      stroke-linejoin="round"
    />
    <path
      d="M15 10L17 12L15 14"
      stroke="white"
      stroke-width="1.5"
      stroke-linecap="round"
      stroke-linejoin="round"
    />
    <path
      d="M13 9L11 15"
      stroke="white"
      stroke-width="1.5"
      stroke-linecap="round"
    />
  </svg>
{:else if isText}
  <!-- 文本文件图标 -->
  <svg
    class={className}
    viewBox="0 0 24 24"
    fill="none"
    xmlns="http://www.w3.org/2000/svg"
  >
    <rect x="3" y="3" width="18" height="18" rx="2" fill="#6B7280" />
    <line
      x1="7"
      y1="9"
      x2="17"
      y2="9"
      stroke="white"
      stroke-width="1.5"
      stroke-linecap="round"
    />
    <line
      x1="7"
      y1="12"
      x2="17"
      y2="12"
      stroke="white"
      stroke-width="1.5"
      stroke-linecap="round"
    />
    <line
      x1="7"
      y1="15"
      x2="13"
      y2="15"
      stroke="white"
      stroke-width="1.5"
      stroke-linecap="round"
    />
  </svg>
{:else}
  <!-- 默认文件图标 -->
  <svg
    class={className}
    viewBox="0 0 24 24"
    fill="none"
    xmlns="http://www.w3.org/2000/svg"
  >
    <rect x="3" y="3" width="18" height="18" rx="2" fill="#9CA3AF" />
    <path
      d="M8 3H13L16 6V8H8V3Z"
      fill="white"
      opacity="0.3"
      fill-rule="evenodd"
      clip-rule="evenodd"
    />
    <circle cx="12" cy="14" r="3" stroke="white" stroke-width="1.5" />
  </svg>
{/if}
