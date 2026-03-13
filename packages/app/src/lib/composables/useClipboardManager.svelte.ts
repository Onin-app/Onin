/**
 * Clipboard Manager Composable
 *
 * 管理剪贴板内容和文件附件的状态和逻辑
 * 遵循单一职责原则，只处理剪贴板/附件相关功能
 */
import { invoke } from "@tauri-apps/api/core";
import { inferMimeType } from "$lib/utils/mimeTypeMap";
import type { LaunchableItem } from "$lib/type";
import { getMatchedCommands } from "$lib/utils/matchCommand";

export interface ClipboardState {
  attachedFiles: File[];
  attachedText: string;
  showAllFiles: boolean;
}

export interface ClipboardManagerReturn {
  state: ClipboardState;
  handlePaste: (e: ClipboardEvent) => Promise<void>;
  handleDrop: (e: DragEvent) => void;
  handleDragOver: (e: DragEvent) => void;
  autoPasteClipboard: () => Promise<void>;
  clearAttachments: () => void;
  removeFile: (index: number) => void;
  editTextAttachment: (onEdit: (text: string) => void) => void;
  toggleShowAllFiles: () => void;
  getMatchedCommands: (
    originAppList: LaunchableItem[],
    inputText?: string,
  ) => LaunchableItem[];
}

export function useClipboardManager(): ClipboardManagerReturn {
  let state = $state<ClipboardState>({
    attachedFiles: [],
    attachedText: "",
    showAllFiles: false,
  });

  const handlePaste = async (e: ClipboardEvent) => {
    const items = e.clipboardData?.items;
    if (!items || items.length === 0) return;

    let hasFile = false;
    let hasText = false;
    for (let i = 0; i < items.length; i++) {
      if (items[i].kind === "file") hasFile = true;
      if (items[i].kind === "string" && items[i].type === "text/plain") {
        hasText = true;
      }
    }

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
        pastedText = await new Promise<string>((resolve) => {
          item.getAsString(resolve);
        });
      }
    }

    if (files.length > 0) {
      state.attachedText = "";
      state.attachedFiles = [...state.attachedFiles, ...files];
    } else if (pastedText) {
      state.attachedFiles = [];
      state.showAllFiles = false;
      state.attachedText = pastedText.trim();
    }
  };

  const handleDrop = (e: DragEvent) => {
    e.preventDefault();
    const files = Array.from(e.dataTransfer?.files || []);
    if (files.length > 0) {
      state.attachedFiles = [...state.attachedFiles, ...files];
    }
  };

  const handleDragOver = (e: DragEvent) => {
    e.preventDefault();
  };

  const autoPasteClipboard = async () => {
    try {
      const clipboardContent = await invoke<{
        text?: string;
        files?: Array<{ path: string; name: string; is_directory: boolean }>;
        timestamp?: number;
      }>("get_clipboard_content");

      const config = await invoke<{ auto_paste_time_limit: number }>(
        "get_app_config",
      );
      const timeLimit = config.auto_paste_time_limit;

      if (timeLimit > 0) {
        if (!clipboardContent.timestamp) {
          return;
        }

        const clipboardTimestamp = clipboardContent.timestamp;
        const currentTime = Math.floor(Date.now() / 1000);
        const timeDiff = currentTime - clipboardTimestamp;

        if (timeDiff > timeLimit) {
          return;
        }
      }

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

  const clearAttachments = () => {
    state.attachedFiles = [];
    state.showAllFiles = false;
    state.attachedText = "";
  };

  const removeFile = (index: number) => {
    state.attachedFiles = state.attachedFiles.filter((_, i) => i !== index);
    if (state.attachedFiles.length <= 1) {
      state.showAllFiles = false;
    }
  };

  const editTextAttachment = (onEdit: (text: string) => void) => {
    onEdit(state.attachedText);
    state.attachedText = "";
  };

  const toggleShowAllFiles = () => {
    state.showAllFiles = !state.showAllFiles;
  };

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
