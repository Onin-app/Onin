# 插件指令执行功能使用说明

## 功能概述

现在已经实现了插件指令执行功能，支持：

1. **Headless插件**：通过JS运行时执行指令
2. **Webview插件**：通过IPC通信执行指令
3. **指令配置**：通过manifest.json配置插件指令
4. **指令执行**：宿主应用可以选择并执行插件指令

## 指令配置（manifest.json）

插件指令通过manifest.json文件配置：

```json
{
  "id": "com.example.myplugin",
  "name": "我的插件",
  "version": "1.0.0",
  "description": "示例插件",
  "entry": "index.js",
  "type": "headless",
  "commands": [
    {
      "code": "hello",
      "name": "问候指令",
      "description": "向用户问候",
      "keywords": [
        {"name": "hello", "type": "text"},
        {"name": "你好", "type": "text"},
        {"name": "问候", "type": "text"}
      ]
    },
    {
      "code": "calculate",
      "name": "计算指令",
      "description": "执行简单计算",
      "keywords": [
        {"name": "calc", "type": "text"},
        {"name": "计算", "type": "text"},
        {"name": "add", "type": "text"}
      ]
    }
  ]
}
```

## SDK使用方法

### 注册指令处理器

```typescript
import baize from '@baize/plugin-sdk';

// 注册指令处理器
baize.registerCommandHandler(async (command: string, args: any) => {
  console.log(`执行指令: ${command}`, args);
  
  switch (command) {
    case 'hello':
      return `Hello, ${args?.name || 'World'}!`;
    case 'calculate':
      return args.a + args.b;
    default:
      throw new Error(`未知指令: ${command}`);
  }
});
```

## Headless插件示例

```javascript
// plugin-entry.js
import baize from '@baize/plugin-sdk';

// 注册指令处理器
baize.registerCommandHandler(async (command, args) => {
  switch (command) {
    case 'system_info':
      return {
        timestamp: Date.now(),
        platform: 'headless',
        message: 'System info retrieved successfully'
      };
    case 'process_data':
      // 处理数据
      return `Processed: ${JSON.stringify(args)}`;
    default:
      throw new Error(`Unknown command: ${command}`);
  }
});

console.log('Headless plugin loaded successfully');
```

## Webview插件示例

```html
<!-- plugin-entry.html -->
<!DOCTYPE html>
<html>
<head>
    <title>Webview Plugin</title>
    <script type="module">
        import baize from '@baize/plugin-sdk';
        
        // 注册指令处理器
        await baize.registerCommandHandler(async (command, args) => {
            const output = document.getElementById('output');
            
            switch (command) {
                case 'show_message':
                    output.innerHTML = `<p>消息: ${args?.message || 'Hello!'}</p>`;
                    return 'Message displayed';
                case 'get_input':
                    const input = document.getElementById('userInput');
                    return input.value;
                default:
                    throw new Error(`Unknown command: ${command}`);
            }
        });
        
        // 注册指令
        console.log('Webview plugin loaded successfully');
    </script>
</head>
<body>
    <h1>Webview Plugin</h1>
    <input id="userInput" type="text" placeholder="输入内容...">
    <div id="output"></div>
</body>
</html>
```

## 宿主应用API

### 执行插件指令

```typescript
import { invoke } from '@tauri-apps/api/core';

// 执行插件指令
const result = await invoke('execute_plugin_command', {
  pluginId: 'com.example.myplugin',
  commandName: 'hello',
  args: { name: 'Alice' }
});

console.log('执行结果:', result);
```

### 获取插件指令列表

```typescript
// 获取所有插件指令（从manifest.json配置中读取）
const commands = await invoke('get_plugin_commands_list');
console.log('插件指令:', commands);
```

## 指令执行流程

### Headless插件
1. 宿主应用调用 `execute_plugin_command`
2. 后端加载插件JS代码
3. 构造指令执行代码并在JS运行时中执行
4. 调用插件注册的 `__BAIZE_COMMAND_HANDLER__` 函数
5. 返回执行结果

### Webview插件
1. 宿主应用调用 `execute_plugin_command`
2. 后端生成请求ID并发送IPC事件到插件窗口
3. 插件接收事件并调用注册的处理器
4. 插件通过 `plugin_command_result` 返回结果
5. 后端接收结果并返回给调用者

## 错误处理

- 指令不存在：返回 "Command not found" 错误
- 插件不存在：返回 "Plugin not found" 错误  
- 执行超时：Webview插件30秒超时
- 处理器异常：捕获并返回错误信息

## 修复的问题

### 1. 插件代码重复执行问题
**问题**：每次执行指令时整个插件代码都会重新运行。

**解决方案**：
- 实现了全局JS运行时管理器 `PluginRuntimeManager`
- 每个插件只创建一次JS运行时实例
- 插件代码只在首次加载时执行一次
- 后续指令执行只调用已注册的处理器

### 2. Console输出问题
**问题**：插件的console.log不在宿主webview控制台显示。

**解决方案**：
- 重写了插件运行时的console对象
- Console输出通过IPC事件发送到前端
- 在前端监听 `plugin_console_log` 事件并输出到浏览器控制台

### 前端集成Console监听

```typescript
// 在你的主应用中添加
import { setupPluginConsoleListener } from '$lib/plugin-console';

// 在应用启动时调用
setupPluginConsoleListener();
```

## 注意事项

1. Headless插件现在使用持久化的JS运行时，插件状态会保持
2. 插件代码只在首次执行指令时加载一次
3. Console输出会同时显示在终端和浏览器控制台
4. Webview插件需要先打开插件窗口才能执行指令
5. 指令参数必须是可序列化的JSON值
6. 建议为指令添加适当的错误处理和验证