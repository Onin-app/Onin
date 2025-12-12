# 插件市场调试指南

## 问题 1：Icon 不显示

### 调试步骤

1. **打开浏览器开发者工具**（F12）

2. **查看已安装插件数据**
   - 切换到 Console 标签
   - 在已安装页面，查看日志中的插件数据
   - 检查 `icon` 字段的值

3. **检查 icon 字段**
   ```javascript
   // 在控制台执行
   plugins.forEach(p => console.log(p.name, p.icon));
   ```

4. **可能的情况**：
   - `icon` 字段为 `undefined` → 后端没有读取 manifest 中的 icon
   - `icon` 是相对路径（如 `icon.svg`）→ 需要转换为 `plugin://` URL
   - `icon` 是完整 URL → 应该能直接显示

### 解决方案

**如果 icon 是相对路径**：
- 已添加 `getPluginIconUrl()` 函数
- 会自动转换为 `plugin://{dir_name}/{icon}`

**如果 icon 字段不存在**：
- 检查插件的 manifest.json 是否有 `icon` 字段
- 后端的 `PluginManifest` 结构已添加 `icon` 字段

---

## 问题 2：已安装插件在市场没有置灰

### 调试步骤

1. **查看控制台日志**
   ```
   [Marketplace] Installed plugin IDs: ["com.web-translate.20251202", ...]
   [Marketplace] Loaded plugins: [{ id: "translate", name: "...", icon: "..." }]
   ```

2. **检查 ID 是否匹配**
   - 已安装插件的 ID：`com.web-translate.20251202`
   - 市场插件的 ID：`translate`（你说接口已改为 manifest 中的 ID）

3. **关键问题**：
   - 如果市场接口返回的 `id` 是 `translate`
   - 但已安装插件的 `id` 是 `com.web-translate.20251202`
   - 它们不匹配，所以 `installedPluginIds.has(plugin.id)` 返回 `false`

### 解决方案

**确认后端接口返回的 ID**：
```bash
curl "http://localhost:3001/api/v1/plugins" | jq '.data[].id'
```

应该返回：
```json
"com.web-translate.20251202"  // ✅ 正确（manifest 中的真实 ID）
```

而不是：
```json
"translate"  // ❌ 错误（市场的简短 ID）
```

**如果 ID 确实匹配**：
- 检查 `loadInstalledPlugins()` 是否被调用
- 检查控制台日志中的 `installedPluginIds`
- 检查 `PluginCard` 的 `isInstalled` prop 是否正确传递

---

## 快速测试

在浏览器控制台执行：

```javascript
// 1. 查看市场插件 ID
console.log('Market plugins:', plugins.map(p => p.id));

// 2. 查看已安装插件 ID
console.log('Installed IDs:', Array.from(installedPluginIds));

// 3. 检查是否匹配
const marketId = plugins[0]?.id;
const isInstalled = installedPluginIds.has(marketId);
console.log(`Market plugin ${marketId} installed:`, isInstalled);
```

---

## 预期结果

### Icon 显示
- ✅ 市场插件：显示 HTTP URL 的图标
- ✅ 已安装插件：显示 `plugin://` 协议的图标

### 已安装状态
- ✅ 已安装插件：按钮显示"已安装"，灰色，不可点击
- ✅ 未安装插件：按钮显示"安装"，蓝色，可点击

---

## 如果问题仍然存在

请提供：
1. 控制台中的日志输出
2. 市场接口返回的插件数据（`plugins` 数组）
3. 已安装插件数据（`installedPluginIds` Set）
4. 插件的 manifest.json 内容
