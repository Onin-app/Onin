import { marked } from "marked";

interface RepoInfo {
  type: "github" | "gitee";
  owner: string;
  repo: string;
  branch: string;
  rawBase: string;
  blobBase: string;
}

/**
 * 解析仓库 URL，获取所有者、仓库名、分支以及 Raw / Blob 的基准 URL
 */
export function parseRepoInfo(repositoryUrl: string): RepoInfo | null {
  if (!repositoryUrl) return null;

  // 清洗 URL
  let cleanUrl = repositoryUrl
    .trim()
    .replace(/^git\+/, "")
    .replace(/\.git$/, "")
    .replace(/\/$/, "");

  // 1. 尝试匹配分支信息（常见格式如 /tree/branch_name 或 /blob/branch_name）
  const branchMatch = cleanUrl.match(/\/(tree|blob)\/([^/]+)/);
  const branch = branchMatch ? branchMatch[2] : null;

  // 2. 匹配 GitHub
  const githubMatch = cleanUrl.match(/github\.com\/([^/]+)\/([^/]+)/);
  if (githubMatch) {
    const owner = githubMatch[1];
    const repo = githubMatch[2].split("/")[0]; // 去掉 /tree/... 后面的路径
    const finalBranch = branch || "main"; // 默认 main
    return {
      type: "github",
      owner,
      repo,
      branch: finalBranch,
      rawBase: `https://raw.githubusercontent.com/${owner}/${repo}/${finalBranch}/`,
      blobBase: `https://github.com/${owner}/${repo}/blob/${finalBranch}/`,
    };
  }

  // 3. 匹配 Gitee
  const giteeMatch = cleanUrl.match(/gitee\.com\/([^/]+)\/([^/]+)/);
  if (giteeMatch) {
    const owner = giteeMatch[1];
    const repo = giteeMatch[2].split("/")[0];
    const finalBranch = branch || "master"; // gitee 默认使用 master
    return {
      type: "gitee",
      owner,
      repo,
      branch: finalBranch,
      rawBase: `https://gitee.com/${owner}/${repo}/raw/${finalBranch}/`,
      blobBase: `https://gitee.com/${owner}/${repo}/blob/${finalBranch}/`,
    };
  }

  return null;
}

/**
 * 判断是否为相对路径 (排除绝对 URL 和锚点)
 */
