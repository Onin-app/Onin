/**
 * Clipboard Manager Composable
 *
 * 管理剪贴板内容和文件附件的状态和逻辑
 * 遵循单一职责原则，只处理剪贴板/附件相关功能
 */
import { invoke } from "@tauri-apps/api/core";
import { inferMimeType } from "$lib/utils/mimeTypeMap";
import type { AppConfig, LaunchableItem } from "$lib/type";
import { getMatchedCommands } from "$lib/utils/matchCommand";

export interface ClipboardState {
  attachedFiles: File[];
  attachedText: string;
  showAllFiles: boolean;
}

export interface ClipboardManagerReturn {
  // State (reactive)
  state: ClipboardState;
  // Methods
  handlePaste: (e: ClipboardEvent) => Promise<void>;
  handleDrop: (e: DragEvent) => void;
  handleDragOver: (e: DragEvent) => void;
  autoPasteClipboard: () => Promise<void>;
  clearAttachments: () => void;
  removeFile: (index: number) => void;
  editTextAttachment: (onEdit: (text: string) => void) => void;
  toggleShowAllFiles: () => void;
  getMatchedCommands: (originAppList: LaunchableItem[], inputText?: string) => LaunchableItem[];
}

/**
 * 创建剪贴板管理器
 *
 * 使用 Svelte 5 runes 管理剪贴板状态
 */
