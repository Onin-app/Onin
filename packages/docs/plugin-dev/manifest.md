# manifest.json 详解

`manifest.json` 是插件的配置文件，放在插件根目录，定义了插件的基本信息、提供的指令、权限需求和行为模式。

## 完整字段说明

```json
{
  "id": "my-plugin",
  "name": "我的插件",
  "version": "1.0.0",
  "description": "插件描述",
  "entry": "index.html",
  "icon": "icon.png",
  "type": "ui",
  "display_mode": "inline",
  "auto_detach": false,
  "lifecycle": "lifecycle.js",
  "devMode": false,
  "devServer": "http://localhost:5173",
  "commands": [...],
  "permissions": {...}
}
```

## 基础字段

| 字段          | 类型                 | 必填 | 说明                                                                     |
| ------------- | -------------------- | ---- | ------------------------------------------------------------------------ |
| `id`          | `string`             | ✅   | 插件唯一标识符，建议用小写字母和连字符，如 `my-plugin`                   |
| `name`        | `string`             | ✅   | 插件显示名称                                                             |
| `version`     | `string`             | ✅   | 插件版本号，建议遵循 SemVer，如 `1.0.0`                                  |
| `description` | `string`             | ✅   | 插件描述                                                                 |
| `entry`       | `string`             | ✅   | 入口文件路径，UI 插件填 HTML 文件（如 `index.html`），脚本插件填 JS 文件 |
| `icon`        | `string`             | ❌   | 图标文件路径（相对于插件根目录），支持 PNG、SVG                          |
| `type`        | `"ui"` \| `"script"` | ❌   | 插件类型，默认 `"ui"`                                                    |

## 显示模式

| 字段           | 类型                     | 默认值     | 说明                            |
| -------------- | ------------------------ | ---------- | ------------------------------- |
| `display_mode` | `"inline"` \| `"window"` | `"inline"` | 插件默认显示方式                                                                       |
| `auto_detach`  | `boolean`                | `false`    | UI 插件是否始终在独立窗口中打开                                                       |
| `terminate_on_bg` | `boolean`             | `false`    | 应用隐藏到后台时是否立即结束插件运行。对于节省资源的工具类插件建议开启 |
| `run_at_startup` | `boolean`              | `false`    | 是否随 Onin 主程序启动自动加载并运行插件                                              |
| `lifecycle`    | `string`                 | `"lifecycle.js"` | 视图插件的初始化脚本路径。即便 UI 未打开，该脚本也会被执行（需 `run_at_startup` 支持） |

## 开发模式

| 字段        | 类型      | 说明                                        |
| ----------- | --------- | ------------------------------------------- |
| `devMode`   | `boolean` | 是否启用开发模式，启用后从 `devServer` 加载 |
| `devServer` | `string`  | 开发服务器地址，如 `http://localhost:5173`  |

## commands 指令列表

每个插件可以声明一组指令，用户在 Onin 搜索框输入关键词时触发：

```json
{
  "commands": [
    {
      "code": "my-command",
      "name": "我的指令",
      "description": "指令描述（可选）",
      "keywords": [
        { "name": "keyword1" },
        { "name": "keyword2", "type": "prefix" }
      ],
      "matches": [
        {
          "type": "text",
          "name": "文本匹配",
          "description": "匹配选中文本",
          "min": 1
        }
      ]
    }
  ]
}
```

### 指令字段

| 字段          | 类型     | 说明                                             |
| ------------- | -------- | ------------------------------------------------ |
| `code`        | `string` | 指令唯一标识，在 `command.handle` 中用于区分指令 |
| `name`        | `string` | 指令显示名称                                     |
| `description` | `string` | 指令描述（可选）                                 |
| `keywords`    | `array`  | 触发关键词列表                                   |
| `matches`     | `array`  | 内容匹配规则（可选），用于文件/文本触发          |

### 关键词类型

| `type` 值  | 说明                                                 |
| ---------- | ---------------------------------------------------- |
| `"prefix"` | 前缀匹配（默认），用户输入的文本以该关键词开头时触发 |
| `"fuzzy"`  | 模糊匹配                                             |
| `"exact"`  | 精确匹配                                             |

### matches 匹配规则

| `type` 值  | 说明               |
| ---------- | ------------------ |
| `"text"`   | 匹配选中的文本内容 |
| `"image"`  | 匹配选中的图片文件 |
| `"file"`   | 匹配选中的文件     |
| `"folder"` | 匹配选中的文件夹   |

```json
{
  "matches": [
    {
      "type": "file",
      "name": "处理图片",
      "extensions": [".png", ".jpg", ".webp"],
      "min": 1,
      "max": 10
    }
  ]
}
```
