# settings

插件设置页 API，在 Onin 的插件设置界面动态生成配置表单。

## 导入

```typescript
import { settings } from 'onin-plugin-sdk';
```

## API

### `settings.useSettingsSchema(fields)`

注册插件设置表单的字段定义。调用后，用户可以在插件设置页看到并编辑这些字段。

```typescript
await settings.useSettingsSchema([
  {
    key: 'apiKey',
    label: 'API 密钥',
    type: 'password',
    required: true,
    description: '从服务商控制台获取',
  },
  {
    key: 'language',
    label: '语言',
    type: 'select',
    defaultValue: 'zh-CN',
    options: [
      { label: '中文', value: 'zh-CN' },
      { label: 'English', value: 'en-US' },
    ],
  },
]);
```

### `settings.get()`

获取当前所有设置值。

```typescript
const values = await settings.get();
console.log(values.apiKey); // 用户填写的值
```

### `settings.getField(key)`

获取单个设置项的值。

```typescript
const apiKey = await settings.getField('apiKey');
```

## 支持的字段类型

| `type`        | 说明             | 额外字段                                |
| ------------- | ---------------- | --------------------------------------- |
| `text`        | 单行文本         | `placeholder`, `maxLength`, `minLength` |
| `password`    | 密码（星号隐藏） | `placeholder`                           |
| `textarea`    | 多行文本         | `placeholder`                           |
| `number`      | 数字             | `min`, `max`, `step`                    |
| `switch`      | 开关（boolean）  | —                                       |
| `select`      | 下拉选择         | `options`                               |
| `radio`       | 单选             | `options`                               |
| `multiSelect` | 多选             | `options`                               |
| `color`       | 颜色选择器       | —                                       |
| `date`        | 日期             | —                                       |
| `time`        | 时间             | —                                       |
| `slider`      | 滑块             | `min`, `max`, `step`                    |
| `button`      | 按钮（触发动作） | `buttonText`                            |

## 完整示例

```typescript
import { lifecycle, settings, command } from 'onin-plugin-sdk';

lifecycle.onLoad(async () => {
  // 注册设置
  await settings.useSettingsSchema([
    {
      key: 'apiKey',
      label: 'API 密钥',
      type: 'password',
      required: true,
    },
    {
      key: 'maxResults',
      label: '最大结果数',
      type: 'slider',
      defaultValue: 10,
      min: 5,
      max: 50,
      step: 5,
    },
    {
      key: 'autoRefresh',
      label: '自动刷新',
      type: 'switch',
      defaultValue: true,
    },
  ]);

  // 读取设置，执行初始化
  await command.handle(async (code, args) => {
    const config = await settings.get();
    if (!config.apiKey) {
      return { error: '请先在插件设置中填写 API 密钥' };
    }
    // 使用 config.apiKey 进行请求...
  });
});
```