export function useClipboardManager(): ClipboardManagerReturn {
  // ===== State =====
  let state = $state<ClipboardState>({
    attachedFiles: [],
    attachedText: "",
    showAllFiles: false,
  });

  // ===== Methods =====

  /**
   * 处理粘贴事件
   * 同时处理文件和文本，使手动粘贴的样式与自动粘贴一致
   */
  const handlePaste = async (e: ClipboardEvent) => {
    const items = e.clipboardData?.items;
    if (!items || items.length === 0) return;

    // 先检查是否有我们需要处理的内容类型
    let hasFile = false;
    let hasText = false;
    for (let i = 0; i < items.length; i++) {
      if (items[i].kind === "file") hasFile = true;
      if (items[i].kind === "string" && items[i].type === "text/plain")
        hasText = true;
    }

    // 如果有需要处理的内容，立即阻止默认行为
    if (hasFile || hasText) {
      e.preventDefault();
    } else {
      return;
    }

    const files: File[] = [];
    let pastedText = "";

    for (let i = 0; i < items.length; i++) {
      const item = items[i];
      if (item.kind === "file") {
        const file = item.getAsFile();
        if (file) {
          files.push(file);
        }
      } else if (item.kind === "string" && item.type === "text/plain") {
        // 获取粘贴的文本内容
        pastedText = await new Promise<string>((resolve) => {
          item.getAsString(resolve);
        });
      }
    }

    // 优先处理文件
    if (files.length > 0) {
      state.attachedText = "";
      state.attachedFiles = [...state.attachedFiles, ...files];
    }
    // 如果没有文件但有文本，将文本存储到 attachedText 中
    else if (pastedText) {
      state.attachedFiles = [];
      state.showAllFiles = false;
      state.attachedText = pastedText.trim();
    }
  };

  /**
   * 处理文件拖放
   */
  const handleDrop = (e: DragEvent) => {
    e.preventDefault();
    const files = Array.from(e.dataTransfer?.files || []);
    if (files.length > 0) {
      state.attachedFiles = [...state.attachedFiles, ...files];
      console.log("Dropped files:", state.attachedFiles);
    }
  };

  /**
   * 处理拖放悬停
   */
  const handleDragOver = (e: DragEvent) => {
    e.preventDefault();
  };

  /**
   * 自动粘贴剪贴板内容
   */
  const autoPasteClipboard = async () => {
    try {
      const clipboardContent = await invoke<{
        text?: string;
        files?: Array<{ path: string; name: string; is_directory: boolean }>;
        timestamp?: number;
      }>("get_clipboard_content");

      // 获取配置的时间限制
      const config = await invoke<{ auto_paste_time_limit: number }>(
        "get_app_config",
      );
      const timeLimit = config.auto_paste_time_limit;

      console.log("Auto paste config:", {
        timeLimit,
        timestamp: clipboardContent.timestamp,
        fullConfig: config,
      });

      // 如果设置了时间限制（不为0），检查剪贴板内容的时间
      if (timeLimit > 0) {
        if (!clipboardContent.timestamp) {
          console.log("No timestamp available, skipping auto-paste");
          return;
        }

        const clipboardTimestamp = clipboardContent.timestamp;
        const currentTime = Math.floor(Date.now() / 1000);
        const timeDiff = currentTime - clipboardTimestamp;

        console.log("Time check:", {
          clipboardTimestamp,
          currentTime,
          timeDiff,
          timeLimit,
          shouldPaste: timeDiff <= timeLimit,
        });

        if (timeDiff > timeLimit) {
          console.log(
            `Clipboard content is too old (${timeDiff}s > ${timeLimit}s), skipping auto-paste`,
          );
          return;
        }

        console.log(
          `Clipboard content is recent (${timeDiff}s <= ${timeLimit}s), auto-pasting`,
        );
      }

      // 处理文件路径
      if (clipboardContent.files && clipboardContent.files.length > 0) {
        const files: File[] = [];

        for (const fileInfo of clipboardContent.files) {
          if (fileInfo.is_directory) {
            const placeholderBlob = new Blob([]);
            const file = new File([placeholderBlob], fileInfo.name, {
              type: "application/x-directory",
            });
            Object.defineProperty(file, "path", {
              value: fileInfo.path,
              writable: false,
            });
            files.push(file);
            continue;
          }

          const mimeType = inferMimeType(fileInfo.name);
          const placeholderBlob = new Blob([], { type: mimeType });
          const file = new File([placeholderBlob], fileInfo.name, {
            type: mimeType,
          });

          Object.defineProperty(file, "path", {
            value: fileInfo.path,
            writable: false,
          });

          files.push(file);
        }

        if (files.length > 0) {
          state.attachedText = "";
          state.attachedFiles = files;
        }
      } else if (clipboardContent.text) {
        const text = clipboardContent.text.trim();
        state.attachedFiles = [];
        state.showAllFiles = false;
        state.attachedText = text;
      }
    } catch (error) {
      console.error("Failed to auto-paste clipboard:", error);
    }
  };

  /**
   * 清理所有附件和相关状态
   */
  const clearAttachments = () => {
    state.attachedFiles = [];
    state.showAllFiles = false;
    state.attachedText = "";
  };

  /**
   * 移除指定索引的文件
   */
  const removeFile = (index: number) => {
    state.attachedFiles = state.attachedFiles.filter((_, i) => i !== index);
    if (state.attachedFiles.length <= 1) {
      state.showAllFiles = false;
    }
  };

  /**
   * 编辑文本附件 - 将文本转移到输入框
   */
  const editTextAttachment = (onEdit: (text: string) => void) => {
    onEdit(state.attachedText);
    state.attachedText = "";
  };

  /**
   * 切换显示所有文件
   */
  const toggleShowAllFiles = () => {
    state.showAllFiles = !state.showAllFiles;
  };

  /**
   * 获取匹配的命令
   * @param originAppList 原始应用列表
   * @param inputText 手动输入的文本（可选）
   */
  const getMatchedCommandsForAttachments = (
    originAppList: LaunchableItem[],
    inputText: string = "",
  ): LaunchableItem[] => {
    return getMatchedCommands(
      originAppList,
      state.attachedText,
      state.attachedFiles,
      inputText,
    );
  };

  return {
    get state() {
      return state;
    },
    handlePaste,
    handleDrop,
    handleDragOver,
    autoPasteClipboard,
    clearAttachments,
    removeFile,
    editTextAttachment,
    toggleShowAllFiles,
    getMatchedCommands: getMatchedCommandsForAttachments,
  };
}
