# 插件匹配指令 API 实现文档

## 概述

插件匹配指令（Match Commands）是一个强大的功能，允许插件自动检测用户粘贴到输入框的内容（文本、图片、文件、文件夹），并在满足条件时自动显示相应的命令供用户选择执行。

## 核心概念

### 匹配指令 vs 功能指令

- **静态指令**：在 manifest.json 中定义，用户通过关键词主动调用
- **动态指令**：通过 `command.register()` SDK API 动态注册
- **匹配指令**：自动检测输入框中粘贴的内容，当内容符合条件时自动在列表中显示

### 一切皆指令

匹配指令本质上也是通过指令系统执行的，它只是提供了一种自动触发的方式。

### 三层优雅降级模型

为了提升开发者体验，我们采用了三层优雅降级模型：

#### 1️⃣ 开发者层（配置层）
- 开发者只需要配置简单易记的**扩展名**（如 `[".png", ".jpg"]`）
- 无需记忆复杂的 MIME 类型字符串（如 `"image/jpeg"`）
- 配置更简洁、更直观、更不容易出错

#### 2️⃣ 系统加载层（解析层）
- 系统自动将扩展名映射为标准 MIME 类型
- 内置常见文件类型的映射表（100+ 种文件类型）
- 如果配置中同时存在 `extensions` 和 `mimeTypes`，以 `mimeTypes` 为准
- 这样既保持了灵活性，又提供了便利性

#### 3️⃣ 运行层（判断层）
- 优先基于文件的 MIME 类型进行判断（更准确）
- 如果文件没有 MIME 类型，系统会从文件名推断
- 如果 MIME 类型判断失败，fallback 到扩展名判断
- 双重保障，确保匹配的可靠性

**为什么这样设计？**
1. **对开发者友好**：MIME 类型很难记住，大部分开发者需要查资料
2. **避免重复**：MIME 类型和扩展名本质上是重复的信息
3. **更可靠**：双重判断机制，即使文件 MIME 类型不准确也能正确匹配

## 配置结构

在插件的 `manifest.json` 中，`matches` 配置在每个 `command` 内部，与 `keywords` 同级：

```json
{
  "commands": [
    {
      "code": "process-url",
      "name": "处理URL",
      "description": "处理粘贴的URL",
      "keywords": [
        {
          "name": "url",
          "type": "prefix"
        }
      ],
      "matches": [
        {
          "type": "text",
          "name": "URL检测",
          "description": "检测文本中的URL",
          "regexp": "https?://[\\w\\-._~:/?#\\[\\]@!$&'()*+,;=%]+",
          "min": 1,
          "max": 5
        }
      ]
    }
  ]
}
```

## 匹配类型

### 1. text - 文本匹配

检测粘贴的文本内容。

**参数：**
- `type`: `"text"`
- `name`: 匹配名称
- `description`: 匹配描述
- `regexp`: （可选）正则表达式，用于匹配文本内容
- `min`: （可选）最小字符数
- `max`: （可选）最大字符数

**示例：**
```json
{
  "type": "text",
  "name": "URL检测",
  "description": "检测文本中的URL",
  "regexp": "https?://[\\w\\-._~:/?#\\[\\]@!$&'()*+,;=%]+",
  "min": 10,
  "max": 1000
}
```

**匹配逻辑：**
1. **min/max 始终表示字符数**（不是匹配次数）
2. **regexp 是额外的匹配条件**
3. **执行顺序**：先检查字符数（min/max），通过后再检查正则表达式

**示例说明：**
- 上面的配置表示：文本长度在 10-1000 字符之间，且包含 URL
- 如果只配置 `min/max`，不配置 `regexp`，则只检查字符数
- 如果只配置 `regexp`，不配置 `min/max`，则只检查正则匹配

### 2. image - 图片匹配

检测粘贴的图片文件。

**参数：**
- `type`: `"image"`
- `name`: 匹配名称
- `description`: 匹配描述
- `extensions`: （可选）文件扩展名数组，如 `[".png", ".jpg"]`
- `min`: （可选）最少图片数量
- `max`: （可选）最多图片数量

**示例：**
```json
{
  "type": "image",
  "name": "图片检测",
  "description": "检测粘贴的图片",
  "extensions": [".png", ".jpg", ".jpeg"],
  "min": 1,
  "max": 10
}
```

**系统会自动处理：**
- 将 `[".png", ".jpg", ".jpeg"]` 映射为 `["image/png", "image/jpeg"]`
- 运行时优先使用 MIME 类型判断
- 如果 MIME 类型不可靠，fallback 到扩展名判断

### 3. file - 文件匹配

检测粘贴的文件（不包括图片和文件夹）。