export function isRelativePath(path: string): boolean {
  if (!path) return false;
  const trimmed = path.trim();
  return !/^(https?:|data:|mailto:|tel:|ftp:|#|\/\/)/i.test(trimmed);
}

/**
 * 转换相对路径为绝对路径
 */
export function resolveRelativePath(
  relativePath: string,
  baseUrl: string,
): string {
  const cleanPath = relativePath
    .trim()
    .replace(/^(\.\.?\/)+/, "") // 去除开头的 ./ 或 ../
    .replace(/^\//, ""); // 去除开头的 /

  return baseUrl + cleanPath;
}

/**
 * 极简的原生 DOMParser HTML 安全消毒函数，彻底防御 XSS 攻击
 */
export function sanitizeHtml(html: string): string {
  if (typeof window === "undefined") return html;
  try {
    const parser = new DOMParser();
    const doc = parser.parseFromString(html, "text/html");

    // 深度优先递归清理节点和属性
    const clean = (node: Node) => {
      if (node.nodeType === 1) {
        const el = node as Element;
        const tagName = el.tagName.toLowerCase();

        // 移除任何危险或不安全标签
        if (
          [
            "script",
            "iframe",
            "object",
            "embed",
            "form",
            "style",
            "meta",
            "link",
          ].includes(tagName)
        ) {
          el.remove();
          return;
        }

        // 移除所有 inline 的事件监听属性 (on*) 以及 javascript: 伪协议
        const attrs = Array.from(el.attributes);
        for (const attr of attrs) {
          const attrName = attr.name.toLowerCase();
          if (attrName.startsWith("on")) {
            el.removeAttribute(attr.name);
          }
          if (
            ["src", "href", "data"].includes(attrName) &&
            (attr.value.toLowerCase().trim().startsWith("javascript:") ||
              attr.value.toLowerCase().trim().startsWith("data:"))
          ) {
            el.removeAttribute(attr.name);
          }
        }
      }

      const children = Array.from(node.childNodes);
      for (const child of children) {
        clean(child);
      }
    };

    if (doc.body) {
      clean(doc.body);
      return doc.body.innerHTML;
    }
    return html;
  } catch (e) {
    console.error("HTML 消毒失败:", e);
    return html;
  }
}

/**
 * 遍历并改写 HTML 中的相对路径为绝对路径
 */
export function resolveHtmlPaths(html: string, repositoryUrl?: string): string {
  if (!repositoryUrl || typeof window === "undefined") return html;

  const repoInfo = parseRepoInfo(repositoryUrl);
  if (!repoInfo) return html;

  try {
    const parser = new DOMParser();
    const doc = parser.parseFromString(html, "text/html");

    // 1. 处理图片 (img) 的相对路径 src
    const imgs = doc.querySelectorAll("img");
    imgs.forEach((img) => {
      const src = img.getAttribute("src");
      if (src && isRelativePath(src)) {
        img.setAttribute("src", resolveRelativePath(src, repoInfo.rawBase));
      }
    });

    // 2. 处理链接 (a) 的相对路径 href
    const links = doc.querySelectorAll("a");
    links.forEach((a) => {
      const href = a.getAttribute("href");
      if (href && isRelativePath(href)) {
        a.setAttribute("href", resolveRelativePath(href, repoInfo.blobBase));
      }
    });

    if (doc.body) {
      return doc.body.innerHTML;
    }
    return html;
  } catch (e) {
    console.error("Failed to resolve HTML paths:", e);
    return html;
  }
}

/**
 * Markdown 渲染总入口：解析 -> 路径补全 -> XSS 消毒
 */
export function renderMarkdown(
  markdown: string,
  repositoryUrl?: string,
): string {
  if (!markdown) return "";
  try {
    // 1. 解析 Markdown 为 HTML
    const rawHtml = marked.parse(markdown, { async: false }) as string;

    // 2. 补全其中的相对路径为仓库绝对路径
    const resolvedHtml = resolveHtmlPaths(rawHtml, repositoryUrl);

    // 3. 安全消毒
    return sanitizeHtml(resolvedHtml);
  } catch (e) {
    console.error("Failed to render markdown:", e);
    return markdown;
  }
}

/**
 * Svelte Action: 在捕获阶段监听容器内所有图片的加载错误，并尝试对默认分支进行 fallback 切换（main <-> master）
 */
export function setupImageFallback(container: HTMLElement) {
  if (!container) return;

  const handleError = (event: Event) => {
    const target = event.target as HTMLElement;
    if (target && target.tagName.toLowerCase() === "img") {
      const img = target as HTMLImageElement;
      const src = img.src;

      // 如果已经尝试过 fallback，避免死循环
      if (img.dataset.fallbackAttempted) return;

      // 仅对 GitHub 或 Gitee 的 Raw 域名及图片路径进行 fallback
      if (
        src.includes("raw.githubusercontent.com") ||
        src.includes("gitee.com") ||
        src.includes("githubusercontent.com")
      ) {
        img.dataset.fallbackAttempted = "true";
        if (src.includes("/main/")) {
          img.src = src.replace("/main/", "/master/");
        } else if (src.includes("/master/")) {
          img.src = src.replace("/master/", "/main/");
        }
      }
    }
  };

  // 在捕获阶段（useCapture = true）监听，因为 error 事件在 img 元素上不冒泡
  container.addEventListener("error", handleError, true);

  return {
    destroy() {
      container.removeEventListener("error", handleError, true);
    },
  };
}
