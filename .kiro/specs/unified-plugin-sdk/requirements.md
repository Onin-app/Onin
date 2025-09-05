# Requirements Document

## Introduction

创建一个统一的插件 SDK，为 headless 插件和 UI 插件提供一致的 API 接口。当前系统中存在两种不同的插件调用方式，需要通过 SDK 层抽象这些差异，让开发者能够使用统一的方式调用插件功能。

## Requirements

### Requirement 1

**User Story:** 作为插件开发者，我希望能够使用统一的 API 调用插件功能，而不需要关心底层是 headless 还是 UI 插件，这样我可以专注于业务逻辑而不是技术细节。

#### Acceptance Criteria

1. WHEN 开发者调用 SDK API THEN 系统 SHALL 自动检测当前环境类型（headless 或 UI）
2. WHEN 系统检测到 headless 环境 THEN SDK SHALL 使用 Deno.core.ops.op_invoke 方式调用
3. WHEN 系统检测到 UI 环境 THEN SDK SHALL 使用 @tauri-apps/api/core invoke 方式调用
4. WHEN API 调用成功 THEN SDK SHALL 返回统一格式的成功响应
5. WHEN API 调用失败 THEN SDK SHALL 返回统一格式的错误响应

### Requirement 2

**User Story:** 作为插件开发者，我希望 SDK 提供类型安全的 TypeScript 接口，这样我可以在开发时获得更好的代码提示和错误检查。

#### Acceptance Criteria

1. WHEN 开发者导入 SDK THEN 系统 SHALL 提供完整的 TypeScript 类型定义
2. WHEN 开发者调用 API 方法 THEN IDE SHALL 显示正确的参数类型和返回类型
3. WHEN 开发者传入错误类型的参数 THEN TypeScript 编译器 SHALL 报告类型错误
4. WHEN SDK 返回数据 THEN 返回值 SHALL 具有明确的类型定义

### Requirement 3

**User Story:** 作为插件开发者，我希望能够调用通知功能，无论在哪种环境下都能正常工作，这样我可以向用户发送消息。

#### Acceptance Criteria

1. WHEN 开发者调用 showNotification 方法 THEN SDK SHALL 接受 title 和 body 参数
2. WHEN 在 headless 环境调用通知 THEN SDK SHALL 使用 op_invoke("show_notification") 方式
3. WHEN 在 UI 环境调用通知 THEN SDK SHALL 使用 invoke("show_notification") 方式
4. WHEN 通知发送成功 THEN SDK SHALL 返回成功状态
5. WHEN 通知发送失败 THEN SDK SHALL 返回错误信息

### Requirement 4

**User Story:** 作为系统维护者，我希望 SDK 具有良好的错误处理机制，这样当出现问题时能够快速定位和解决。

#### Acceptance Criteria

1. WHEN SDK 无法检测环境类型 THEN 系统 SHALL 抛出明确的错误信息
2. WHEN 底层 API 调用失败 THEN SDK SHALL 捕获错误并转换为统一格式
3. WHEN 传入无效参数 THEN SDK SHALL 在调用前进行参数验证
4. WHEN 发生错误 THEN SDK SHALL 提供有用的调试信息

### Requirement 5

**User Story:** 作为项目维护者，我希望 SDK 具有可扩展的架构，这样未来可以轻松添加新的插件功能和环境支持。

#### Acceptance Criteria

1. WHEN 需要添加新的插件功能 THEN SDK 架构 SHALL 支持简单的功能扩展
2. WHEN 需要支持新的环境类型 THEN SDK SHALL 允许添加新的环境适配器
3. WHEN 修改内部实现 THEN 外部 API 接口 SHALL 保持稳定
4. WHEN 添加新功能 THEN 现有功能 SHALL 不受影响