**参数：**
- `type`: `"file"`
- `name`: 匹配名称
- `description`: 匹配描述
- `extensions`: （推荐）文件扩展名数组，如 `[".pdf", ".docx"]` 或 `["*"]`
- `mimeTypes`: （可选）MIME 类型数组，通常不需要指定
- `min`: （可选）最少文件数量
- `max`: （可选）最多文件数量

**推荐示例（只使用 extensions）：**
```json
{
  "type": "file",
  "name": "PDF文件检测",
  "description": "检测粘贴的PDF文件",
  "extensions": [".pdf"],
  "min": 1
}
```

**系统会自动处理：**
- 将 `[".pdf"]` 自动映射为 `["application/pdf"]`
- 运行时优先使用 MIME 类型判断
- 如果 MIME 类型不可靠，fallback 到扩展名判断



**注意：**
- 只需配置 `extensions`，系统会自动处理 MIME 类型
- 支持通配符 `"*"` 匹配所有文件
- 系统内置 100+ 种常见文件类型的映射

### 4. folder - 文件夹匹配

检测粘贴的文件夹。

**参数：**
- `type`: `"folder"`
- `name`: 匹配名称
- `description`: 匹配描述
- `min`: （可选）最少文件夹数量
- `max`: （可选）最多文件夹数量

**示例：**
```json
{
  "type": "folder",
  "name": "文件夹检测",
  "description": "检测粘贴的文件夹",
  "min": 1
}
```

## 匹配逻辑

1. 当用户粘贴内容到输入框时，系统会检查所有插件命令的 `matches` 配置
2. 对于每个命令，只要有**任意一个**匹配条件满足，该命令就会显示在列表中
3. 用户可以点击匹配的命令来执行
4. 匹配的命令会**优先显示**，覆盖普通的应用列表

## 类型检测方案

系统通过以下方式检测内容类型：

- **text**: 检查 `attachedText` 变量
- **image**: 检查 `file.type` 是否以 `"image/"` 开头
- **file**: 检查 `file.type` 不是 `"image/"` 且不是 `"application/x-directory"`
- **folder**: 检查 `file.type === "application/x-directory"`

## 完整示例

查看 `example-plugin-match-test` 目录获取完整的示例插件。

### manifest.json

```json
{
  "id": "com.example.match-test",
  "name": "匹配指令测试插件",
  "version": "1.0.0",
  "description": "测试插件匹配指令功能",
  "entry": "index.js",
  "type": "script",
  "commands": [
    {
      "code": "process-url",
      "name": "处理URL",
      "description": "检测并处理粘贴的URL",
      "keywords": [
        {
          "name": "url",
          "type": "prefix"
        }
      ],
      "matches": [
        {
          "type": "text",
          "name": "URL检测",
          "description": "检测文本中的URL",
          "regexp": "https?://[\\w\\-._~:/?#\\[\\]@!$&'()*+,;=%]+",
          "min": 1
        }
      ]
    }
  ]
}
```

### index.js

```javascript
import { command, notification } from 'onin-sdk';

// 处理命令执行
command.handle((commandCode, args) => {
  console.log('命令被调用 >>>', commandCode, args);
  
  if (commandCode === 'process-url') {
    notification.show({
      title: '检测到URL',
      body: `找到 URL: ${args?.text || '未知'}`,
    });
  }
});
```

### 动态注册命令（可选）

除了在 manifest.json 中静态声明命令，插件也可以通过 SDK 动态注册命令：

```javascript
import { command } from 'onin-sdk';

// 动态注册命令
await command.register({
  code: 'bookmark-1',
  name: '我的书签',
  keywords: [{ name: '书签' }],
  matches: [{ type: 'text', name: 'URL', regexp: '^https?://' }]
});

// 移除命令
await command.remove('bookmark-1');
```

## 参数结构

插件命令接收到的 `args` 参数结构如下：

```javascript
{
  input: "用户在输入框中输入的内容",  // 如果用户有输入
  text: "粘贴的文本内容",             // 如果有粘贴的文本
  images: [                            // 如果有图片
    {
      name: "图片名称",
      path: "文件路径",
      type: "image/png",
      size: 12345
    }
  ],
  textFiles: [                  // 如果有纯文本文件（.txt, .md等）
    {
      name: "文件名",
      path: "文件路径",
      type: "text/plain",
      size: 12345
    }
  ],
  files: [                      // 如果有其他文件
    {
      name: "文件名",
      path: "文件路径",
      type: "application/pdf",
      size: 12345
    }
  ],
  folders: [                    // 如果有文件夹
    {
      name: "文件夹名",
      path: "文件夹路径",
      type: "application/x-directory",
      size: 0
    }
  ]
}
```

