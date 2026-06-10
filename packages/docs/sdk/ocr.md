# ocr

提供光学字符识别（OCR）能力，可用于识别图像文件或 Base64 编码图像中的文本信息，并提供精准的文字位置坐标。

## 导入

```typescript
import { ocr } from 'onin-sdk';
```

> **所需权限**：此 API 目前无需特殊权限即可使用。

---

## 跨平台底层支持说明

为保障“零安装包体积”以及极高的处理性能，`ocr` 底层根据不同操作系统采用了不同引擎，但向上提供完全一致的 API 与坐标体系：

- **Windows**：调用 Windows 运行时内置的 `Windows.Media.Ocr`。
- **macOS**：动态加载系统自带的 `Vision.framework` 进行识别。
- **Linux**：桥接系统中的 `tesseract` 命令。使用 Linux 平台前，请确保系统中已通过包管理器安装 `tesseract-ocr`（例如运行 `sudo apt install tesseract-ocr tesseract-ocr-chi-sim`）。**如需识别其他语言，请安装对应的语言包（如 `tesseract-ocr-eng`、`tesseract-ocr-jpn` 等）**。
- **坐标体系对齐**：所有平台输出的包围盒（Bounding Box）坐标都已被底层统一换算为了**相对于图片左上角的绝对像素坐标**。同时针对 macOS 上的中日韩（CJK）分词行为进行了适配，使其输出的单字词定位与 Windows 高度对齐。

---

## API

### `ocr.recognize(image, options?)`

识别图像中的文本。

```typescript
const result = await ocr.recognize('C:\\path\\to\\image.png', {
  language: 'zh-CN',
});
```

**参数：**

| 字段               | 类型     | 必填 | 说明                                                                                                                                                                                             |
| ------------------ | -------- | ---- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| `image`            | `string` | ✅   | 图片文件路径（绝对路径）或以 `data:image/` 开头的 Base64 数据 URL                                                                                                                                |
| `options`          | `object` | ❌   | 配置项                                                                                                                                                                                           |
| `options.language` | `string` | ❌   | 语言代码，支持 BCP-47 规范（如 `'zh-CN'`, `'en-US'`, `'ja'`, `'ko'` 等）。不指定时：<br/>- **Windows / macOS**：系统将基于偏好语言列表智能匹配识别。<br/>- **Linux**：Tesseract 默认识别为英文。 |

**返回值 `Promise<OcrResult>`：**

```typescript
interface OcrResult {
  /** 识别出的完整文本，多行结果使用 "\n" 拼接 */
  text: string;
  /** 包含行级和字词级坐标的结构化信息 */
  lines: OcrLine[];
}

interface OcrLine {
  /** 单行文本内容 */
  text: string;
  /** 单行左上角 X 轴像素坐标 */
  x: number;
  /** 单行左上角 Y 轴像素坐标 */
  y: number;
  /** 单行宽度（像素） */
  width: number;
  /** 单行高度（像素） */
  height: number;
  /** 单行内的字词结构划分 */
  words: OcrWord[];
}

interface OcrWord {
  /** 单字或单词内容 */
  text: string;
  /** 字符包围盒左上角 X 轴像素坐标 */
  x: number;
  /** 字符包围盒左上角 Y 轴像素坐标 */
  y: number;
  /** 字符包围盒宽度（像素） */
  width: number;
  /** 字符包围盒高度（像素） */
  height: number;
}
```

---

## 示例

```typescript
import { ocr, dialog, toast } from 'onin-sdk';

async function performOcr() {
  try {
    // 1. 选择要识别的图片
    const selected = await dialog.showOpen({
      title: '选择识别的图片',
      filters: [
        { name: '图片文件', extensions: ['png', 'jpg', 'jpeg', 'bmp'] },
      ],
    });

    if (!selected) return;

    await toast.info('正在识别中...', { duration: 1500 });

    // 2. 执行 OCR 识别（通过类型断言收窄 string | string[] 类型）
    const result = await ocr.recognize(selected as string);

    console.log('识别文本:', result.text);
    console.log('详细结构定位:', result.lines);

    await toast.success('OCR 识别完成！');
  } catch (error: any) {
    console.error(error);
    await toast.error(`识别失败: ${error.message || error}`);
  }
}
```
