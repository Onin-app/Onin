import { invoke } from '../core/ipc';

export interface OcrWord {
  text: string;
  x: number;
  y: number;
  width: number;
  height: number;
}

export interface OcrLine {
  text: string;
  x: number;
  y: number;
  width: number;
  height: number;
  words: OcrWord[];
}

export interface OcrResult {
  text: string;
  lines: OcrLine[];
}

export interface OcrOptions {
  /**
   * BCP-47 language tag (e.g. "zh-CN", "en-US").
   * If not specified, the system's user profile languages are used.
   */
  language?: string;
}

/**
 * Recognize text from an image (either a file path or a base64 encoded image string)
 * @param image - The path to the image file or base64 data URL
 * @param options - Optional OCR configurations like language
 */
async function recognize(
  image: string,
  options?: OcrOptions,
): Promise<OcrResult> {
  return invoke<OcrResult>('plugin_ocr_recognize', { image, options });
}

export const ocr = {
  recognize,
};