**注意**：
- `input`: 用户在输入框中输入的内容（不是粘贴的）
- `text`: 用户粘贴的纯文本内容
- 文件会自动分类到对应的数组中
- 支持同时处理多种类型的内容
- 纯文本文件（.txt, .md）会单独分类到 `textFiles`

## 测试步骤

1. 将 `example-plugin-match-test` 文件夹复制到应用的插件目录
2. 重启应用或刷新插件列表
3. 复制一个 URL（如 `https://github.com`）
4. 打开应用主窗口，URL 会自动粘贴
5. 应该看到"处理URL"命令出现在列表中
6. 点击该命令，会显示通知

### 测试混合内容

1. 同时复制多个文件（图片、文本文件、其他文件、文件夹）
2. 打开应用主窗口
3. 应该看到"处理混合内容"命令出现
4. 点击后会显示所有类型的统计信息

## TypeScript 类型定义

SDK 中已添加完整的类型定义（`plugins-sdk/src/types/permissions.ts`）：

```typescript
export interface PluginCommandMatch {
  type: 'text' | 'image' | 'file' | 'folder';
  name: string;
  description: string;
  regexp?: string;
  min?: number;
  max?: number;
  mimeTypes?: string[];
  extensions?: string[];
}

export interface PluginCommand {
  code: string;
  name: string;
  description: string;
  keywords: PluginCommandKeyword[];
  matches?: PluginCommandMatch[];
}
```

## 实现细节

### 后端（Rust）

1. `PluginCommandMatch` 结构体定义在 `plugin_manager.rs`
2. `CommandMatch` 结构体定义在 `shared_types.rs`
3. `command_manager.rs` 中将插件的 matches 转换为 Command 的 matches
4. `unified_launch_manager.rs` 中将 Command 的 matches 传递到 LaunchableItem

### 前端（Svelte）

1. `matchCommand.ts` 包含所有匹配逻辑
2. `+page.svelte` 中调用匹配函数并显示结果
3. 匹配的命令优先显示，覆盖普通列表
4. 键盘导航自动适配匹配命令列表

## 使用场景示例

### 场景 1：区分输入和粘贴

```javascript
command.handle((commandCode, args) => {
  if (commandCode === 'search') {
    // 优先使用输入框内容，其次使用粘贴内容
    const query = args?.input || args?.text || '';
    
    if (args?.input) {
      console.log('用户主动输入:', query);
    } else if (args?.text) {
      console.log('用户粘贴内容:', query);
    }
  }
});
```

### 场景 2：简单的文件类型检测

```json
{
  "type": "file",
  "name": "Markdown文件",
  "description": "只接受 .md 或 .markdown 文件",
  "extensions": [".md", ".markdown"],
  "min": 1
}
```

系统会自动处理：
- 将 `[".md", ".markdown"]` 映射为 `["text/markdown"]`
- 运行时优先使用 MIME 类型判断
- 如果文件没有 MIME 类型或不准确，使用扩展名判断

### 场景 3：处理纯文本文件

```javascript
command.handle((commandCode, args) => {
  if (commandCode === 'process-text-files') {
    const textFiles = args?.textFiles || [];
    
    // textFiles 包含 .txt, .md 等纯文本文件
    textFiles.forEach(file => {
      console.log('处理文本文件:', file.name, file.path);
    });
  }
});
```

## 注意事项

1. **推荐使用 extensions**：
   - 只需配置扩展名，系统会自动处理 MIME 类型
   - 更简单、更直观、不容易出错
   - 例如：`["extensions": [".png", ".jpg"]]`

2. **正则表达式**：在 JSON 中需要转义反斜杠（`\\`）

3. **通配符**：`extensions` 支持 `"*"` 通配符匹配所有文件

4. **优先级**：匹配命令会完全覆盖普通应用列表

5. **性能**：匹配检查在每次内容变化时执行，保持逻辑简单高效

6. **三层降级**：
   - 系统会自动将 `extensions` 映射为 `mimeTypes`
   - 运行时优先使用 MIME 类型判断（更准确）
   - 如果 MIME 类型不可靠，fallback 到扩展名判断

7. **input vs text**：
   - `input`: 用户在输入框中输入的内容（可能为空）
   - `text`: 用户粘贴的纯文本内容（可能为空）
   - 两者可以同时存在

8. **支持的文件类型**：
   - 系统内置 100+ 种常见文件类型的映射
   - 包括图片、视频、音频、文档、代码、压缩包等
   - 查看 `src/lib/utils/mimeTypeMap.ts` 获取完整列表

## 未来改进

- [ ] 支持更复杂的匹配条件组合（AND/OR）
- [ ] 支持自定义匹配函数
- [ ] 添加匹配优先级配置
- [ ] 支持匹配结果预览
